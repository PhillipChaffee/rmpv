//! Reports which mpv code paths the Lean model in `formal/` accounts for.

mod claims;
mod preprocess;
mod source;

use anyhow::{Context as _, Result, anyhow, bail, ensure};
use claims::{Claim, Paths};
use clap::{Parser, Subcommand};
use globset::Glob;
use preprocess::Database;
use serde::Deserialize;
use source::{Function, SourceFile};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    io::Write as _,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
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
    /// mpv build holding `compile_commands.json`. Defaults to `<formal>/.mpv-build`.
    #[arg(long)]
    build: Option<PathBuf>,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Coverage totals per mpv directory.
    Report,
    /// mpv source as a diff: `-` lines are paths the model does not account for, `~` lines
    /// are unobservable paths, `!` lines are failed assertions in modelled functions.
    Diff {
        /// mpv files or directories. Defaults to files with at least one claim.
        paths: Vec<String>,
        /// Hide functions that are fully covered.
        #[arg(long)]
        uncovered: bool,
    },
    /// One function with its path ids and hash, for writing a claim: `FILE FUNCTION`.
    Show { file: String, function: String },
    /// Fail on invalid or stale claims. With `--full`, also fail on any path that is not
    /// covered, unobservable or asserted.
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
    unobservable: BTreeSet<usize>,
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
    paths_unobservable: usize,
    paths_asserted: usize,
    /// Covered paths in functions the fidelity check runs against real mpv.
    paths_checked: usize,
}

impl Tally {
    fn add(&mut self, function: &Function, accounted: &Accounted, checked: bool) {
        let covered = accounted.covered.len();
        self.functions += 1;
        self.functions_covered += usize::from(accounted.len() == function.arms.len());
        self.paths += function.arms.len();
        self.paths_covered += covered;
        self.paths_unobservable += accounted.unobservable.len();
        self.paths_asserted += accounted.asserted.len();
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

/// The paths of one function that a current claim accounts for, each in one set.
#[derive(Debug, Default)]
struct Accounted {
    covered: BTreeSet<usize>,
    unobservable: BTreeSet<usize>,
    /// `assert` arms: the model takes the assertion as a hypothesis, so their failing side is
    /// outside it.
    asserted: BTreeSet<usize>,
}

impl Accounted {
    fn len(&self) -> usize {
        self.covered.len() + self.unobservable.len() + self.asserted.len()
    }

    /// The sign `diff` gives a line on these paths.
    fn sign(&self, paths: &[usize]) -> char {
        let has = |set: &BTreeSet<usize>| paths.iter().any(|path| set.contains(path));
        if paths.iter().any(|path| {
            !self.covered.contains(path)
                && !self.unobservable.contains(path)
                && !self.asserted.contains(path)
        }) {
            '-'
        } else if has(&self.asserted) {
            '!'
        } else if has(&self.unobservable) {
            '~'
        } else {
            ' '
        }
    }
}

impl Analysis {
    fn load(formal: &Path, mpv: &Path, build: &Path) -> Result<Self> {
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
        let database = Database::load(build, mpv)?;

        let mut sources = Vec::new();
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
                sources.push(path);
            } else if let Some(language) = UNTRACKED_LANGUAGES.iter().find(|l| **l == extension) {
                let lines = String::from_utf8_lossy(&fs::read(entry.path())?)
                    .lines()
                    .count();
                *analysis.untracked_lines.entry(language).or_default() += lines;
            }
        }

        analysis.files = parse_sources(mpv, &sources, &database)?;

        let bugs = checked_functions(formal, &analysis.files)?;
        analysis.checked = bugs.checked;
        let scan = claims::scan(formal)?;
        analysis.problems = scan.problems;
        analysis.problems.extend(bugs.problems);
        for claim in scan.claims {
            if let Err(problem) = analysis.apply(&claim) {
                analysis
                    .problems
                    .push(format!("{}: {problem}", claim.origin));
            }
        }
        for ((file, function), coverage) in &analysis.coverage {
            if let Some(id) = coverage.paths.intersection(&coverage.unobservable).next() {
                analysis.problems.push(format!(
                    "{file} {function}: path {id} is claimed by one declaration and unobservable in another"
                ));
            }
        }
        Ok(analysis)
    }

