# Formal model of mpv

This folder holds a Lean 4 model of mpv's behaviour, with machine-checked
theorems about it, and two tools. `mpv-cover` reports which mpv code paths the
model accounts for. `mpv-fidelity` checks that the model behaves like mpv's
real code.

| Path | Role |
| --- | --- |
| `Mpv/` | The Lean model. `Mpv/Common/Playlist.lean` mirrors `common/playlist.c`. In `Mpv/Player/`, `Playloop.lean`, `Audio.lean` and `Video.lean` mirror parts of `player/playloop.c`, `player/audio.c` and `player/video.c`. Each has a `…Theorems.lean` with its proofs. |
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
  program that includes mpv headers or an mpv `.c` file. `harness.rsp` holds
  every flag mpv's own C files are compiled with, such as `-O2` and
  `-fno-signed-zeros`, except warnings, diagnostics and output names. The
  script stops if mpv's files do not all share one flag set.

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
invokes the macro, so `mp_assert(x)` has a `?` path (the assertion holds) and an
`assert` path (it fails and aborts) on its own line. The body of a
`do { } while (0)` is not a separate path, because it always runs once, and
branches inside `sizeof` or `__typeof__` are not paths, because neither
evaluates its operand. Without `compile_commands.json`, every command stops and asks for
`make mpv-build`; pass `--build <dir>` to read another build.

A path that exists in C but whose only effect is memory management or log
output, such as the branch in `MP_TARRAY_INSERT_AT` that grows the array, can
be marked `unobservable`:

```lean
-- mpv-cover: common/playlist.c playlist_insert_at paths=0,1,3,4,6,7,8 unobservable=10 hash=5b59e2deca02
```

A path cannot be in both lists. With `paths=all`, `unobservable=` takes its
paths out of `all`. Unobservable paths are never counted as covered. Any other
path the model does not account for stays out of both lists.

An arm that does nothing but call glibc's assertion-failure handler
(`__assert_fail`, which `mp_assert`, `mp_require` and `assert` expand to in this
build) has the kind `assert`; `show` labels it. In a file parsed without the
preprocessor, each call to one of those macros is one `assert` arm. The tool
finds these arms itself: a claim may not list them in `paths=` or
`unobservable=`, and `paths=all` leaves them out. In a function with a current
claim they count as `asserted`, on their own line in `report`, and `diff` marks
their lines with `!`. In a function without a claim they are uncovered like any
other path.

Asserted means only this: the model takes the asserted condition as a
hypothesis of its theorems and a precondition of the fidelity generator, so it
describes mpv only while the assertion holds. It says nothing about what mpv
does when an assertion fails, and it does not prove that no input can make one
fail.

Print a function's path ids and a ready-made claim line:

```bash
cargo run -q --release -p mpv-cover -- show common/playlist.c playlist_get_next
```

## Reading coverage

```bash
cargo run -q --release -p mpv-cover -- report                   # totals per mpv directory
cargo run -q --release -p mpv-cover -- diff                     # files that have claims
cargo run -q --release -p mpv-cover -- diff player --uncovered  # what is missing in player/
cargo run -q --release -p mpv-cover -- check --full             # fails until every path is covered, unobservable or asserted
```

`diff` prints mpv's source in unified-diff form. A line starting with `-` is on
a path the model does not account for. A line starting with `~` is on an
unobservable path and on no unaccounted one. A line starting with `!` is on an
asserted path and on no unaccounted one. `report` prints the numbers of
unobservable and asserted paths on their own lines, under `TOTAL`.

`check` fails when a claim names a missing file, function, or path, when its
hash is stale, when one path is both claimed and unobservable, when it lists
an `assert` path, when it has no declaration, or when a Lean file contains
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
`target.toml`, and a script generator in `fidelity/src/`. The harness is built
with `cc @harness.rsp harness.c @harness-link.rsp` against the real static
build in `.mpv-build` (`make mpv-build`; override with `--build`). A manifest
may add `link_args`, extra linker arguments such as `-Wl,--wrap=name`.

The harness does not run the object file mpv's build made for the file under
test. `harness.c` includes that `.c` file, so the file is compiled again from
source, together with the harness, with the flags in `harness.rsp`: the
code-generation flags of the real build. Everything the file calls in other
mpv files comes from `libmpv.a`, as mpv's build compiled it, except the callees
a target replaces with `--wrap`. A planted bug is compiled the same way, from
an edited copy of the file.

