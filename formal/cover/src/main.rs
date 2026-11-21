//! Reports which mpv code paths the Lean model in `formal/` accounts for.

mod claims;
mod source;

use anyhow::{Context as _, Result, bail, ensure};
use claims::{Claim, Paths};
use clap::{Parser, Subcommand};
use globset::Glob;
use serde::Deserialize;
use source::{Function, SourceFile};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};
use walkdir::WalkDir;

/// Languages mpv ships behaviour in that this tool cannot parse yet.
const UNTRACKED_LANGUAGES: [&str; 4] = ["lua", "js", "m", "swift"];

#[derive(Debug, Parser)]
#[command(about)]
struct Cli {
    /// Directory holding the Lean model, `mpv.lock.toml` and `scope.toml`.
    #[arg(long, default_value = "formal")]
    formal: PathBuf,
    /// mpv checkout at the locked commit. Defaults to `<formal>/.mpv`.
    #[arg(long)]
    mpv: Option<PathBuf>,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Coverage totals per mpv directory.
    Report,
    /// mpv source as a diff: `-` lines are paths the model does not account for.
    Diff {
        /// mpv files or directories. Defaults to files with at least one claim.
        paths: Vec<String>,
        /// Hide functions that are fully covered.
        #[arg(long)]
        uncovered: bool,
    },
    /// One function with its path ids and hash, for writing a claim: `FILE FUNCTION`.
    Show { file: String, function: String },
    /// Fail on invalid or stale claims. With `--full`, also fail on any uncovered path.
    Check {
        #[arg(long)]
        full: bool,
    },
}

#[derive(Debug, Deserialize)]
struct Lock {
    commit: String,
}

#[derive(Debug, Deserialize)]
struct Scope {
    #[serde(default)]
    exclude: Vec<Exclusion>,
}

#[derive(Debug, Deserialize)]
struct Exclusion {
    glob: String,
    reason: String,
}

/// What the model claims about one mpv function.
#[derive(Debug, Default)]
struct Coverage {
    paths: BTreeSet<usize>,
    declarations: BTreeSet<String>,
    stale: bool,
}

#[derive(Debug, Default)]
struct Analysis {
    commit: String,
    files: Vec<SourceFile>,
    coverage: BTreeMap<(String, String), Coverage>,
    /// Glob, reason, number of files.
    excluded: Vec<(String, String, usize)>,
    untracked_lines: BTreeMap<&'static str, usize>,
    problems: Vec<String>,
    /// File and function pairs with a planted-bug check in `fidelity/targets`.
    checked: BTreeSet<(String, String)>,
}

#[derive(Debug, Default, Clone, Copy)]
struct Tally {
    functions: usize,
    functions_covered: usize,
    paths: usize,
    paths_covered: usize,
    /// Covered paths in functions the fidelity check runs against real mpv.
    paths_checked: usize,
}

impl Tally {
    fn add(&mut self, function: &Function, covered: usize, checked: bool) {
        self.functions += 1;
        self.functions_covered += usize::from(covered == function.arms.len());
        self.paths += function.arms.len();
        self.paths_covered += covered;
        self.paths_checked += if checked { covered } else { 0 };
    }

    fn row(&self, label: &str) -> String {
        #[expect(clippy::cast_precision_loss, reason = "path counts are far below 2^52")]
        let percent = 100.0 * self.paths_covered as f64 / self.paths.max(1) as f64;
        format!(
            "{label:<12} {:>6} / {:<6} {:>7} / {:<7} {percent:>7.3}% {:>8}\n",
            self.functions_covered,
            self.functions,
            self.paths_covered,
            self.paths,
            self.paths_checked
        )
    }
}

