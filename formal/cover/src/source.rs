//! Extracts functions and their branch arms from mpv's C sources.

use anyhow::{Context as _, Result};
use sha2::{Digest as _, Sha256};
use std::fmt::Write as _;
use tree_sitter::{Node, Parser};

/// One path through a function: the straight-line body (path 0) or a branch arm.
#[derive(Debug)]
pub(crate) struct Arm {
    pub(crate) kind: &'static str,
    pub(crate) start_row: usize,
    start_byte: usize,
    end_byte: usize,
}

#[derive(Debug)]
pub(crate) struct Function {
    pub(crate) name: String,
    /// Digest of the whitespace-normalized source, so a claim goes stale when mpv changes.
    pub(crate) hash: String,
    pub(crate) start_row: usize,
    pub(crate) end_row: usize,
    /// Index is the path id. Path 0 spans the whole function.
    pub(crate) arms: Vec<Arm>,
}

#[derive(Debug)]
pub(crate) struct SourceFile {
    pub(crate) path: String,
    pub(crate) functions: Vec<Function>,
    pub(crate) code_lines: usize,
    pub(crate) function_lines: usize,
    /// Lines in regions the parser gave up on. Functions in them are invisible to the tool.
    pub(crate) unparsed_lines: usize,
    text: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    pub(crate) fn parse(path: String, text: String) -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .context("loading the C grammar")?;
        let tree = parser
            .parse(&text, None)
            .with_context(|| format!("parsing {path}"))?;

        let mut functions = Vec::new();
        collect_functions(tree.root_node(), text.as_bytes(), &mut functions);
        number_duplicates(&mut functions);

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

        Ok(Self {
            path,
            functions,
            code_lines,
            function_lines,
            unparsed_lines: unparsed_lines(tree.root_node()),
            text,
            line_starts,
        })
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
        let line = self.line(row);
        let Some(indent) = line.find(|c: char| !c.is_whitespace()) else {
            return Vec::new();
        };
        let offset = self.line_starts.get(row).copied().unwrap_or_default() + indent;

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

fn unparsed_lines(node: Node<'_>) -> usize {
    if node.is_error() {
        return node.end_position().row - node.start_position().row + 1;
    }
    if !node.has_error() || node.kind() == "function_definition" {
        return 0;
    }
    let mut cursor = node.walk();
    node.children(&mut cursor).map(unparsed_lines).sum()
}

fn collect_functions(node: Node<'_>, source: &[u8], functions: &mut Vec<Function>) {
    if node.kind() == "function_definition" {
        functions.extend(function(node, source));
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_functions(child, source, functions);
    }
}

fn function(node: Node<'_>, source: &[u8]) -> Option<Function> {
    let name = declared_name(node, source)?;
    let text = node.utf8_text(source).ok()?;
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut hash = String::new();
    for byte in Sha256::digest(normalized.as_bytes()).iter().take(6) {
        write!(hash, "{byte:02x}").ok()?;
    }

    let mut branches = Vec::new();
    collect_arms(node, &mut branches);
    branches.sort_by_key(|arm| (arm.start_byte, std::cmp::Reverse(arm.end_byte)));
    let mut arms = vec![arm(node, "body")];
    arms.append(&mut branches);

    Some(Function {
        name,
        hash,
        start_row: node.start_position().row,
        end_row: node.end_position().row,
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

fn arm(node: Node<'_>, kind: &'static str) -> Arm {
    Arm {
        kind,
        start_row: node.start_position().row,
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
    }
}

fn collect_arms(node: Node<'_>, arms: &mut Vec<Arm>) {
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
        "for_statement" | "while_statement" | "do_statement" => {
            arms.extend(field("body").map(|body| arm(body, "loop")));
        }
        "conditional_expression" => {
            arms.extend(field("consequence").map(|then| arm(then, "?")));
            arms.extend(field("alternative").map(|otherwise| arm(otherwise, ":")));
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_arms(child, arms);
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
