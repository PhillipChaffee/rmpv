# Formal Coverage Honesty and Player Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `mpv-cover`'s path counts see short-circuit and macro branches, run fidelity harnesses against a real mpv build, finish `common/playlist.c`, and model the first two slices of the player core.

**Architecture:** `formal/` holds a Lean 4 model of mpv (`Mpv/`), executable oracles (`Oracle/`), and two Rust tools. `mpv-cover` parses mpv's C source into functions and paths and matches them to `-- mpv-cover:` claims in the Lean files. `mpv-fidelity` compiles real mpv code into a harness, runs it and the Lean oracle on one random script, compares outputs, then plants bugs in the C and requires each to be caught. This plan adds a real meson build of mpv that both tools read from.

**Tech Stack:** Lean 4.36.0 (core only, no Mathlib), Rust 1.97 (edition 2024), tree-sitter-c, meson and ninja, gcc, FFmpeg 6.1 from apt, libplacebo built from source.

**Spec:** The section below. Read `formal/README.md` first for how claims, paths and targets work today.

## Spec

1. **Short-circuit paths.** The right operand of every `&&` and `||` is its own path.
2. **Real mpv build.** One command builds mpv at the pinned commit as a static library with a compilation database. It is the single source of compile flags for both tools.
3. **Preprocessed parsing.** `mpv-cover` counts paths on preprocessed source, so branches inside macros count. Coverage is still shown against original source lines.
4. **Unobservable paths.** A claim can mark a path as existing in C but invisible at the model's level of abstraction. These are reported separately, never silently.
5. **Playlist fidelity on the real build.** The playlist target uses real mpv headers and objects, with no stub headers.
6. **`common/playlist.c` complete.** Every function is modelled, claimed, exercised by the fidelity check, and has a planted bug that is caught.
7. **Player core, first slices.** Playback time and position queries from `player/playloop.c`, then the numeric A/V sync helpers from `player/video.c` and `player/audio.c`, each modelled, claimed, checked, and with planted bugs caught.

Out of scope: the orchestration functions in the player core (`mp_seek`, `write_video`, `fill_audio_out_buffers`, `run_playloop` and the like). They need a harness that boots a whole `MPContext`, which is a later plan. No media player code is written here.

## Global Constraints

- mpv is pinned in `formal/mpv.lock.toml` at `d11ffba253def00b5172c4b8b47e6b2eb765cae1`. Never edit files under `formal/.mpv`.
- `make ci` must pass at the end of every task. It runs lint, tests, `lake build --wfail`, `mpv-cover check`, and `mpv-fidelity`.
- Lean is at `~/.local/lean/bin`. Put it on `PATH` before running `make` or `lake`.
- Lean files may not contain `sorry`, `admit`, `axiom`, `native_decide`, `implemented_by`, `extern`, or `unsafe`. Lean core only: do not add Mathlib or Batteries.
- Rust lints are strict (`clippy::pedantic`, `nursery`, `unwrap_used`, `expect_used`, `indexing_slicing`, `panic` are errors outside tests). Do not add `#[allow]`; use `#[expect(..., reason = "...")]` only when the lint is wrong.
- No new Rust dependencies without a stated reason in the commit message. `serde_json` is approved for Task 3.
- A claim is only honest if the Lean definition accounts for that path. When a path is not modelled, list path ids explicitly instead of `paths=all`, or mark it `unobservable` under the rules in Task 3.
- Never weaken a theorem, a comparison, or a mutation to get green. If the model and mpv diverge, the model is wrong until shown otherwise. If you believe mpv has a bug, stop and report it.
- Commit as the configured git user. Do not add `Co-Authored-By` lines, session links, or any mention of an AI tool. Do not push.
- Comments explain a non-obvious why. No comments that narrate what the code does.
- Modelling conventions: a C `int` is a Lean `Int`; `mp_assert` conditions become theorem hypotheses and generator preconditions; memory management and log output are not modelled.

## Review Focus