| Target | mpv file | What the harness sets up |
| --- | --- | --- |
| `playlist` | `common/playlist.c` | Real playlists. The random numbers and the demuxer that opens a playlist file are replaced. |
| `playloop` | `player/playloop.c` | A zero-initialized `MPContext` with the options, demuxer and chapter fields the script fills in. No player runs. `get_play_start_pts` and `get_play_end_pts` are replaced. |
| `audio` | `player/audio.c` | A zero-initialized `MPContext` with the gain options, an `ao_chain`, and a track whose codec holds the replaygain data. No player runs and no audio output exists: `ao` is only tested for `NULL`, and `ao_get_delay` is replaced. |
| `video` | `player/video.c` | A zero-initialized `MPContext` with the sync options, an `ao_chain`, a `vo_chain` with its filter's `container_fps`, a `vo` with its options, and the past and next frame arrays. No player runs. `written_audio_pts` and `playing_audio_pts` are mpv's own, from the unchanged `audio.c`. `ao_get_delay`, `ao_get_format` and `af_fmt_is_pcm` are replaced. |

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
- Code the C parser cannot read has no paths. `report` gives two numbers: lines
  outside functions, and lines inside functions with the number of functions
  they are in. A branch in an unreadable line inside a function has no path id,
  so that function's path list is incomplete: `show` marks those lines with
  `X`, and `check` refuses `paths=all` for it, so a claim must list its paths.
- Lua, JavaScript, Objective-C, and Swift sources are not parsed.

## Known mpv defects and avoided inputs

This is the one list of inputs on which mpv misbehaves, or on which the model
says nothing. A coverage figure carries these caveats even at 100%. Each was
found by reading the C source. None has been reproduced by running mpv.

| Function | Input or state | What mpv does | In the model | Stated at |
| --- | --- | --- | --- | --- |
| `playlist_unshuffle` with `cmp_unshuffle` (`common/playlist.c`) | An entry with `original_index` -1 sits after an entry with a known index: shuffle, append, then unshuffle. | The comparison is not a consistent total order, so the result of `qsort` is undefined. | Avoided: the generator does not unshuffle such a playlist. No theorem assumes the comparison is consistent; each states what it needs of `qsort`'s output as a hypothesis. `unshuffle_entries` needs nothing. `unshuffle_perm` and `unshuffle_wf` need only that the output is a permutation of the input. `unshuffle_shuffle_entries` and `unshuffle_shuffle` also need the output to be in order by `cmp_unshuffle`, on a playlist that was just shuffled, where every entry has a known index. | Doc comment on `unshuffle` in `Mpv/Common/Playlist.lean`. |
| `playlist_get_first_in_next_playlist` (`common/playlist.c`) | No current entry, as after a playlist has ended and `playlist-prev-playlist` is issued, unless `playlist_get_next` returns NULL, or `direction > 0` and the entry it returns has no playlist path. | Reads `pl->current->playlist_path`: a NULL dereference. | Avoided: the generator does not produce that state, and the model describes mpv only outside it. | Doc comment on `getFirstInNextPlaylist` in `Mpv/Common/Playlist.lean`. |
| `calculate_frame_duration` (`player/video.c`) | `vo_chain` is `NULL`. | Reads `vo_chain->filter` without a test. Its one caller, `write_video`, returns on entry when `vo_chain` is `NULL`; whether the field can change between that test and the call was not traced. | Avoided: the generator sets `vo_chain` first. The model returns the state unchanged, which describes nothing mpv does. | Doc comment on `calculateFrameDuration` in `Mpv/Player/Video.lean`. |
| `adjust_audio_drift_compensation` (`player/video.c`) | `video_out` is `NULL`. | Reads `video_out->opts` without a test. | Out of the model: `video_sync` is a plain field of the model's context, and the harness always has a `vo`. | Doc comment on `adjustAudioDriftCompensation` in `Mpv/Player/Video.lean`. |
| `get_current_chapter` (`player/playloop.c`) | `last_chapter_seek` at or above `num_chapters`: a seek to chapter 4, then a `chapter-list` change or a demuxer chapter update that leaves 2 chapters. Neither resets the field. | Returns 4, an index at or above `get_chapter_count`, until a seek of another kind or a new file resets the field, or a new chapter seek overwrites it with a valid chapter. Possibly a bug in mpv. | Modelled as is: the model returns the same value and the generator produces the state. | Doc comment on `getCurrentChapter_bounds` in `Mpv/Player/PlayloopTheorems.lean`; `getCurrentChapter_lt_count_iff` is the unconditional fact. |