    fn apply(&mut self, claim: &Claim) -> Result<()> {
        let function = self.function(&claim.file, &claim.function)?;
        let hash = function.hash.clone();
        let asserted = asserted_paths(function);
        let path_count = function.arms.len();
        let key = (claim.file.clone(), claim.function.clone());
        if claim.hash != hash {
            let coverage = self.coverage.entry(key).or_default();
            coverage.declarations.insert(claim.declaration.clone());
            coverage.stale = true;
            bail!(
                "stale: `{}` now hashes to {hash}, the claim says {}",
                claim.function,
                claim.hash
            );
        }
        let listed = match &claim.paths {
            Paths::All if !function.unreadable_rows.is_empty() => bail!(
                "`{}` has {} lines the parser could not read, so its path list is incomplete; \
                 list the claimed paths instead of `paths=all`",
                claim.function,
                function.unreadable_rows.len()
            ),
            Paths::All => None,
            Paths::Listed(ids) => Some(ids),
        };
        let named = listed.into_iter().flatten().chain(&claim.unobservable);
        for id in named {
            if *id >= path_count {
                bail!("`{}` has paths 0..{path_count}, not {id}", claim.function);
            }
            if asserted.contains(id) {
                bail!(
                    "path {id} of `{}` is an assertion failure; the tool counts it as asserted",
                    claim.function
                );
            }
        }
        let coverage = self.coverage.entry(key).or_default();
        coverage.declarations.insert(claim.declaration.clone());
        match listed {
            None => coverage.paths.extend(
                (0..path_count)
                    .filter(|id| !claim.unobservable.contains(id) && !asserted.contains(id)),
            ),
            Some(ids) => coverage.paths.extend(ids),
        }
        coverage.unobservable.extend(&claim.unobservable);
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

    /// What a current claim accounts for in a function. Empty without one, or when it is stale.
    fn accounted(&self, file: &str, function: &Function) -> Accounted {
        self.coverage
            .get(&(file.to_owned(), function.name.clone()))
            .filter(|coverage| !coverage.stale)
            .map(|coverage| Accounted {
                covered: coverage.paths.clone(),
                unobservable: coverage.unobservable.clone(),
                asserted: asserted_paths(function),
            })
            .unwrap_or_default()
    }

    fn tally(&self, tally: &mut Tally, file: &SourceFile) {
        for function in &file.functions {
            let accounted = self.accounted(&file.path, function);
            let checked = self.is_checked(&file.path, function);
            tally.add(function, &accounted, checked);
        }
    }

    fn is_checked(&self, file: &str, function: &Function) -> bool {
        self.checked
            .contains(&(file.to_owned(), function.name.clone()))
    }

    fn total(&self) -> Tally {
        let mut total = Tally::default();
        for file in &self.files {
            self.tally(&mut total, file);
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
            self.tally(by_directory.entry(directory).or_default(), file);
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
        let total = self.total();
        out.push_str(&total.row("TOTAL"));
        let _ = writeln!(
            out,
            "{:<12} {:>15} {:>7} paths only manage memory or write logs; not in covered",
            "unobservable", "", total.paths_unobservable
        );
        let _ = writeln!(
            out,
            "{:<12} {:>15} {:>7} paths fail an assertion in modelled functions; not in covered",
            "asserted", "", total.paths_asserted
        );
        let _ = writeln!(
            out,
            "{:<12} covered paths in functions with at least one planted bug that mpv-fidelity \
             must catch; sensitivity is shown per function, not per path",
            "checked"
        );

        let code: usize = self.files.iter().map(|file| file.code_lines).sum();
        let in_functions: usize = self.files.iter().map(|file| file.function_lines).sum();
        let unparsed: usize = self.files.iter().map(|file| file.unparsed_lines).sum();
        out.push_str("\nNot measured by the path counts above:\n");
        let _ = writeln!(
            out,
            "  {} of {code} C lines sit outside any counted function (tables, macros, \
             declarations, and code that `#if` leaves out of this build)",
            code - in_functions
        );
        let _ = writeln!(
            out,
            "  {unparsed} C lines outside functions are in regions the parser could not read"
        );
        let unreadable = self.files.iter().flat_map(|file| &file.functions);
        let (lines, functions) = unreadable.fold((0, 0), |(lines, functions), function| {
            let rows = function.unreadable_rows.len();
            (lines + rows, functions + usize::from(rows > 0))
        });
        let _ = writeln!(
            out,
            "  {lines} lines in {functions} functions are in regions the parser could not read; \
             branches there are not counted as paths"
        );
        let raw = |extension: &str| {
            let raw = self.files.iter().filter(|file| !file.preprocessed);
            raw.filter(|file| file.path.ends_with(extension)).count()
        };
        let _ = writeln!(
            out,
            "  {} .c files not compiled in this build and {} headers are parsed without the \
             preprocessor, so branches inside their macros are not counted",
            raw(".c"),
            raw(".h")
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
                let accounted = self.accounted(&file.path, function);
                if uncovered_only && accounted.len() == function.arms.len() {
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
                let mut extra = String::new();
                for (count, label) in [
                    (accounted.unobservable.len(), "unobservable"),
                    (accounted.asserted.len(), "asserted"),
                ] {
                    if count > 0 {
                        let _ = write!(extra, ", {count} {label}");
                    }
                }
                let _ = writeln!(
                    hunks,
                    "@@ {} : {}/{} paths{extra}, {status} @@",
                    function.name,
                    accounted.covered.len(),
                    function.arms.len()
                );
                for row in function.start_row..=function.end_row {
                    let sign = accounted.sign(&file.row_paths(function, row));
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
        let asserted = asserted_paths(function);
        let paths = if function.unreadable_rows.is_empty() {
            "all".to_owned()
        } else {
            let ids = (0..function.arms.len()).filter(|id| !asserted.contains(id));
            ids.map(|id| id.to_string()).collect::<Vec<_>>().join(",")
        };
        let mut out = format!(
            "{} {file} {name} paths={paths} hash={}\n\n",
            claims::MARKER,
            function.hash
        );
        if !function.unreadable_rows.is_empty() {
            let _ = writeln!(
                out,
                "The parser could not read the lines marked `X`. Branches there have no path id,\n\
                 so `paths=all` is refused; check those lines before claiming the list above.\n"
            );
        }
        for row in function.start_row..=function.end_row {
            let starts: Vec<String> = function
                .arms
                .iter()
                .enumerate()
                .filter(|(_, arm)| arm.start_row == row)
                .map(|(id, arm)| format!("{id}:{}", arm.kind))
                .collect();
            let mark = if function.unreadable_rows.contains(&row) {
                'X'
            } else {
                '|'
            };
            let _ = writeln!(out, "{:>14} {mark} {}", starts.join(" "), source.line(row));
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
    find: String,
}

/// The functions with a planted bug that sits where its label says, and a problem for every
/// bug that does not.
#[derive(Debug, Default)]
struct PlantedBugs {
    checked: BTreeSet<(String, String)>,
    problems: Vec<String>,
}

/// What is wrong with a bug's label, if anything: the function it names must exist in the
/// file, and its `find` text must lie within that function's lines wherever it occurs.
fn label_problem(source: &SourceFile, bug: &PlantedBug) -> Option<String> {
    let file = &source.path;
    let named = |function: &&Function| function.name == bug.function;
    let Some(function) = source.functions.iter().find(named) else {
        return Some(format!("but {file} has no such function"));
    };
    let within = source.byte_range(function);
    let inside = |at: usize| within.start <= at && at + bug.find.len() <= within.end;
    let mut starts = source.text().match_indices(&bug.find).map(|(at, _)| at);
    if !source.text().contains(&bug.find) {
        Some(format!("but its `find` text does not occur in {file}"))
    } else if starts.all(inside) {
        None
    } else {
        Some(format!(
            "but its `find` text is outside that function in {file}"
        ))
    }
}

/// Checks one target's labels. A label is all that ties a bug to a function, so one that
/// names the wrong function would mark that function checked for nothing.
fn planted_bugs(manifest: &str, target: &FidelityTarget, files: &[SourceFile]) -> PlantedBugs {
    let mut bugs = PlantedBugs::default();
    let file = &target.file;
    let Some(source) = files.iter().find(|source| source.path == *file) else {
        bugs.problems
            .push(format!("{manifest}: no in-scope mpv file `{file}`"));
        return bugs;
    };
    for (index, bug) in target.mutation.iter().enumerate() {
        if let Some(problem) = label_problem(source, bug) {
            bugs.problems.push(format!(
                "{manifest}: mutation {} names `{}`, {problem}",
                index + 1,
                bug.function
            ));
        } else {
            bugs.checked.insert((file.clone(), bug.function.clone()));
        }
    }
    bugs
}

/// Every target's planted bugs.
fn checked_functions(formal: &Path, files: &[SourceFile]) -> Result<PlantedBugs> {
    let mut all = PlantedBugs::default();
    let Ok(targets) = fs::read_dir(formal.join("fidelity/targets")) else {
        return Ok(all);
    };
    let mut manifests = Vec::new();
    for target in targets {
        manifests.push(target?.path().join("target.toml"));
    }
    manifests.sort();
    for manifest in manifests.iter().filter(|manifest| manifest.exists()) {
        let target: FidelityTarget = read_toml(manifest)?;
        let name = manifest.strip_prefix(formal).unwrap_or(manifest);
        let bugs = planted_bugs(&name.display().to_string(), &target, files);
        all.checked.extend(bugs.checked);
        all.problems.extend(bugs.problems);
    }
    Ok(all)
}

fn asserted_paths(function: &Function) -> BTreeSet<usize> {
    let arms = function.arms.iter().enumerate();
    arms.filter(|(_, arm)| arm.kind == "assert")
        .map(|(id, _)| id)
        .collect()
}

/// Parses every source file, running the preprocessor first for files the build compiles.
fn parse_sources(mpv: &Path, paths: &[String], database: &Database) -> Result<Vec<SourceFile>> {
    let parse = |path: &String| -> Result<SourceFile> {
        let bytes = fs::read(mpv.join(path)).with_context(|| format!("reading {path}"))?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        match database.get(path) {
            Some(compile) => {
                let preprocessed = compile.preprocess()?;
                SourceFile::parse_preprocessed(path.clone(), text, &preprocessed)
            }
            None => SourceFile::parse(path.clone(), text),
        }
    };
    let next = AtomicUsize::new(0);
    let workers = thread::available_parallelism().map_or(1, NonZeroUsize::get);
    let mut parsed = thread::scope(|scope| {
        let workers: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(path) = paths.get(index) else {
                            return done;
                        };
                        done.push((index, parse(path)));
                    }
                })
            })
            .collect();
        let mut parsed = Vec::with_capacity(paths.len());
        for worker in workers {
            parsed.extend(
                worker
                    .join()
                    .map_err(|_| anyhow!("a parser thread panicked"))?,
            );
        }
        Ok::<_, anyhow::Error>(parsed)
    })?;
    parsed.sort_by_key(|(index, _)| *index);
    parsed.into_iter().map(|(_, file)| file).collect()
}

fn read_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

fn run(cli: &Cli) -> Result<(String, bool)> {
    let mpv = cli.mpv.clone().unwrap_or_else(|| cli.formal.join(".mpv"));
    let build = cli
        .build
        .clone()
        .unwrap_or_else(|| cli.formal.join(".mpv-build"));
    let analysis = Analysis::load(&cli.formal, &mpv, &build)?;
    Ok(match &cli.command {
        Cmd::Report => (analysis.report(), true),
        Cmd::Diff { paths, uncovered } => (analysis.diff(paths, *uncovered), true),
        Cmd::Show { file, function } => (analysis.show(file, function)?, true),
        Cmd::Check { full } => {
            let total = analysis.total();
            let missing = total.paths.saturating_sub(
                total.paths_covered + total.paths_unobservable + total.paths_asserted,
            );
            let complete = missing == 0;
            let mut out = total.row("TOTAL");
            out.push_str(&analysis.problem_list());
            if *full && !complete {
                let _ = writeln!(out, "\n{missing} paths are not modelled");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn claim(paths: Paths, unobservable: &[usize], hash: &str) -> Claim {
        Claim {
            file: "f.c".to_owned(),
            function: "f".to_owned(),
            paths,
            unobservable: unobservable.iter().copied().collect(),
            hash: hash.to_owned(),
            declaration: "model".to_owned(),
            origin: "M.lean:1".to_owned(),
        }
    }

    #[test]
    fn unobservable_paths_are_reported_apart_from_covered_ones() {
        let text = "void f(int a) {\n    if (a)\n        grow();\n    use(a);\n}\n";
        let file = SourceFile::parse("f.c".to_owned(), text.to_owned()).expect("parses");
        let hash = file.functions.first().expect("one function").hash.clone();
        let mut analysis = Analysis {
            files: vec![file],
            ..Analysis::default()
        };
        analysis
            .apply(&claim(Paths::All, &[1], &hash))
            .expect("claim applies");

        let total = analysis.total();
        assert_eq!(
            (total.paths, total.paths_covered, total.paths_unobservable),
            (2, 1, 1)
        );
        assert_eq!(total.functions_covered, 1);
        let diff = analysis.diff(&[], false);
        assert!(diff.contains("\n~        grow();\n"), "{diff}");
        assert!(diff.contains("\n     use(a);\n"), "{diff}");
        assert!(analysis.apply(&claim(Paths::All, &[2], &hash)).is_err());
    }

    #[test]
    fn assert_arms_of_a_claimed_function_are_asserted_and_not_claimable() {
        let text = "void f(int a) {\n    mp_assert(a);\n    if (a)\n        use(a);\n}\n";
        let file = SourceFile::parse("f.c".to_owned(), text.to_owned()).expect("parses");
        let hash = file.functions.first().expect("one function").hash.clone();
        let mut analysis = Analysis {
            files: vec![file],
            ..Analysis::default()
        };
        let unclaimed = analysis.total();
        assert_eq!((unclaimed.paths, unclaimed.paths_asserted), (3, 0));

        let listed = Paths::Listed([0, 1, 2].into_iter().collect());
        assert!(analysis.apply(&claim(listed, &[], &hash)).is_err());
        assert!(analysis.apply(&claim(Paths::All, &[1], &hash)).is_err());
        analysis
            .apply(&claim(Paths::All, &[], &hash))
            .expect("claim applies");

        let total = analysis.total();
        assert_eq!(
            (
                total.paths,
                total.paths_covered,
                total.paths_unobservable,
                total.paths_asserted
            ),
            (3, 2, 0, 1)
        );
        assert_eq!(total.functions_covered, 1);
        let diff = analysis.diff(&[], false);
        assert!(diff.contains("\n!    mp_assert(a);\n"), "{diff}");
        assert!(diff.contains("\n         use(a);\n"), "{diff}");
    }

    #[test]
    fn a_planted_bug_counts_only_where_its_label_says_it_is() {
        let text = "int f(int a) {\n    return a + 1;\n}\n\nint g(int a) {\n    if (a)\n        return a + 2;\n    return 0;\n}\n";
        let file = SourceFile::parse("f.c".to_owned(), text.to_owned()).expect("parses");
        let manifest = r#"
            file = "f.c"
            [[mutation]]
            function = "f"
            find = "return a + 1;"
            [[mutation]]
            function = "g"
            find = "if (a)\n        return a + 2;"
            [[mutation]]
            function = "g"
            find = "return a + 1;"
            [[mutation]]
            function = "h"
            find = "return a + 2;"
            [[mutation]]
            function = "f"
            find = "return a + 3;"
            [[mutation]]
            function = "f"
            find = "return a + 1;\n}\n\nint g"
            [[mutation]]
            function = "g"
            find = "return"
        "#;
        let target: FidelityTarget = toml::from_str(manifest).expect("manifest parses");

        let bugs = planted_bugs("t/target.toml", &target, &[file]);

        let function = |name: &str| ("f.c".to_owned(), name.to_owned());
        assert_eq!(bugs.checked, [function("f"), function("g")].into());
        assert_eq!(
            bugs.problems,
            [
                "t/target.toml: mutation 3 names `g`, but its `find` text is outside that function in f.c",
                "t/target.toml: mutation 4 names `h`, but f.c has no such function",
                "t/target.toml: mutation 5 names `f`, but its `find` text does not occur in f.c",
                "t/target.toml: mutation 6 names `f`, but its `find` text is outside that function in f.c",
                "t/target.toml: mutation 7 names `g`, but its `find` text is outside that function in f.c",
            ]
        );
    }

    #[test]
    fn a_planted_bug_in_a_file_out_of_scope_is_a_problem() {
        let manifest = "file = \"g.c\"\n[[mutation]]\nfunction = \"f\"\nfind = \"x\"\n";
        let target: FidelityTarget = toml::from_str(manifest).expect("manifest parses");
        let bugs = planted_bugs("t/target.toml", &target, &[]);
        assert!(bugs.checked.is_empty());
        assert_eq!(bugs.problems, ["t/target.toml: no in-scope mpv file `g.c`"]);
    }

    #[test]
    fn paths_all_is_refused_where_the_parser_could_not_read_the_function() {
        let text = "int f(int a) {\n    if (a)\n        return __builtin_offsetof(struct s, m);\n    return 0;\n}\n";
        let file = SourceFile::parse("f.c".to_owned(), text.to_owned()).expect("parses");
        let hash = file.functions.first().expect("one function").hash.clone();
        let mut analysis = Analysis {
            files: vec![file],
            ..Analysis::default()
        };
        assert!(analysis.apply(&claim(Paths::All, &[], &hash)).is_err());
        let listed = Paths::Listed(std::iter::once(0).collect());
        analysis
            .apply(&claim(listed, &[], &hash))
            .expect("an explicit list applies");
        assert!(
            analysis.report().contains("1 lines in 1 functions"),
            "{}",
            analysis.report()
        );
    }
}