1. **A claim says `paths=all` after preprocessing adds macro paths the model ignores.** Expected: every existing claim is re-read against `show` output in Task 3, not blindly re-stamped.
2. **A mutation survives because the generator never reaches that branch.** Expected: the task widens the generator. Deleting the mutation is not allowed.
3. **A file in scope is not compiled in the Linux build.** Expected: it is still counted, parsed without preprocessing, and the report says how many such files exist.
4. **Floating-point results differ in the last bit between Lean and C.** Expected: outputs are compared as IEEE-754 bit patterns, and any difference is investigated as an evaluation-order mismatch in the model, not tolerated with an epsilon.
5. **The mpv build is missing.** Expected: both tools stop with a message naming `make mpv-build`. Neither falls back to different numbers.

---

### Task 1: Count `&&` and `||` as paths

**Files:**
- Modify: `formal/cover/src/source.rs` (`collect_arms`, tests module)
- Modify: `formal/README.md` (definition of a path, blind-spot list)

**Interfaces:**
- Produces: `Arm.kind` gains the values `"&&"` and `"||"`. Path ids of existing functions shift where a condition has a short-circuit.

- [ ] **Step 1: Write the failing test** in `source.rs`:

```rust
#[test]
fn short_circuit_operands_are_paths() {
    let text = "int f(int a, int b) { if (a && b) return 1; return a || b ? 2 : 3; }\n";
    let file = SourceFile::parse("f.c".to_owned(), text.to_owned()).expect("parses");
    let kinds: Vec<&str> = file.functions.first().expect("one function").arms.iter().map(|arm| arm.kind).collect();
    assert_eq!(kinds, ["body", "&&", "if", "||", "?", ":"]);
}
```

- [ ] **Step 2: Run** `cargo test -p mpv-cover short_circuit` and confirm it fails on the kinds.
- [ ] **Step 3: Implement.** In `collect_arms`, a `binary_expression` whose `operator` field has kind `&&` or `||` adds an arm for its `right` field.
- [ ] **Step 4: Run** `cargo test -p mpv-cover`. Expected: pass.
- [ ] **Step 5: Re-check claims.** Run `mpv-cover show common/playlist.c <fn>` for each of the ten claimed functions. A short-circuit operand in a condition the Lean definition evaluates as a whole is modelled, so `paths=all` stays. Hashes do not change.
- [ ] **Step 6: Update `formal/README.md`.** Add short-circuit operands to the definition of a path and remove them from the blind-spot list.
- [ ] **Step 7: Run** `make ci`. Record the new in-scope path total in the commit message. Commit.

---

### Task 2: Real mpv build

**Files:**
- Create: `formal/build-mpv.sh`
- Modify: `formal/mpv.lock.toml` (add a `[libplacebo]` table with `repo` and `commit`)
- Modify: `Makefile` (`mpv-build` target; `proofs` and `cover` depend on it)
- Modify: `.gitignore` (`/formal/.deps`, `/formal/.mpv-build`)
- Modify: `.github/workflows/ci.yml` (apt packages)
- Modify: `formal/README.md` (setup section)

**Interfaces:**
- Produces, all under `formal/.mpv-build/`:
  - `compile_commands.json`, with one entry per compiled mpv C file.
  - `libmpv.a`, a static library of all mpv objects.
  - `harness.rsp`, a compiler response file with every `-I` and `-D` flag needed to compile a C file that includes mpv headers or an mpv `.c` file.
  - `harness-link.rsp`, a response file with `libmpv.a` and every library it needs, in link order.
  - Use: `cc @formal/.mpv-build/harness.rsp x.c @formal/.mpv-build/harness-link.rsp -o x`.

mpv at this commit needs libplacebo `>=7.387.0` and meson `>=1.3.0`. Ubuntu 24.04 ships libplacebo 6.338, so libplacebo is built from source into `formal/.deps`. FFmpeg 6.1 and libass from apt satisfy mpv's minimums.