impl Analysis {
    fn load(formal: &Path, mpv: &Path) -> Result<Self> {
        let lock: Lock = read_toml(&formal.join("mpv.lock.toml"))?;
        let scope: Scope = read_toml(&formal.join("scope.toml"))?;
        let head = Command::new("git")
            .arg("-C")
            .arg(mpv)
            .args(["rev-parse", "HEAD"])
            .output();
        let head = head.with_context(|| format!("running git in {}", mpv.display()))?;
        let head = String::from_utf8_lossy(&head.stdout).trim().to_owned();
        ensure!(
            head == lock.commit,
            "{} is at `{head}` but mpv.lock.toml pins {}",
            mpv.display(),
            lock.commit
        );

        let mut analysis = Self {
            commit: lock.commit,
            ..Self::default()
        };
        let mut matchers = Vec::new();
        for exclusion in scope.exclude {
            let glob =
                Glob::new(&exclusion.glob).with_context(|| format!("glob {}", exclusion.glob))?;
            matchers.push(glob.compile_matcher());
            analysis
                .excluded
                .push((exclusion.glob, exclusion.reason, 0));
        }

        let entries = WalkDir::new(mpv)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|entry| entry.file_name() != ".git")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file());
        for entry in entries {
            let path = entry
                .path()
                .strip_prefix(mpv)?
                .to_string_lossy()
                .replace('\\', "/");
            let extension = entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();
            if let Some(index) = matchers.iter().position(|matcher| matcher.is_match(&path)) {
                if let Some(excluded) = analysis.excluded.get_mut(index) {
                    excluded.2 += 1;
                }
            } else if extension == "c" || extension == "h" {
                let text = String::from_utf8_lossy(&fs::read(entry.path())?).into_owned();
                analysis.files.push(SourceFile::parse(path, text)?);
            } else if let Some(language) = UNTRACKED_LANGUAGES.iter().find(|l| **l == extension) {
                let lines = String::from_utf8_lossy(&fs::read(entry.path())?)
                    .lines()
                    .count();
                *analysis.untracked_lines.entry(language).or_default() += lines;
            }
        }

        analysis.checked = checked_functions(formal)?;
        let scan = claims::scan(formal)?;
        analysis.problems = scan.problems;
        for claim in scan.claims {
            if let Err(problem) = analysis.apply(&claim) {
                analysis
                    .problems
                    .push(format!("{}: {problem}", claim.origin));
            }
        }
        Ok(analysis)
    }

    fn apply(&mut self, claim: &Claim) -> Result<()> {
        let function = self.function(&claim.file, &claim.function)?;
        let (hash, path_count) = (function.hash.clone(), function.arms.len());
        let coverage = self
            .coverage
            .entry((claim.file.clone(), claim.function.clone()))
            .or_default();
        coverage.declarations.insert(claim.declaration.clone());
        if claim.hash != hash {
            coverage.stale = true;
            bail!(
                "stale: `{}` now hashes to {hash}, the claim says {}",
                claim.function,
                claim.hash
            );
        }
        match &claim.paths {
            Paths::All => coverage.paths.extend(0..path_count),
            Paths::Listed(ids) => {
                if let Some(id) = ids.iter().find(|id| **id >= path_count) {
                    bail!("`{}` has paths 0..{path_count}, not {id}", claim.function);
                }
                coverage.paths.extend(ids);
            }
        }
        Ok(())
    }

    fn function(&self, file: &str, name: &str) -> Result<&Function> {
        let source = self.files.iter().find(|source| source.path == file);
        let source = source.with_context(|| format!("no in-scope mpv file `{file}`"))?;
        let function = source
            .functions
            .iter()
            .find(|function| function.name == name);
        function.with_context(|| format!("no function `{name}` in {file}"))
    }

    fn covered_paths(&self, file: &str, function: &Function) -> BTreeSet<usize> {
        self.coverage
            .get(&(file.to_owned(), function.name.clone()))
            .filter(|coverage| !coverage.stale)
            .map(|coverage| coverage.paths.clone())
            .unwrap_or_default()
    }

    fn is_checked(&self, file: &str, function: &Function) -> bool {
        self.checked
            .contains(&(file.to_owned(), function.name.clone()))
    }

    fn total(&self) -> Tally {
        let mut total = Tally::default();
        for file in &self.files {
            for function in &file.functions {
                let covered = self.covered_paths(&file.path, function).len();
                total.add(function, covered, self.is_checked(&file.path, function));
            }
        }
        total
    }

    fn report(&self) -> String {
        let mut by_directory = BTreeMap::<&str, Tally>::new();
        for file in &self.files {
            let directory = file
                .path
                .split_once('/')
                .map_or(".", |(directory, _)| directory);
            let tally = by_directory.entry(directory).or_default();
            for function in &file.functions {
                let covered = self.covered_paths(&file.path, function).len();
                tally.add(function, covered, self.is_checked(&file.path, function));
            }
        }

        let mut out = format!("mpv @ {}\n\n", self.commit);
        let _ = writeln!(
            out,
            "{:<12} {:>15} {:>17} {:>8} {:>8}",
            "directory", "functions", "paths", "covered", "checked"
        );
        for (directory, tally) in &by_directory {
            out.push_str(&tally.row(directory));
        }
        out.push_str(&self.total().row("TOTAL"));

        let code: usize = self.files.iter().map(|file| file.code_lines).sum();
        let in_functions: usize = self.files.iter().map(|file| file.function_lines).sum();
        let unparsed: usize = self.files.iter().map(|file| file.unparsed_lines).sum();
        out.push_str("\nNot measured by the path counts above:\n");
        let _ = writeln!(
            out,
            "  {} of {code} C lines sit outside any function (tables, macros, declarations)",
            code - in_functions
        );
        let _ = writeln!(
            out,
            "  {unparsed} C lines outside functions are in regions the parser could not read"
        );
        for (language, lines) in &self.untracked_lines {
            let _ = writeln!(out, "  {lines} lines of .{language} are not parsed");
        }
        for (glob, reason, files) in &self.excluded {
            let _ = writeln!(out, "  {files} files excluded by `{glob}`: {reason}");
        }
        out.push_str(&self.problem_list());
        out
    }

    fn problem_list(&self) -> String {
        let mut out = String::new();
        if !self.problems.is_empty() {
            let _ = writeln!(out, "\n{} problem(s):", self.problems.len());
        }
        for problem in &self.problems {
            let _ = writeln!(out, "  {problem}");
        }
        out
    }

    fn diff(&self, paths: &[String], uncovered_only: bool) -> String {
        let claimed: BTreeSet<&str> = self
            .coverage
            .keys()
            .map(|(file, _)| file.as_str())
            .collect();
        let selected = |file: &SourceFile| {
            if paths.is_empty() {
                return claimed.contains(file.path.as_str());
            }
            paths.iter().any(|path| {
                let path = path.trim_end_matches('/');
                file.path == path || file.path.starts_with(&format!("{path}/"))
            })
        };

        let mut out = String::new();
        for file in self.files.iter().filter(|file| selected(file)) {
            let mut hunks = String::new();
            for function in &file.functions {
                let covered = self.covered_paths(&file.path, function);
                if uncovered_only && covered.len() == function.arms.len() {
                    continue;
                }
                let key = (file.path.clone(), function.name.clone());
                let coverage = self.coverage.get(&key);
                let status = match coverage {
                    Some(coverage) if coverage.stale => "STALE claim".to_owned(),
                    Some(coverage) => {
                        let names: Vec<&str> =
                            coverage.declarations.iter().map(String::as_str).collect();
                        let checked = if self.is_checked(&file.path, function) {
                            ", checked against mpv"
                        } else {
                            ""
                        };
                        format!("modelled by {}{checked}", names.join(", "))
                    }
                    None => "not modelled".to_owned(),
                };
                let _ = writeln!(
                    hunks,
                    "@@ {} : {}/{} paths, {status} @@",
                    function.name,
                    covered.len(),
                    function.arms.len()
                );
                for row in function.start_row..=function.end_row {
                    let line_paths = file.row_paths(function, row);
                    let sign = if line_paths.iter().all(|path| covered.contains(path)) {
                        ' '
                    } else {
                        '-'
                    };
                    let _ = writeln!(hunks, "{sign}{}", file.line(row));
                }
            }
            if !hunks.is_empty() {
                let _ = writeln!(out, "--- mpv/{}\n+++ model/{}", file.path, file.path);
                out.push_str(&hunks);
            }
        }
        out
    }

    fn show(&self, file: &str, name: &str) -> Result<String> {
        let function = self.function(file, name)?;
        let source = self
            .files
            .iter()
            .find(|source| source.path == file)
            .context("file vanished")?;
        let mut out = format!(
            "{} {file} {name} paths=all hash={}\n\n",
            claims::MARKER,
            function.hash
        );
        for row in function.start_row..=function.end_row {
            let starts: Vec<String> = function
                .arms
                .iter()
                .enumerate()
                .filter(|(_, arm)| arm.start_row == row)
                .map(|(id, arm)| format!("{id}:{}", arm.kind))
                .collect();
            let _ = writeln!(out, "{:>14} | {}", starts.join(" "), source.line(row));
        }
        Ok(out)
    }
}

