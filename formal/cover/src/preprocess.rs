//! Runs mpv's C files through the C preprocessor with the flags of the real build.

use anyhow::{Context as _, Result, bail, ensure};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Flags that make the compiler write files instead of preprocessed text on stdout.
const OUTPUT_FLAGS: [&str; 4] = ["-o", "-MF", "-MQ", "-MT"];
const MODE_FLAGS: [&str; 6] = ["-c", "-M", "-MM", "-MD", "-MMD", "-MP"];

/// How the real build compiles one file, turned into a preprocessor run.
#[derive(Debug)]
pub(crate) struct Compile {
    directory: PathBuf,
    arguments: Vec<String>,
}

impl Compile {
    pub(crate) fn new(directory: PathBuf, arguments: Vec<String>) -> Self {
        let mut kept = Vec::with_capacity(arguments.len() + 1);
        let mut arguments = arguments.into_iter();
        while let Some(argument) = arguments.next() {
            if OUTPUT_FLAGS.contains(&argument.as_str()) {
                arguments.next();
            } else if !MODE_FLAGS.contains(&argument.as_str()) {
                kept.push(argument);
            }
        }
        kept.push("-E".to_owned());
        Self {
            directory,
            arguments: kept,
        }
    }

    pub(crate) fn preprocess(&self) -> Result<String> {
        let (program, arguments) = self
            .arguments
            .split_first()
            .context("empty compile command")?;
        let output = Command::new(program)
            .args(arguments)
            .current_dir(&self.directory)
            .output()
            .with_context(|| format!("running {program} in {}", self.directory.display()))?;
        if !output.status.success() {
            bail!(
                "`{}` failed in {}:\n{}",
                self.arguments.join(" "),
                self.directory.display(),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

#[derive(Debug, Deserialize)]
struct Entry {
    directory: PathBuf,
    file: PathBuf,
    command: Option<String>,
    arguments: Option<Vec<String>>,
}

/// `compile_commands.json`, keyed by path relative to the mpv checkout.
#[derive(Debug)]
pub(crate) struct Database {
    entries: BTreeMap<String, Compile>,
}

impl Database {
    pub(crate) fn load(build: &Path, mpv: &Path) -> Result<Self> {
        let path = build.join("compile_commands.json");
        let Ok(text) = fs::read_to_string(&path) else {
            bail!(
                "{} is missing; run `make mpv-build` to build mpv first",
                path.display()
            );
        };
        let entries: Vec<Entry> =
            serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        let mpv = fs::canonicalize(mpv).with_context(|| format!("resolving {}", mpv.display()))?;

        let mut database = Self {
            entries: BTreeMap::new(),
        };
        for entry in entries {
            let file = entry.directory.join(&entry.file);
            let file =
                fs::canonicalize(&file).with_context(|| format!("resolving {}", file.display()))?;
            let Ok(relative) = file.strip_prefix(&mpv) else {
                continue;
            };
            let arguments = match (entry.arguments, entry.command) {
                (Some(arguments), _) => arguments,
                (None, Some(command)) => split_command(&command)
                    .with_context(|| format!("compile command for {}", file.display()))?,
                (None, None) => bail!("{} has neither `command` nor `arguments`", file.display()),
            };
            database.entries.insert(
                relative.to_string_lossy().replace('\\', "/"),
                Compile::new(entry.directory, arguments),
            );
        }
        // A database for another checkout would silently turn every file into a raw parse.
        ensure!(
            !database.entries.is_empty(),
            "{} has no entry for a file in {}; run `make mpv-build` for this checkout",
            path.display(),
            mpv.display()
        );
        Ok(database)
    }

    pub(crate) fn get(&self, path: &str) -> Option<&Compile> {
        self.entries.get(path)
    }
}

/// Splits a shell command line the way `sh` would, for the quoting compilers' databases use.
fn split_command(command: &str) -> Result<Vec<String>> {
    let (mut words, mut word, mut in_word) = (Vec::new(), String::new(), false);
    let mut chars = command.chars();
    while let Some(c) = chars.next() {
        match c {
            c if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut word));
                }
                in_word = false;
                continue;
            }
            '\\' => word.extend(chars.next()),
            '\'' => loop {
                match chars.next() {
                    Some('\'') => break,
                    Some(c) => word.push(c),
                    None => bail!("unclosed ' in `{command}`"),
                }
            },
            '"' => loop {
                match chars.next() {
                    Some('"') => break,
                    Some('\\') => match chars.next() {
                        Some(c @ ('"' | '\\' | '$' | '`')) => word.push(c),
                        Some(c) => word.extend(['\\', c]),
                        None => bail!("unclosed \" in `{command}`"),
                    },
                    Some(c) => word.push(c),
                    None => bail!("unclosed \" in `{command}`"),
                }
            },
            c => word.push(c),
        }
        in_word = true;
    }
    if in_word {
        words.push(word);
    }
    Ok(words)
}