- [ ] **Step 1: Write the acceptance check first**, as the last block of `formal/build-mpv.sh`: it writes a C file containing `#include "common/playlist.c"` and a `main` that appends one file to a new playlist and returns non-zero unless `num_entries == 1`, compiles it with the two response files, and runs it. The script exits non-zero if that fails.
- [ ] **Step 2: Write the build.** The script is idempotent and skips finished stages. Stages: install or verify apt packages (`meson`, `libavcodec-dev`, `libavfilter-dev`, `libavformat-dev`, `libavutil-dev`, `libswresample-dev`, `libswscale-dev`, `libass-dev`, plus whatever libplacebo needs); build libplacebo at the locked commit as a static library with every GPU backend, demo and test disabled; configure mpv with `-Dlibmpv=true -Ddefault_library=static -Dcplayer=false` and optional features off unless the configure step requires them; build; write the two response files.
- [ ] **Step 3: Run** `make mpv-build` from a clean state. Expected: exit 0 and the acceptance program's output.
- [ ] **Step 4: Run it again.** Expected: finishes in seconds, no rebuild.
- [ ] **Step 5: Update CI and README** with the apt package list and the new target.
- [ ] **Step 6: Run** `make ci`. Commit.

**Stop condition:** if libplacebo at a version mpv accepts cannot be built here after a real attempt, do not stub or patch mpv. Report BLOCKED with the failing command and its output.

---

### Task 3: Parse preprocessed source and add unobservable paths

**Files:**
- Create: `formal/cover/src/preprocess.rs` (compilation database, running `cc -E`, line mapping)
- Modify: `formal/cover/src/source.rs` (build functions from preprocessed text, hash from original lines)
- Modify: `formal/cover/src/claims.rs` (`unobservable=` field)
- Modify: `formal/cover/src/main.rs` (`--build` flag, report lines, diff marker)
- Modify: `formal/cover/Cargo.toml` (`serde_json`)
- Modify: `formal/Mpv/Common/Playlist.lean` (re-stamped and re-reviewed claims)
- Modify: `formal/README.md`

**Interfaces:**
- Consumes: `formal/.mpv-build/compile_commands.json` from Task 2.
- Produces:
  - `preprocess::LineMap::parse(preprocessed: &str) -> LineMap` and `LineMap::origin(&self, row: usize) -> Option<(&str, usize)>`, giving the original file and zero-based line for a row of preprocessed text, from GCC linemarkers (`# 12 "file.c" 2`).
  - `mpv-cover --build <dir>`, default `<formal>/.mpv-build`.
  - Claim field `unobservable=<ids>`, for example `paths=0,1,2 unobservable=3,4`. A path may not be in both lists. `paths=all` with `unobservable=` means all the others.
  - A function's hash is the SHA-256, first 12 hex characters, of the whitespace-normalized original source lines from its first to its last line. This holds in both parsing modes.

Rules:
- A file with a compilation database entry is preprocessed with that entry's flags (`-E`, without `-c`, `-o` or dependency-file flags), and only functions whose definition starts in that file are kept. An arm that comes from a macro is attributed to the line of the macro invocation.
- A file without an entry (headers, files not built on Linux) is parsed raw as today. The report states how many in-scope C files that is.
- If `compile_commands.json` is missing, stop with an error that names `make mpv-build`.
- Preprocess files in parallel with `std::thread::scope`. `mpv-cover report` should finish in under 60 seconds here; say so in the commit message with the measured time.
- `unobservable` is allowed only for a path whose sole effect is memory management or log output. The report prints the total of unobservable paths on its own line, they are not added to `covered`, and `diff` prints their lines with a `~` prefix. `check --full` passes when every path is covered or unobservable.

- [ ] **Step 1: Write failing tests.**

```rust
#[test]
fn linemarkers_map_rows_to_original_lines() {
    let map = LineMap::parse("# 1 \"a.c\"\nint x;\n# 10 \"b.h\" 1\nint y;\n# 3 \"a.c\" 2\nint z;\n");
    assert_eq!(map.origin(1), Some(("a.c", 0)));
    assert_eq!(map.origin(3), Some(("b.h", 9)));
    assert_eq!(map.origin(5), Some(("a.c", 2)));
}
```