/// A fidelity target: mpv-fidelity runs these functions against the model and
/// proves the comparison is sensitive by planting a bug in each.
#[derive(Debug, Deserialize)]
struct FidelityTarget {
    file: String,
    mutation: Vec<PlantedBug>,
}

#[derive(Debug, Deserialize)]
struct PlantedBug {
    function: String,
}

fn checked_functions(formal: &Path) -> Result<BTreeSet<(String, String)>> {
    let mut checked = BTreeSet::new();
    let Ok(targets) = fs::read_dir(formal.join("fidelity/targets")) else {
        return Ok(checked);
    };
    for target in targets {
        let manifest = target?.path().join("target.toml");
        if manifest.exists() {
            let target: FidelityTarget = read_toml(&manifest)?;
            checked.extend(
                target
                    .mutation
                    .into_iter()
                    .map(|bug| (target.file.clone(), bug.function)),
            );
        }
    }
    Ok(checked)
}

fn read_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

fn run(cli: &Cli) -> Result<(String, bool)> {
    let mpv = cli.mpv.clone().unwrap_or_else(|| cli.formal.join(".mpv"));
    let analysis = Analysis::load(&cli.formal, &mpv)?;
    Ok(match &cli.command {
        Cmd::Report => (analysis.report(), true),
        Cmd::Diff { paths, uncovered } => (analysis.diff(paths, *uncovered), true),
        Cmd::Show { file, function } => (analysis.show(file, function)?, true),
        Cmd::Check { full } => {
            let total = analysis.total();
            let complete = total.paths_covered == total.paths;
            let mut out = total.row("TOTAL");
            out.push_str(&analysis.problem_list());
            if *full && !complete {
                let _ = writeln!(
                    out,
                    "\n{} paths are not modelled",
                    total.paths - total.paths_covered
                );
            }
            (out, analysis.problems.is_empty() && (complete || !*full))
        }
    })
}

fn main() -> ExitCode {
    match run(&Cli::parse()) {
        Ok((output, passed)) => {
            // A closed pipe (`| head`) is not an error worth reporting.
            let _ = std::io::stdout().write_all(output.as_bytes());
            if passed {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(2)
        }
    }
}
