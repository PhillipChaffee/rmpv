//! Extracts functions and their branch arms from mpv's C sources.

use crate::preprocess::LineMap;
use anyhow::{Context as _, Result};
use sha2::{Digest as _, Sha256};
use std::{collections::BTreeSet, fmt::Write as _};
use tree_sitter::{Node, Parser};

/// One path through a function: the straight-line body (path 0) or a branch arm.
#[derive(Debug)]
pub(crate) struct Arm {
    pub(crate) kind: &'static str,
    /// Row in the original file. An arm a macro expands to starts on the macro's row.
    pub(crate) start_row: usize,
    /// Byte range in the text the parser read, which is the preprocessed text if there was one.
    start_byte: usize,
    end_byte: usize,
}

#[derive(Debug)]
pub(crate) struct Function {
    pub(crate) name: String,
    /// Digest of the whitespace-normalized original lines, so a claim goes stale when mpv changes.
    pub(crate) hash: String,
    pub(crate) start_row: usize,
    pub(crate) end_row: usize,
    /// Index is the path id. Path 0 spans the whole function.
    pub(crate) arms: Vec<Arm>,
}

#[derive(Debug)]
pub(crate) struct SourceFile {
    pub(crate) path: String,
    /// Whether the functions come from the preprocessed text, so macro branches are counted.
    pub(crate) preprocessed: bool,
    pub(crate) functions: Vec<Function>,
    pub(crate) code_lines: usize,
    pub(crate) function_lines: usize,
    /// Lines in regions the parser gave up on. Functions in them are invisible to the tool.
    pub(crate) unparsed_lines: usize,
    text: String,
    line_starts: Vec<usize>,
    /// Per original row, the parsed-text byte that stands for the row when matching it to arms.
    row_offsets: Vec<Option<usize>>,
}

/// Text handed to the parser, and the original row each of its rows came from.
/// `None` marks rows from another file, such as an included header.
struct Parsed<'a> {
    text: &'a str,
    origins: Vec<Option<usize>>,
}

impl Parsed<'_> {
    /// The original row of a parsed row, or of the nearest row above it from this file.
    fn row(&self, row: usize) -> Option<usize> {
        self.origins
            .get(..=row)?
            .iter()
            .rev()
            .find_map(|origin| *origin)
    }
}

impl SourceFile {
    /// Parses the file as written, without running the preprocessor.
    pub(crate) fn parse(path: String, text: String) -> Result<Self> {
        let parsed = Parsed {
            text: &text,
            origins: (0..=text.matches('\n').count()).map(Some).collect(),
        };
        let (functions, unparsed_lines, row_offsets) = extract(&path, &parsed, &text)?;
        Ok(Self::new(
            path,
            false,
            text,
            functions,
            unparsed_lines,
            row_offsets,
        ))
    }

    /// Parses the preprocessor's output for `text`, keeping functions defined in `text` itself.
    pub(crate) fn parse_preprocessed(
        path: String,
        text: String,
        preprocessed: &str,
    ) -> Result<Self> {
        let map = LineMap::parse(preprocessed);
        let main = map
            .main_file()
            .with_context(|| format!("no linemarkers in the preprocessed {path}"))?;
        let blanked = prepare_for_parser(preprocessed);
        let parsed = Parsed {
            text: &blanked,
            origins: (0..=blanked.matches('\n').count())
                .map(|row| {
                    let origin = map.origin(row).filter(|(file, _)| *file == main);
                    origin.map(|(_, line)| line)
                })
                .collect(),
        };
        let (functions, unparsed_lines, row_offsets) = extract(&path, &parsed, &text)?;
        Ok(Self::new(
            path,
            true,
            text,
            functions,
            unparsed_lines,
            row_offsets,
        ))
    }