In `claims.rs`: a claim `paths=0,1 unobservable=2` parses to both sets, and `paths=0,1 unobservable=1` is a problem.

In `source.rs`, using `cc -E` on a temp file: for `#define PICK(a, b) ((a) > (b) ? (a) : (b))` and `int f(int a, int b) {\n    return PICK(a, b);\n}`, the function has arm kinds `["body", "?", ":"]` and both branch arms start on original line 1.

- [ ] **Step 2: Run** `cargo test -p mpv-cover`. Expected: the new tests fail.
- [ ] **Step 3: Implement** `preprocess.rs`, then the preprocessed path through `SourceFile`, then `unobservable`.
- [ ] **Step 4: Run** `cargo test -p mpv-cover`. Expected: pass.
- [ ] **Step 5: Re-review all ten playlist claims.** For each, run `show`, read every path, and write the claim again with the new hash. A macro branch the model accounts for stays claimed. An allocation-growth branch becomes `unobservable`. Anything else the model does not account for is left out of `paths=`, which makes the function partial.
- [ ] **Step 6: Update the README.** Document preprocessing, `unobservable`, and the revised blind-spot list.
- [ ] **Step 7: Run** `make ci`. Put the before and after totals (functions, paths, covered, unobservable) in the commit message. Commit.

---

### Task 4: Move the playlist fidelity target onto the real build

**Files:**
- Modify: `formal/fidelity/src/main.rs` (`Run::build`, `--build` flag)
- Modify: `formal/fidelity/targets/playlist/harness.c` (delete the two stub function definitions)
- Modify: `formal/fidelity/targets/playlist/target.toml` (delete `sources`)
- Delete: `formal/fidelity/targets/playlist/stubs/`
- Modify: `formal/cover/src/main.rs` (if `FidelityTarget` needs to follow the manifest change)
- Modify: `formal/README.md`

**Interfaces:**
- Consumes: `harness.rsp` and `harness-link.rsp` from Task 2.
- Produces: `target.toml` has `oracle`, `file`, optional `link_args` (extra linker arguments such as `-Wl,--wrap=name`), and `[[mutation]]` entries. A harness is `harness.c` alone, compiled as `cc @harness.rsp [-I mutant dirs] harness.c @harness-link.rsp <link_args>`. `mpv-fidelity --build <dir>` defaults to `<formal>/.mpv-build`.

- [ ] **Step 1: Change the build command and manifest**, delete the stubs, and make a missing build directory an error that names `make mpv-build`.
- [ ] **Step 2: Run** `cargo run -q --release -p mpv-fidelity`. Expected: `playlist: model and mpv agree on 205000 operations` and `caught 15 of 15 planted bugs`.
- [ ] **Step 3: Prove the check still bites.** Temporarily change `getNext` in the Lean model to return the wrong entry, rebuild, and confirm a divergence is printed. Revert.
- [ ] **Step 4: Run** `make ci`. Commit.

---

### Task 5: Finish `common/playlist.c`, part one

Functions: `playlist_entry_new`, `playlist_entry_add_param`, `playlist_entry_add_params`, `playlist_set_params`, `playlist_entry_unref`, `playlist_clear`, `playlist_clear_except_current`, `playlist_move`, `playlist_append_file`, `playlist_populate_playlist_path`, `playlist_set_stream_flags`, `playlist_set_current`.

**Files:**
- Modify: `formal/Mpv/Common/Playlist.lean`, `formal/Oracle/Playlist.lean`
- Modify: `formal/fidelity/targets/playlist/harness.c`, `target.toml`
- Modify: `formal/fidelity/src/playlist.rs`

**Interfaces:**
- Consumes: the claim syntax from Task 3 and the target layout from Task 4.
- Produces: `Entry` gains the fields these functions read or write (`filename`, `params`, `playlistPath`, `streamFlags`, `reserved`, and others as needed); `Playlist` gains `playlistDir`. The oracle and harness state line prints every modelled field of every entry. Task 6 builds on these definitions.

