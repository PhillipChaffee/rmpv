//! Checks the Lean model against mpv's real code by running both on the same inputs.
//!
//! For each target it compares the outputs on a random script, then plants bugs
//! in a copy of the mpv source and confirms the comparison notices each one.

mod playlist;

use anyhow::{Context as _, Result, bail, ensure};
use clap::Parser;
use playlist::Rng;
use serde::Deserialize;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

/// A target's name and the generator for its operation scripts.
type Generator = fn(&mut Rng, usize, usize) -> String;
const TARGETS: [(&str, Generator); 1] = [("playlist", playlist::script)];

#[derive(Debug, Parser)]
#[command(about)]
struct Cli {
    /// Directory holding the Lean model and `fidelity/targets`.
    #[arg(long, default_value = "formal")]
    formal: PathBuf,
    /// mpv checkout at the locked commit. Defaults to `<formal>/.mpv`.
    #[arg(long)]
    mpv: Option<PathBuf>,
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
    sources: Vec<String>,
    file: String,
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
    target_dir: &'a Path,
    work: &'a Path,
    script: &'a Path,
    target: &'a Target,
}

impl Run<'_> {
    /// Builds the harness against mpv's source, with `mutation` planted when given.
    fn build(&self, mutation: Option<&Mutation>) -> Result<PathBuf> {
        let binary = self.work.join("harness");
        let mut cc = Command::new("cc");
        cc.args([
            "-std=gnu11",
            "-w",
            "-O1",
            "-ffunction-sections",
            "-fdata-sections",
        ]);
        cc.args(["-Wl,--gc-sections", "-Werror=implicit-function-declaration"]);
        if let Some(mutation) = mutation {
            let original = self.mpv.join(&self.target.file);
            let text = fs::read_to_string(&original)?;
            ensure!(
                text.matches(&mutation.find).count() == 1,
                "`{}` must occur exactly once in {}",
                mutation.find,
                self.target.file
            );
            let mutated = self.work.join("mutant").join(&self.target.file);
            let directory = mutated.parent().context("mutant path has no parent")?;
            fs::create_dir_all(directory)?;
            fs::write(&mutated, text.replace(&mutation.find, &mutation.replace))?;
            // The copy still includes its sibling headers by bare name.
            cc.arg("-I").arg(self.work.join("mutant"));
            cc.arg("-I")
                .arg(original.parent().context("mpv file has no parent")?);
        }
        cc.arg("-I")
            .arg(self.target_dir.join("stubs"))
            .arg("-I")
            .arg(self.mpv);
        cc.arg(self.target_dir.join("harness.c"));
        cc.args(
            self.target
                .sources
                .iter()
                .map(|source| self.mpv.join(source)),
        );
        let output = cc.arg("-o").arg(&binary).output().context("running cc")?;
        ensure!(
            output.status.success(),
            "harness does not compile:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(binary)
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
    for mutation in &target.mutation {
        let mutant = run.output(&run.build(Some(mutation))?)?;
        if first_divergence(&model, &mutant).is_none() {
            survivors += 1;
            println!(
                "{name}: SURVIVED in {}: `{}`",
                mutation.function, mutation.replace
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