    fn new(
        path: String,
        preprocessed: bool,
        text: String,
        functions: Vec<Function>,
        unparsed_lines: usize,
        row_offsets: Vec<Option<usize>>,
    ) -> Self {
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(offset, _)| offset + 1))
            .collect();
        let is_code: Vec<bool> = text.lines().map(|line| !line.trim().is_empty()).collect();
        let code_lines = is_code.iter().filter(|is_code| **is_code).count();
        let function_lines = functions
            .iter()
            .map(|function| {
                let rows = is_code
                    .iter()
                    .take(function.end_row + 1)
                    .skip(function.start_row);
                rows.filter(|is_code| **is_code).count()
            })
            .sum();
        Self {
            path,
            preprocessed,
            functions,
            code_lines,
            function_lines,
            unparsed_lines,
            text,
            line_starts,
            row_offsets,
        }
    }

    pub(crate) fn line(&self, row: usize) -> &str {
        let start = self
            .line_starts
            .get(row)
            .copied()
            .unwrap_or(self.text.len());
        let end = self
            .line_starts
            .get(row + 1)
            .map_or(self.text.len(), |next| next - 1);
        self.text.get(start..end.max(start)).unwrap_or_default()
    }

    /// The path ids a source line belongs to. Empty for blank lines.
    pub(crate) fn row_paths(&self, function: &Function, row: usize) -> Vec<usize> {
        let Some(offset) = self.row_offsets.get(row).copied().flatten() else {
            return Vec::new();
        };
        let innermost = function
            .arms
            .iter()
            .enumerate()
            .filter(|(_, arm)| arm.start_byte <= offset && offset < arm.end_byte)
            .min_by_key(|(_, arm)| arm.end_byte - arm.start_byte)
            .map(|(id, _)| id);
        let starting_here = function
            .arms
            .iter()
            .enumerate()
            .filter(|(_, arm)| arm.start_row == row)
            .map(|(id, _)| id);

        let mut paths: Vec<usize> = innermost.into_iter().chain(starting_here).collect();
        paths.sort_unstable();
        paths.dedup();
        paths
    }
}

/// Rewrites what the C grammar cannot read, keeping every byte and newline in place so rows
/// and offsets stay aligned with the `LineMap`. Linemarkers and pragmas become spaces: the
/// grammar rejects them inside expressions. GCC's `__builtin_offsetof`, which every option
/// table uses, becomes the standard `offsetof`.
fn prepare_for_parser(preprocessed: &str) -> String {
    let mut out = String::with_capacity(preprocessed.len());
    for line in preprocessed.split_inclusive('\n') {
        if line.trim_start().starts_with('#') {
            let content = line.trim_end_matches('\n');
            out.extend(std::iter::repeat_n(' ', content.len()));
            if line.ends_with('\n') {
                out.push('\n');
            }
        } else if line.contains("__builtin_offsetof") {
            out.push_str(&line.replace("__builtin_offsetof", "offsetof          "));
        } else {
            out.push_str(line);
        }
    }
    out
}

fn extract(
    path: &str,
    parsed: &Parsed<'_>,
    original: &str,
) -> Result<(Vec<Function>, usize, Vec<Option<usize>>)> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_c::LANGUAGE.into())
        .context("loading the C grammar")?;
    let tree = parser
        .parse(parsed.text, None)
        .with_context(|| format!("parsing {path}"))?;

    let lines: Vec<&str> = original.lines().collect();
    let mut functions = Vec::new();
    collect_functions(tree.root_node(), parsed, &lines, &mut functions);
    number_duplicates(&mut functions);
    let mut unparsed = BTreeSet::new();
    unparsed_rows(tree.root_node(), parsed, &mut unparsed);
    Ok((functions, unparsed.len(), row_offsets(parsed, &lines)))
}

/// For each original row, the first parsed byte that came from it. A row with nothing left after
/// preprocessing (a comment, a directive, a macro argument continued from the row above) stands
/// at the end of the code before it, so it belongs to the same block as that code.
fn row_offsets(parsed: &Parsed<'_>, lines: &[&str]) -> Vec<Option<usize>> {
    let mut first = vec![None; lines.len()];
    let mut last_end = vec![None; lines.len()];
    let mut start = 0;
    for (row, line) in parsed.text.split('\n').enumerate() {
        let origin = parsed.origins.get(row).copied().flatten();
        let indent = line.find(|c: char| !c.is_whitespace());
        if let Some((origin, indent)) = origin.zip(indent) {
            if let Some(first) = first.get_mut(origin) {
                first.get_or_insert(start + indent);
            }
            if let Some(last_end) = last_end.get_mut(origin) {
                *last_end = Some(start + line.trim_end().len());
            }
        }
        start += line.len() + 1;
    }
    let mut carry = None;
    lines
        .iter()
        .enumerate()
        .map(|(row, line)| {
            let direct = first.get(row).copied().flatten();
            let offset = direct.or_else(|| carry.filter(|_| !line.trim().is_empty()));
            carry = last_end.get(row).copied().flatten().or(carry);
            offset
        })
        .collect()
}