Read each C function before modelling it. For each one: add the Lean definition with its claim, add a script operation to the oracle and the harness, teach the generator to emit it within the function's asserted preconditions, and add at least one mutation.

- [ ] **Step 1: Add the mutations first**, one or more per function, to `target.toml`. Run `mpv-fidelity`. Expected: the new mutations are reported as SURVIVED, because nothing exercises them yet.
- [ ] **Step 2: Extend the model, oracle, harness and generator** function by function until `mpv-fidelity` reports zero divergence and every mutation caught.
- [ ] **Step 3: Prove these theorems** in `Playlist.lean`, adjusting statements only to match what the C code really does:
  - `move_wf`: `playlist_move` preserves `WF`, given the entries are live.
  - `move_perm`: `playlist_move` keeps the same entries (`List.Perm`).
  - `clear_empty`: after `playlist_clear` there are no entries and `current` is `none`.
  - `clearExceptCurrent_wf`: `WF` is preserved, and the result has at most one entry.
  - `setCurrent` theorems for each case the C function distinguishes.
- [ ] **Step 4: Run** `mpv-cover diff common/playlist.c --uncovered`. Expected: none of this task's functions appear.
- [ ] **Step 5: Run** `make ci`. Commit.

---

### Task 6: Finish `common/playlist.c`, part two

Functions: `playlist_shuffle`, `cmp_unshuffle`, `playlist_unshuffle`, `playlist_get_first_in_next_playlist`, `playlist_get_first_in_same_playlist`, `playlist_transfer_entries_to`, `playlist_transfer_entries`, `playlist_append_entries`, `playlist_parse_file`.

**Files:** as Task 5.

**Interfaces:**
- Consumes: the `Entry` and `Playlist` definitions from Task 5, and `link_args` from Task 4.
- Produces: `common/playlist.c` at 100% covered or unobservable, and every function checked.

Decisions:
- **Randomness.** The Lean `shuffle` takes the random picks as an argument. The harness replaces the random function `playlist_shuffle` calls using `-Wl,--wrap=<that function>` in `link_args`, and feeds it the picks from the script.
- **Two playlists.** The transfer functions take a source playlist. The harness and oracle hold a second playlist, with script operations to fill it.
- **`playlist_parse_file`.** The demuxer is outside the model. The Lean function takes the demuxer's result (no demuxer, a demuxer without a playlist, or a demuxer with a playlist and a file type) as an argument. The harness wraps `demux_open_url` and `demux_free` with `--wrap` to return that result. Arms that only log are `unobservable`.

- [ ] **Step 1: Add the mutations first** and confirm they SURVIVE.
- [ ] **Step 2: Extend the model, oracle, harness and generator** until there is zero divergence and every mutation is caught.
- [ ] **Step 3: Prove:**
  - `shuffle_perm` and `unshuffle_perm`: both keep the same entries.
  - `shuffle_wf`, `unshuffle_wf`, `transferEntriesTo_wf`: `WF` is preserved.
  - `unshuffle_shuffle`: unshuffling a shuffled playlist restores the original order, under whatever precondition the C code needs. If the C code does not guarantee this, state and prove what it does guarantee, and say so in the commit message.
  - `transferEntries_count`: the destination grows by the source's count and the source ends empty.
- [ ] **Step 4: Run** `mpv-cover diff common/playlist.c --uncovered`. Expected: no output.
- [ ] **Step 5: Run** `make ci`. Commit.

---

### Task 7: Player core slice one, playback time and position

Functions in `player/playloop.c`: `get_time_length`, `get_start_time`, `get_current_time`, `get_playback_time`, `get_current_pos_ratio`, `get_current_chapter`, `chapter_start_time`, `get_chapter_count`, `get_cache_buffering_percentage`.