/// Maps rows of preprocessed text back to the original file and line, from GCC linemarkers.
#[derive(Debug, Default)]
pub(crate) struct LineMap {
    files: Vec<String>,
    /// Per row: index into `files` and one-based line. `None` for the linemarkers themselves.
    rows: Vec<Option<(usize, usize)>>,
}

impl LineMap {
    pub(crate) fn parse(preprocessed: &str) -> Self {
        let mut map = Self::default();
        let mut current: Option<(usize, usize)> = None;
        for line in preprocessed.lines() {
            if let Some((number, name)) = linemarker(line) {
                let index = map.files.iter().position(|file| *file == name);
                let index = index.unwrap_or_else(|| {
                    map.files.push(name);
                    map.files.len() - 1
                });
                current = Some((index, number));
                map.rows.push(None);
            } else {
                map.rows.push(current);
                current = current.map(|(file, number)| (file, number + 1));
            }
        }
        map
    }

    /// The original file and zero-based line of a row of preprocessed text.
    pub(crate) fn origin(&self, row: usize) -> Option<(&str, usize)> {
        let (file, number) = (*self.rows.get(row)?)?;
        Some((self.files.get(file)?.as_str(), number.checked_sub(1)?))
    }

    /// The file the compiler was asked to preprocess: GCC names it in the first linemarker.
    pub(crate) fn main_file(&self) -> Option<&str> {
        self.files.first().map(String::as_str)
    }
}

/// Parses `# 12 "file.c" 2`. Other `#` lines left by the preprocessor are pragmas.
fn linemarker(line: &str) -> Option<(usize, String)> {
    let rest = line.strip_prefix('#')?.trim_start();
    let rest = rest.strip_prefix("line").unwrap_or(rest).trim_start();
    let (number, rest) = rest.split_once(char::is_whitespace)?;
    let number = number.parse().ok()?;
    let mut chars = rest.trim_start().strip_prefix('"')?.chars();
    let mut name = String::new();
    loop {
        match chars.next()? {
            '"' => return Some((number, name)),
            '\\' => name.push(chars.next()?),
            c => name.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linemarkers_map_rows_to_original_lines() {
        let map =
            LineMap::parse("# 1 \"a.c\"\nint x;\n# 10 \"b.h\" 1\nint y;\n# 3 \"a.c\" 2\nint z;\n");
        assert_eq!(map.origin(1), Some(("a.c", 0)));
        assert_eq!(map.origin(3), Some(("b.h", 9)));
        assert_eq!(map.origin(5), Some(("a.c", 2)));
    }

    #[test]
    fn commands_split_like_a_shell() {
        let words = split_command(r#"cc '-DA=b c' -DQ=\"x\" "-I a\\b" -c f.c"#).expect("splits");
        assert_eq!(
            words,
            ["cc", "-DA=b c", "-DQ=\"x\"", "-I a\\b", "-c", "f.c"]
        );
    }
}