fn unparsed_rows(node: Node<'_>, parsed: &Parsed<'_>, rows: &mut BTreeSet<usize>) {
    if node.is_error() {
        let range = node.start_position().row..=node.end_position().row;
        rows.extend(range.filter_map(|row| parsed.origins.get(row).copied().flatten()));
        return;
    }
    if !node.has_error() || node.kind() == "function_definition" {
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        unparsed_rows(child, parsed, rows);
    }
}

fn collect_functions(
    node: Node<'_>,
    parsed: &Parsed<'_>,
    lines: &[&str],
    functions: &mut Vec<Function>,
) {
    if node.kind() == "function_definition" {
        functions.extend(function(node, parsed, lines));
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_functions(child, parsed, lines, functions);
    }
}

fn function(node: Node<'_>, parsed: &Parsed<'_>, lines: &[&str]) -> Option<Function> {
    let source = parsed.text.as_bytes();
    // Functions from included headers are counted when the header itself is parsed.
    let start_row = parsed
        .origins
        .get(node.start_position().row)
        .copied()
        .flatten()?;
    let end_row = parsed
        .row(node.end_position().row)
        .unwrap_or(start_row)
        .max(start_row);
    let name = declared_name(node, source)?;
    let lines = lines.get(start_row..=end_row)?;
    let normalized = lines
        .iter()
        .flat_map(|line| line.split_whitespace())
        .collect::<Vec<_>>()
        .join(" ");
    let mut hash = String::new();
    for byte in Sha256::digest(normalized.as_bytes()).iter().take(6) {
        write!(hash, "{byte:02x}").ok()?;
    }

    let mut branches = Vec::new();
    collect_arms(node, source, &mut branches);
    branches.sort_by_key(|arm| (arm.start_byte, std::cmp::Reverse(arm.end_byte)));
    let mut arms = vec![arm(node, "body")];
    arms.append(&mut branches);
    for arm in &mut arms {
        arm.start_row = parsed.row(arm.start_row).unwrap_or(start_row);
    }

    Some(Function {
        name,
        hash,
        start_row,
        end_row,
        arms,
    })
}

fn declared_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    let mut declarator = node.child_by_field_name("declarator")?;
    while declarator.kind() != "identifier" {
        declarator = declarator
            .child_by_field_name("declarator")
            .or_else(|| declarator.named_child(0))?;
    }
    declarator.utf8_text(source).ok().map(str::to_owned)
}

/// An arm whose `start_row` is still a row of the parsed text; `function` maps it.
fn arm(node: Node<'_>, kind: &'static str) -> Arm {
    Arm {
        kind,
        start_row: node.start_position().row,
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
    }
}

fn collect_arms(node: Node<'_>, source: &[u8], arms: &mut Vec<Arm>) {
    let field = |name| node.child_by_field_name(name);
    match node.kind() {
        "if_statement" => {
            arms.extend(field("consequence").map(|then| arm(then, "if")));
            // `else if` is the nested statement's own `if` arm.
            let otherwise = field("alternative")
                .and_then(|alternative| {
                    let mut cursor = alternative.walk();
                    alternative.named_children(&mut cursor).last()
                })
                .filter(|body| body.kind() != "if_statement");
            arms.extend(otherwise.map(|body| arm(body, "else")));
        }
        "case_statement" => arms.push(arm(node, "case")),
        "for_statement" | "while_statement" => {
            arms.extend(field("body").map(|body| arm(body, "loop")));
        }
        // Macros wrap statements in `do { } while (0)`, whose body runs exactly once.
        "do_statement" => {
            let once = field("condition")
                .and_then(|condition| condition.utf8_text(source).ok())
                .is_some_and(|condition| condition.split_whitespace().collect::<String>() == "(0)");
            if !once {
                arms.extend(field("body").map(|body| arm(body, "loop")));
            }
        }
        "conditional_expression" => {
            arms.extend(field("consequence").map(|then| arm(then, "?")));
            arms.extend(field("alternative").map(|otherwise| arm(otherwise, ":")));
        }
        "binary_expression" => {
            let kind = match field("operator").map(|operator| operator.kind()) {
                Some("&&") => Some("&&"),
                Some("||") => Some("||"),
                _ => None,
            };
            if let Some((kind, right)) = kind.zip(field("right")) {
                arms.push(arm(right, kind));
            }
        }
        // The operand is never evaluated, so its branches are not paths.
        "sizeof_expression" | "alignof_expression" => return,
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_arms(child, source, arms);
    }
}

