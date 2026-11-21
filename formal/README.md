# Formal model of mpv

This folder holds a Lean 4 model of mpv's behaviour, with machine-checked
theorems about it, and two tools. `mpv-cover` reports which mpv code paths the
model accounts for. `mpv-fidelity` checks that the model behaves like mpv's
real code.

| Path | Role |
| --- | --- |
| `Mpv/` | The Lean model. `Mpv/Common/Playlist.lean` mirrors `common/playlist.c`. |
| `mpv.lock.toml` | The mpv commit the model describes. |
| `scope.toml` | mpv files left out on purpose, each with a reason. |
| `cover/` | The `mpv-cover` tool. |
| `fidelity/` | The `mpv-fidelity` tool and one folder per checked target. |
| `Oracle/` | Executable wrappers that run the model on a script of operations. |

## Setup

Install Lean with [elan](https://github.com/leanprover/elan). The version is
pinned in `lean-toolchain`. Then, from the repository root:

```bash
make proofs
```

That fetches mpv at the pinned commit into `formal/.mpv`, builds it, checks
every proof with `lake build --wfail`, validates every coverage claim, and runs
the fidelity check. It runs on Linux and needs these packages (Ubuntu 24.04
names):

```bash
sudo apt-get install meson ninja-build pkg-config gcc g++ git python3 \
  libavcodec-dev libavfilter-dev libavformat-dev libavutil-dev \
  libswresample-dev libswscale-dev libass-dev
```

`make mpv-build` runs only the build step, `formal/build-mpv.sh`. It builds
libplacebo at the commit pinned in `formal/mpv.lock.toml` into `formal/.deps`,
because mpv needs a newer libplacebo than Ubuntu ships. Then it builds mpv as a
static library into `formal/.mpv-build`, with every optional feature off. Run
as root, it installs missing packages itself; otherwise it lists them and
stops. A second run skips finished stages and takes about a second. It writes:

- `compile_commands.json`: one entry per compiled mpv C file.
- `libmpv.a`: every mpv object.
- `harness.rsp` and `harness-link.rsp`: the compile and link flags for a
  program that includes mpv headers or an mpv `.c` file:

  ```bash
  cc @formal/.mpv-build/harness.rsp x.c @formal/.mpv-build/harness-link.rsp -o x
  ```

## Claiming coverage

A claim is a comment directly above the Lean declaration that models an mpv function:

```lean
-- mpv-cover: common/playlist.c playlist_get_next paths=all hash=e80538901518
def getNext (pl : Playlist) (direction : Int) : Option Entry :=
```

A path is the function's straight-line body (path 0) or one branch arm: an
`if` or `else` body, a `case`, a loop body, one side of `?:`, or the right
operand of a short-circuit `&&` or `||` (it runs only when the left side lets
it). Use `paths=all` or a list such as `paths=0,2,3`. The hash is a digest of
the function's source, so the claim goes stale when mpv changes that function.

Print a function's path ids and a ready-made claim line:

```bash
cargo run -q --release -p mpv-cover -- show common/playlist.c playlist_get_next
```

## Reading coverage

```bash
cargo run -q --release -p mpv-cover -- report                   # totals per mpv directory
cargo run -q --release -p mpv-cover -- diff                     # files that have claims
cargo run -q --release -p mpv-cover -- diff player --uncovered  # what is missing in player/
cargo run -q --release -p mpv-cover -- check --full             # fails until nothing is missing
```

`diff` prints mpv's source in unified-diff form. A line starting with `-` is on
a path the model does not account for.

`check` fails when a claim names a missing file, function, or path, when its
hash is stale, when it has no declaration, or when a Lean file contains
`sorry`, `axiom`, or another way around the proof checker.

## Checking the model against mpv

`mpv-fidelity` compiles mpv's real source file into a small harness, feeds the
harness and the Lean model the same random script of operations, and compares
every result and every resulting state.

It then plants bugs. Each `[[mutation]]` in a target's `target.toml` changes
one line in a copy of the mpv source. The comparison must notice every one. A
function counts as checked only if it has a planted bug, so "checked" means
the comparison is known to be sensitive to that function.

```bash
cargo run -q --release -p mpv-fidelity
cargo run -q --release -p mpv-fidelity -- --seed 7 --sequences 20000
```

A target is a folder in `fidelity/targets/` with a `harness.c`, a
`target.toml`, stub headers for mpv dependencies the checked functions do not
use, and a script generator in `fidelity/src/`.

## What coverage does and does not mean

A covered path means a Lean declaration claims to model it and the claim is
current. The `checked` column in `report` counts covered paths in functions
that `mpv-fidelity` runs against real mpv.

Checked is strong evidence, not proof. It is random testing, so an input the
generator never produces can still hide a difference. The generator only
produces inputs that satisfy mpv's own assertions.

The path counts also miss some of mpv. `report` lists each gap with its size:

- Branches hidden inside macros (`MPMAX`, `MP_TARRAY_INSERT_AT`) are not
  separate paths.
- Code outside functions, such as option tables, is not counted.
- Regions the C parser cannot read are not counted.
- Lua, JavaScript, Objective-C, and Swift sources are not parsed.

## Modelling conventions

- A C `int` is a Lean `Int`, so overflow is out of the model.
- Memory management (`talloc`, reference counts) is not modelled.
- `mp_assert` conditions become theorem hypotheses.