**Files:**
- Create: `formal/Mpv/Player/Playloop.lean`, `formal/Oracle/Playloop.lean`
- Create: `formal/fidelity/targets/playloop/harness.c`, `target.toml`
- Create: `formal/fidelity/src/playloop.rs`
- Modify: `formal/Mpv.lean`, `formal/lakefile.toml` (`playloop-oracle`), `formal/fidelity/src/main.rs` (`TARGETS`)

**Interfaces:**
- Consumes: the target layout from Task 4.
- Produces: `Mpv.Player.Playloop.Ctx`, a structure with exactly the `MPContext`, option and demuxer fields these functions read. Task 8 may extend it.

Decisions:
- **Numbers.** C `double` is Lean `Float`, and the model performs the same operations in the same order as the C. The oracle prints floats as their IEEE-754 bit pattern (`Float.toBits`), and the harness prints the same bits. `MP_NOPTS_VALUE` is the same sentinel constant on both sides.
- **Harness.** It includes `player/playloop.c`, builds a zero-initialized `MPContext` with fake demuxer, option and chapter data taken from the script, and calls the real functions. It does not start a player.
- **Generator.** It favours boundary values: the sentinel, zero, negative times, a position equal to the length, zero chapters, a position before the first chapter and after the last.
- **Theorems.** Lean cannot prove arithmetic facts about `Float`. Prove the case structure instead, for example which branch is taken when there is no demuxer or the length is not positive, and the chapter-index bounds, which are integer facts. State this limit in the README.

- [ ] **Step 1: Create the target with mutations only**, at least one per function, and a harness that compiles. Expected: `mpv-fidelity` reports them SURVIVED.
- [ ] **Step 2: Write the model, oracle, harness operations and generator** until there is zero divergence and every mutation is caught.
- [ ] **Step 3: Prove** at least one theorem per function about its case structure, and for `get_current_chapter` that the result is at least `-2` and less than the chapter count.
- [ ] **Step 4: Run** `mpv-cover diff player/playloop.c` and confirm the nine functions are covered and checked.
- [ ] **Step 5: Update the README** (float convention, list of targets). Run `make ci`. Commit.

---

### Task 8: Player core slice two, A/V sync arithmetic

Functions in `player/video.c`: `calc_average_frame_duration`, `calc_best_speed`, `find_best_speed`, `adjust_sync`, `update_av_diff`, `calculate_frame_duration`, `using_spdif_passthrough`, `adjust_audio_drift_compensation`. In `player/audio.c`: `db_gain`, `compute_replaygain`, `audio_get_gain`, `written_audio_pts`, `playing_audio_pts`.

**Files:**
- Create: `formal/Mpv/Player/Video.lean`, `formal/Mpv/Player/Audio.lean`, `formal/Oracle/AvSync.lean`
- Create: `formal/fidelity/targets/avsync/harness.c`, `target.toml`
- Create: `formal/fidelity/src/avsync.rs`
- Modify: `formal/Mpv.lean`, `formal/lakefile.toml`, `formal/fidelity/src/main.rs`

**Interfaces:**
- Consumes: the float convention and `Ctx` from Task 7. A target's `target.toml` names one `file`; if one harness cannot include both C files, make two targets, `video` and `audio`.

Decisions: as Task 7. Where a function reads the audio or video output (for example a delay or a queued-sample count), that value is an input in the model, and the harness supplies it by wrapping the called function with `--wrap`.

- [ ] **Step 1: Create the target or targets with mutations only.** Expected: SURVIVED.
- [ ] **Step 2: Write the models, oracle, harness and generator** until there is zero divergence and every mutation is caught.
- [ ] **Step 3: Prove** at least one case-structure theorem per function.
- [ ] **Step 4: Run** `mpv-cover report` and `mpv-cover diff player/video.c player/audio.c`. Confirm the thirteen functions are covered and checked.
- [ ] **Step 5: Update the README.** Run `make ci`. Put the final report totals in the commit message. Commit.

If a function in this list turns out to need a booted player to call at all, leave it unclaimed, name it in the commit message with the reason, and continue with the rest.