/// `#if` variants can define one name twice in a file; later ones become `name#2`, `name#3`.
fn number_duplicates(functions: &mut [Function]) {
    let mut seen = std::collections::BTreeMap::<String, usize>::new();
    for function in functions {
        let count = seen.entry(function.name.clone()).or_default();
        *count += 1;
        if *count > 1 {
            function.name = format!("{}#{count}", function.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preprocess::Compile;

    #[test]
    fn short_circuit_operands_are_paths() {
        let text = "int f(int a, int b) { if (a && b) return 1; return a || b ? 2 : 3; }\n";
        let file = SourceFile::parse("f.c".to_owned(), text.to_owned()).expect("parses");
        let kinds: Vec<&str> = file
            .functions
            .first()
            .expect("one function")
            .arms
            .iter()
            .map(|arm| arm.kind)
            .collect();
        assert_eq!(kinds, ["body", "&&", "if", "||", "?", ":"]);
    }

    fn kinds(function: &Function) -> Vec<&'static str> {
        function.arms.iter().map(|arm| arm.kind).collect()
    }

    #[test]
    fn unevaluated_and_run_once_code_is_not_a_branch() {
        let text = "int f(int a) { do { a++; } while (0); do a--; while (a); return sizeof(a ? 1 : 2); }\n";
        let file = SourceFile::parse("f.c".to_owned(), text.to_owned()).expect("parses");
        assert_eq!(
            kinds(file.functions.first().expect("one function")),
            ["body", "loop"]
        );
    }

    #[test]
    fn macro_branches_count_on_the_invocation_line() {
        let dir = std::env::temp_dir().join(format!("mpv-cover-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let header = "#define PICK(a, b) ((a) > (b) ? (a) : (b))\nstatic inline int g(int a) { return a ? 1 : 0; }\n";
        let text = "int f(int a, int b) {\n    return PICK(a, b);\n}\n";
        std::fs::write(dir.join("pick.h"), header).expect("write header");
        std::fs::write(dir.join("f.c"), text).expect("write source");
        let arguments = "cc -include pick.h -MD -MQ f.o -MF f.o.d -o f.o -c f.c";
        let compile = Compile::new(
            dir.clone(),
            arguments.split(' ').map(str::to_owned).collect(),
        );
        let preprocessed = compile.preprocess();
        std::fs::remove_dir_all(&dir).expect("remove temp dir");

        let file = SourceFile::parse_preprocessed(
            "f.c".to_owned(),
            text.to_owned(),
            &preprocessed.expect("cc -E runs"),
        )
        .expect("parses");
        let names: Vec<&str> = file
            .functions
            .iter()
            .map(|function| function.name.as_str())
            .collect();
        assert_eq!(names, ["f"]);
        let function = file.functions.first().expect("one function");
        assert_eq!(kinds(function), ["body", "?", ":"]);
        let rows: Vec<usize> = function
            .arms
            .iter()
            .skip(1)
            .map(|arm| arm.start_row)
            .collect();
        assert_eq!(rows, [1, 1]);
        assert_eq!((function.start_row, function.end_row), (0, 2));

        let raw = SourceFile::parse("f.c".to_owned(), text.to_owned()).expect("parses");
        assert_eq!(
            raw.functions.first().map(|raw| &raw.hash),
            Some(&function.hash)
        );
    }
}
