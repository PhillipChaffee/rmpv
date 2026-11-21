//! Reads `-- mpv-cover:` claims out of the Lean sources and scans them for proof escape hatches.

use anyhow::{Context as _, Result};
use std::{collections::BTreeSet, fs, path::Path};
use walkdir::WalkDir;

pub(crate) const MARKER: &str = "-- mpv-cover:";
const DECLARATIONS: [&str; 8] = [
    "def",
    "theorem",
    "abbrev",
    "structure",
    "inductive",
    "instance",
    "lemma",
    "opaque",
];
const MODIFIERS: [&str; 3] = ["private", "protected", "noncomputable"];
/// Words that let a Lean file claim a result without a kernel-checked proof.
const ESCAPE_HATCHES: [&str; 7] = [
    "sorry",
    "admit",
    "axiom",
    "native_decide",
    "implemented_by",
    "extern",
    "unsafe",
];

#[derive(Debug)]
pub(crate) enum Paths {
    All,
    Listed(BTreeSet<usize>),
}

/// A statement in a Lean file that a declaration models part of an mpv function.
#[derive(Debug)]
pub(crate) struct Claim {
    pub(crate) file: String,
    pub(crate) function: String,
    pub(crate) paths: Paths,
    pub(crate) hash: String,
    pub(crate) declaration: String,
    /// `file:line` of the claim, for error messages.
    pub(crate) origin: String,
}

#[derive(Debug, Default)]
pub(crate) struct Scan {
    pub(crate) claims: Vec<Claim>,
    pub(crate) problems: Vec<String>,
}

pub(crate) fn scan(formal: &Path) -> Result<Scan> {
    let mut scan = Scan::default();
    let lean_files = WalkDir::new(formal)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| entry.file_name() != ".lake" && entry.file_name() != ".mpv")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "lean")
        });
    for entry in lean_files {
        let text = fs::read_to_string(entry.path())
            .with_context(|| format!("reading {}", entry.path().display()))?;
        let name = entry
            .path()
            .strip_prefix(formal)
            .unwrap_or_else(|_| entry.path())
            .display()
            .to_string();
        scan_file(&name, &text, &mut scan);
    }
    Ok(scan)
}

fn scan_file(name: &str, text: &str, scan: &mut Scan) {
    let lines: Vec<&str> = text.lines().collect();
    for (row, line) in lines.iter().enumerate() {
        let Some(body) = line.trim().strip_prefix(MARKER) else {
            continue;
        };
        let origin = format!("{name}:{}", row + 1);
        let declaration = lines
            .iter()
            .skip(row + 1)
            .find(|next| !is_skippable(next))
            .and_then(|next| declaration_name(next));
        match (parse_claim(body), declaration) {
            (Ok((file, function, paths, hash)), Some(declaration)) => {
                scan.claims.push(Claim {
                    file,
                    function,
                    paths,
                    hash,
                    declaration,
                    origin,
                });
            }
            (Err(problem), _) => scan.problems.push(format!("{origin}: {problem}")),
            (Ok(_), _) => scan
                .problems
                .push(format!("{origin}: claim is not followed by a declaration")),
        }
    }
    for (row, word) in escape_hatches(text) {
        scan.problems.push(format!(
            "{name}:{}: `{word}` bypasses the proof checker",
            row + 1
        ));
    }
}

/// Lines allowed between a claim and the declaration it annotates.
fn is_skippable(line: &str) -> bool {
    let line = line.trim();
    line.is_empty() || line.starts_with("--") || line.starts_with("@[")
}

fn declaration_name(line: &str) -> Option<String> {
    let mut words = line
        .split_whitespace()
        .skip_while(|word| MODIFIERS.contains(word));
    let keyword = words.next()?;
    words
        .next()
        .filter(|_| DECLARATIONS.contains(&keyword))
        .map(str::to_owned)
}

fn parse_claim(body: &str) -> Result<(String, String, Paths, String), String> {
    let mut words = body.split_whitespace();
    let file = words.next().ok_or("missing the mpv file")?;
    let function = words.next().ok_or("missing the mpv function")?;
    let (mut paths, mut hash) = (None, None);
    for word in words {
        match word.split_once('=') {
            Some(("paths", "all")) => paths = Some(Paths::All),
            Some(("paths", ids)) => {
                let ids: Result<BTreeSet<usize>, _> = ids.split(',').map(str::parse).collect();
                paths = Some(Paths::Listed(
                    ids.map_err(|_| format!("bad path list `{word}`"))?,
                ));
            }
            Some(("hash", value)) => hash = Some(value),
            _ => return Err(format!("unknown field `{word}`")),
        }
    }
    Ok((
        file.to_owned(),
        function.to_owned(),
        paths.ok_or("missing `paths=`")?,
        hash.ok_or("missing `hash=`")?.to_owned(),
    ))
}

/// Finds escape-hatch words outside comments and string literals.
fn escape_hatches(text: &str) -> Vec<(usize, &'static str)> {
    let mut found = Vec::new();
    let (mut row, mut comment_depth, mut in_string, mut word) = (0, 0_usize, false, String::new());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let next = chars.peek().copied();
        let is_word_char = c.is_alphanumeric() || c == '_';
        if comment_depth == 0 && !in_string && is_word_char {
            word.push(c);
            continue;
        }
        found.extend(
            ESCAPE_HATCHES
                .iter()
                .find(|hatch| **hatch == word)
                .map(|hatch| (row, *hatch)),
        );
        word.clear();
        match (c, next) {
            ('\n', _) => row += 1,
            ('\\', _) if in_string => {
                chars.next();
            }
            ('"', _) if comment_depth == 0 => in_string = !in_string,
            _ if in_string => {}
            ('/', Some('-')) => {
                comment_depth += 1;
                chars.next();
            }
            ('-', Some('/')) if comment_depth > 0 => {
                comment_depth -= 1;
                chars.next();
            }
            ('-', Some('-')) if comment_depth == 0 => {
                while chars.next_if(|c| *c != '\n').is_some() {}
            }
            _ => {}
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problems(text: &str) -> Vec<String> {
        let mut scan = Scan::default();
        scan_file("M.lean", text, &mut scan);
        scan.problems
    }

    #[test]
    fn claim_binds_to_the_next_declaration() {
        let mut scan = Scan::default();
        scan_file(
            "M.lean",
            "-- mpv-cover: a/b.c f paths=0,2 hash=abc\n@[simp]\ndef model := 1\n",
            &mut scan,
        );
        let claim = scan.claims.first().expect("one claim");
        assert_eq!(
            (claim.file.as_str(), claim.function.as_str()),
            ("a/b.c", "f")
        );
        assert_eq!(claim.declaration, "model");
        assert!(matches!(&claim.paths, Paths::Listed(ids) if ids.iter().copied().eq([0, 2])));
    }

    #[test]
    fn claim_without_hash_or_declaration_is_a_problem() {
        assert_eq!(
            problems("-- mpv-cover: a.c f paths=all\ndef m := 1\n").len(),
            1
        );
        assert_eq!(
            problems("-- mpv-cover: a.c f paths=all hash=abc\n\n#eval 1\n").len(),
            1
        );
    }

    #[test]
    fn sorry_is_flagged_only_in_code() {
        assert_eq!(problems("theorem t : True := by sorry\n").len(), 1);
        assert!(problems("-- sorry\n/- sorry /- axiom -/ -/\ndef s := \"sorry\"\n").is_empty());
    }
}
