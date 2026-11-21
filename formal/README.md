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
the function's original source lines, so the claim goes stale when mpv changes
that function.

`mpv-cover` finds paths in the preprocessed source, so branches inside macros
count. It runs `cc -E` on each file with that file's flags from
`formal/.mpv-build/compile_commands.json`, and keeps only the functions defined
in the file itself. An arm that a macro expands to belongs to the line that
invokes the macro, so `mp_assert(x)` has a `?` path (the assertion holds) and a
`:` path (it fails and aborts) on its own line. The body of a
`do { } while (0)` is not a separate path, because it always runs once, and
branches inside `sizeof` are not paths, because `sizeof` never evaluates its
operand. Without `compile_commands.json`, every command stops and asks for
`make mpv-build`; pass `--build <dir>` to read another build.

A path that exists in C but whose only effect is memory management or log
output, such as the branch in `MP_TARRAY_INSERT_AT` that grows the array, can
be marked `unobservable`:

```lean
-- mpv-cover: common/playlist.c playlist_insert_at paths=0,1,3,4,6,7,8 unobservable=10 hash=5b59e2deca02
```

A path cannot be in both lists. With `paths=all`, `unobservable=` takes its
paths out of `all`. Unobservable paths are never counted as covered. Any other
path the model does not account for, such as an `mp_assert` failure, stays out
of both lists.

Print a function's path ids and a ready-made claim line:

```bash
cargo run -q --release -p mpv-cover -- show common/playlist.c playlist_get_next
```

## Reading coverage

```bash
cargo run -q --release -p mpv-cover -- report                   # totals per mpv directory
cargo run -q --release -p mpv-cover -- diff                     # files that have claims
cargo run -q --release -p mpv-cover -- diff player --uncovered  # what is missing in player/
cargo run -q --release -p mpv-cover -- check --full             # fails until every path is covered or unobservable
```

`diff` prints mpv's source in unified-diff form. A line starting with `-` is on
a path the model does not account for. A line starting with `~` is on an
unobservable path and on no unaccounted one. `report` prints the number of
unobservable paths on its own line, under `TOTAL`.

`check` fails when a claim names a missing file, function, or path, when its
hash is stale, when one path is both claimed and unobservable, when it has no declaration, or when a Lean file contains
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

- Files the Linux build does not compile (optional features are off) and
  headers have no compile command, so they are parsed as written. Branches
  inside the macros they use are not separate paths.
- In compiled files, code that `#if` leaves out of this build, such as Windows
  branches, is not counted.
- Code outside functions, such as option tables, is not counted.
- Regions the C parser cannot read are not counted.
- Lua, JavaScript, Objective-C, and Swift sources are not parsed.

## Modelling conventions

- A C `int` is a Lean `Int`, so overflow is out of the model.
- Memory management (`talloc`, reference counts) and log output are not
  modelled. Paths that only do these are marked `unobservable`.
- `mp_assert` conditions become theorem hypotheses.
