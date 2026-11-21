//! Checks the Lean model against mpv's real code by running both on the same inputs.
//!
//! For each target it compares the outputs on a random script, then plants bugs
//! in a copy of the mpv source and confirms the comparison notices each one.

mod avsync;
mod playlist;
mod playloop;
mod rng;

use anyhow::{Context as _, Result, bail, ensure};
use clap::Parser;
use rng::Rng;
use serde::Deserialize;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

/// A target's name and the generator for its operation scripts.
type Generator = fn(&mut Rng, usize, usize) -> String;
const TARGETS: [(&str, Generator); 4] = [
    ("playlist", playlist::script),
    ("playloop", playloop::script),
    ("audio", avsync::audio_script),
    ("video", avsync::video_script),
];

#[derive(Debug, Parser)]
#[command(about)]
struct Cli {
    /// Directory holding the Lean model and `fidelity/targets`.
    #[arg(long, default_value = "formal")]
    formal: PathBuf,
    /// mpv checkout at the locked commit. Defaults to `<formal>/.mpv`.
    #[arg(long)]
    mpv: Option<PathBuf>,
    /// Static mpv build from `make mpv-build`. Defaults to `<formal>/.mpv-build`.
    #[arg(long)]
    build: Option<PathBuf>,
    /// Independent operation sequences per target, each starting from an empty state.
    #[arg(long, default_value_t = 5000)]
    sequences: usize,
    /// Operations per sequence.
    #[arg(long, default_value_t = 40)]
    operations: usize,
    #[arg(long, default_value_t = 1)]
    seed: u64,
}

#[derive(Debug, Deserialize)]
struct Target {
    oracle: String,
    file: String,
    /// Extra linker arguments, such as `-Wl,--wrap=name`.
    #[serde(default)]
    link_args: Vec<String>,
    mutation: Vec<Mutation>,
}

#[derive(Debug, Deserialize)]
struct Mutation {
    function: String,
    find: String,
    replace: String,
}

/// Where one differential run reads from and writes to.
#[derive(Debug)]
struct Run<'a> {
    mpv: &'a Path,
    build: &'a Path,
    target_dir: &'a Path,
    work: &'a Path,
    script: &'a Path,
    target: &'a Target,
}

impl Run<'_> {
    /// Builds the harness against mpv's source. With a mutation, given with its
    /// index in the manifest, the bug is planted and the build gets a directory
    /// of its own, so mutants can be built at the same time.
    fn build(&self, mutation: Option<(usize, &Mutation)>) -> Result<PathBuf> {
        let directory = mutation.map_or_else(
            || self.work.to_path_buf(),
            |(index, _)| self.work.join("mutants").join(index.to_string()),
        );
        let binary = directory.join("harness");
        let mut cc = Command::new("cc");
        if let Some((_, mutation)) = mutation {
            let original = self.mpv.join(&self.target.file);
            let text = fs::read_to_string(&original)?;
            ensure!(
                text.matches(&mutation.find).count() == 1,
                "`{}` must occur exactly once in {}",
                mutation.find,
                self.target.file
            );
            let source = directory.join("src");
            let mutated = source.join(&self.target.file);
            fs::create_dir_all(mutated.parent().context("mutant path has no parent")?)?;
            fs::write(&mutated, text.replace(&mutation.find, &mutation.replace))?;
            // The copy still includes its sibling headers by bare name.
            cc.arg("-I").arg(source);
            cc.arg("-I")
                .arg(original.parent().context("mpv file has no parent")?);
        }
        // mpv's build makes this an error; the response file carries no -W flags.
        cc.arg("-Werror=implicit-function-declaration");
        // Mutant directories must precede the -I flags in the response file.
        cc.arg(format!("@{}", self.build.join("harness.rsp").display()));
        cc.arg(self.target_dir.join("harness.c"));
        cc.arg(format!(
            "@{}",
            self.build.join("harness-link.rsp").display()
        ));
        cc.args(&self.target.link_args);
        let output = cc.arg("-o").arg(&binary).output().context("running cc")?;
        ensure!(
            output.status.success(),
            "harness does not compile{}:\n{}",
            mutation.map_or_else(String::new, |(_, mutation)| format!(
                " with `{}` replaced by `{}` in {}",
                mutation.find, mutation.replace, mutation.function
            )),
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(binary)
    }

    /// Whether each planted bug goes unnoticed, in the manifest's order.
    /// Mutants are built and run in parallel. The first one in that order that
    /// does not compile is the error.
    fn survivors(&self, model: &[String]) -> Result<Vec<bool>> {
        let mutations = &self.target.mutation;
        let workers = thread::available_parallelism()
            .map_or(1, std::num::NonZero::get)
            .min(mutations.len());
        let next = AtomicUsize::new(0);
        let work = || {
            let mut done = Vec::new();
            loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(mutation) = mutations.get(index) else {
                    return done;
                };
                let survived = self
                    .build(Some((index, mutation)))
                    .and_then(|mutant| self.output(&mutant))
                    .map(|mutant| first_divergence(model, &mutant).is_none());
                done.push((index, survived));
            }
        };
        let mut outcomes = Vec::new();
        thread::scope(|scope| {
            let handles: Vec<_> = (0..workers).map(|_| scope.spawn(work)).collect();
            for handle in handles {
                match handle.join() {
                    Ok(done) => outcomes.extend(done),
                    Err(_) => bail!("a thread building mutants panicked"),
                }
            }
            Ok(())
        })?;
        outcomes.sort_by_key(|(index, _)| *index);
        outcomes.into_iter().map(|(_, survived)| survived).collect()
    }

    /// Runs a program on the script. A crash or hang yields whatever it printed first.
    fn output(&self, program: &Path) -> Result<Vec<String>> {
        let output = Command::new("timeout")
            .arg("120")
            .arg(program)
            .stdin(Stdio::from(File::open(self.script)?))
            .output()
            .with_context(|| format!("running {}", program.display()))?;
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_owned)
            .collect())
    }
}

