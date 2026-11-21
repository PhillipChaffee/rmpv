# Formal model of mpv

This folder holds a Lean 4 model of mpv's behaviour, with machine-checked
theorems about it, and `mpv-cover`, a tool that reports which mpv code paths
the model accounts for.

| Path | Role |
| --- | --- |
| `Mpv/` | The Lean model. `Mpv/Common/Playlist.lean` mirrors `common/playlist.c`. |
| `mpv.lock.toml` | The mpv commit the model describes. |
| `scope.toml` | mpv files left out on purpose, each with a reason. |
| `cover/` | The `mpv-cover` tool. |

## Setup

Install Lean with [elan](https://github.com/leanprover/elan). The version is
pinned in `lean-toolchain`. Then, from the repository root:

```bash
make proofs
```

That fetches mpv at the pinned commit into `formal/.mpv`, checks every proof
with `lake build --wfail`, and validates every coverage claim.

## Claiming coverage

A claim is a comment directly above the Lean declaration that models an mpv function:

```lean
-- mpv-cover: common/playlist.c playlist_get_next paths=all hash=e80538901518
def getNext (pl : Playlist) (direction : Int) : Option Entry :=
```

A path is the function's straight-line body (path 0) or one branch arm: an
`if` or `else` body, a `case`, a loop body, or one side of `?:`. Use
`paths=all` or a list such as `paths=0,2,3`. The hash is a digest of the
function's source, so the claim goes stale when mpv changes that function.

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

## What coverage does and does not mean

A covered path means a Lean declaration claims to model it and the claim is
current. The tool does not check that the Lean definition behaves like the C
code. That link is a human review today. The plan is to close it by running the
Lean model and real mpv on the same inputs and comparing results.

The path counts also miss some of mpv. `report` lists each gap with its size:

- Branches hidden inside macros (`MPMAX`, `MP_TARRAY_INSERT_AT`) and
  short-circuit `&&` and `||` are not separate paths.
- Code outside functions, such as option tables, is not counted.
- Regions the C parser cannot read are not counted.
- Lua, JavaScript, Objective-C, and Swift sources are not parsed.

## Modelling conventions

- A C `int` is a Lean `Int`, so overflow is out of the model.
- Memory management (`talloc`, reference counts) is not modelled.
- `mp_assert` conditions become theorem hypotheses.