## Modelling conventions

- A C `int` is a Lean `Int`, so overflow is out of the model.
- A C `double` is a Lean `Float`: both are IEEE-754 binary64. A definition
  performs the C function's operations in the C function's order, with IEEE
  arithmetic as the C source reads, and writes `MPMAX` and `MPCLAMP` as the
  macros expand, because they are not symmetric for NaN. `MP_NOPTS_VALUE` is
  the same bit pattern on both sides, and the `playloop` script checks it
  against mpv's own.
- The fidelity check compares floats by bit pattern. A float crosses the
  script and the output as the 16 hex digits of its bit pattern
  (`Float.toBits`), so both sides start from the identical value, and a
  difference in the last bit of a result is a divergence. There is no
  tolerance. Two things are not compared:
  - The sign of a zero result. mpv is built with `-fno-signed-zeros`, which
    lets the compiler treat `-0.0` and `+0.0` as one value, so mpv's source
    does not decide which of the two a zero result is. In the pinned build
    `get_start_time` returns `-0.0` for a start time of `-0.0`, where the
    source's `0 + start_time` gives `+0.0` in IEEE arithmetic and so does the
    model. Both sides print every zero as `+0.0`. Inputs still include `-0.0`.
    Where the model or a theorem gives a result of `0`, what is claimed of mpv
    is a zero of either sign.
  - The sign and payload of a NaN. Lean's `Float.ofBits` and `Float.toBits`
    keep a single NaN, so the harness prints every NaN as `7ff8000000000000`.
    Whether a result is NaN is compared.
- Lean has no arithmetic laws for `Float`; its operations are opaque to the
  proof checker. So no theorem says what a sum, product or comparison of two
  times is. The theorems about float code state its case structure: which
  branch runs, which input the result is, and the integer facts such as
  chapter-index bounds. What the arithmetic computes rests on the fidelity
  check alone.
- A C `float` is a Lean `Float32`, and crosses the script as 8 hex digits,
  with every NaN printed as `7fc00000` and every zero as `00000000`. C mixes the two widths: the gain
  functions in `player/audio.c` keep a `float` but divide and call `pow` in
  `double`. A definition converts with `Float32.toFloat` and `Float.toFloat32`
  at exactly the points where C promotes or stores.
- Three results come from the C math library. `Float.pow` and `Float.abs`
  compile to `pow` and `fabs`, the calls mpv makes. Lean has no `rint`, so
  `Video.rint` builds round-to-nearest-even from `Float.floor`, and the `video`
  script compares it with libm's `rint` directly, except for the sign of a
  zero result. `pow` is not correctly
  rounded and may differ between C libraries, so `db_gain`,
  `compute_replaygain` and `audio_get_gain` are checked against the `pow` of
  the machine that runs `mpv-fidelity`, with both sides calling the same one.
  `rint` is checked in the default rounding mode only.
- A C `int64_t` converted to `double` is `Float.ofInt`, which rounds to
  nearest like the C conversion for every `int64_t` value.
- Memory allocation (`talloc`) and log output are not modelled. Paths that only
  do these are marked `unobservable`. A reference count that decides when an
  object is freed is state: `playlist_entry_unref` returns `none` when it frees
  the entry.
- A callee in a file the model does not cover yet is a parameter of the
  definition, as `mp_normalize_path` is for `entryNew`. The oracle supplies a
  stand-in that is right for the inputs the generator emits.
- A callee whose result the test must control, such as the random number
  generator or the demuxer, is replaced in the harness with `--wrap`. The
  oracle passes the model the same replacement.
- A libc routine is a parameter too. `qsort` has a defined result only when the
  comparison is a consistent total order, so the theorems about
  `playlist_unshuffle` assume that result and the generator sorts only such
  arrays.
- `mp_assert` conditions become theorem hypotheses.