/// The first script line where the two outputs disagree.
fn first_divergence(model: &[String], mpv: &[String]) -> Option<usize> {
    let differing = model.iter().zip(mpv).position(|(model, mpv)| model != mpv);
    differing.or_else(|| (model.len() != mpv.len()).then(|| model.len().min(mpv.len())))
}

fn explain(script: &str, at: usize, model: &[String], mpv: &[String]) -> String {
    let lines: Vec<&str> = script.lines().collect();
    let start = lines
        .iter()
        .take(at + 1)
        .rposition(|line| *line == "reset")
        .unwrap_or(0);
    let sequence: Vec<&str> = lines.iter().take(at + 1).skip(start).copied().collect();
    let shown = |output: &[String]| {
        output
            .get(at)
            .cloned()
            .unwrap_or_else(|| "<no output>".into())
    };
    format!(
        "  after: {}\n  model: {}\n  mpv:   {}\n",
        sequence.join("; "),
        shown(model),
        shown(mpv)
    )
}

fn check(cli: &Cli, name: &str, generate: Generator) -> Result<bool> {
    let mpv = cli.mpv.clone().unwrap_or_else(|| cli.formal.join(".mpv"));
    let build = cli
        .build
        .clone()
        .unwrap_or_else(|| cli.formal.join(".mpv-build"));
    for response in ["harness.rsp", "harness-link.rsp"] {
        ensure!(
            build.join(response).exists(),
            "{} is missing; run `make mpv-build`",
            build.join(response).display()
        );
    }
    let target_dir = cli.formal.join("fidelity/targets").join(name);
    let manifest = target_dir.join("target.toml");
    let target: Target = toml::from_str(&fs::read_to_string(&manifest)?)
        .with_context(|| format!("parsing {}", manifest.display()))?;
    let work = cli.formal.join(".fidelity").join(name);
    fs::create_dir_all(&work)?;

    let script = generate(&mut Rng(cli.seed), cli.sequences, cli.operations);
    let script_path = work.join("script");
    fs::write(&script_path, &script)?;
    let run = Run {
        mpv: &mpv,
        build: &build,
        target_dir: &target_dir,
        work: &work,
        script: &script_path,
        target: &target,
    };

    let oracle = cli.formal.join(".lake/build/bin").join(&target.oracle);
    ensure!(
        oracle.exists(),
        "{} is missing; run `lake build` in formal/",
        oracle.display()
    );
    let model = run.output(&oracle)?;
    let expected = script.lines().count();
    if model.len() != expected {
        bail!(
            "the Lean oracle answered {} of {expected} operations",
            model.len()
        );
    }

    let real = run.output(&run.build(None)?)?;
    let mut passed = true;
    if let Some(at) = first_divergence(&model, &real) {
        passed = false;
        println!("{name}: model and mpv DIVERGE at operation {at}");
        print!("{}", explain(&script, at, &model, &real));
    } else {
        println!("{name}: model and mpv agree on {expected} operations");
    }

    let mut survivors = 0;
    for (mutation, survived) in target.mutation.iter().zip(run.survivors(&model)?) {
        if survived {
            survivors += 1;
            println!(
                "{name}: SURVIVED in {}: `{}` replaced by `{}`",
                mutation.function, mutation.find, mutation.replace
            );
        }
    }
    let total = target.mutation.len();
    println!(
        "{name}: caught {} of {total} planted bugs",
        total - survivors
    );
    Ok(passed && survivors == 0)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut passed = true;
    for (name, generate) in TARGETS {
        match check(&cli, name, generate) {
            Ok(result) => passed &= result,
            Err(error) => {
                eprintln!("error: {name}: {error:#}");
                return ExitCode::from(2);
            }
        }
    }
    if passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
