# Pilot: a Rust port of mpv's `misc/` for Aeneas

This pilot ports mpv's `misc/` folder (commit pinned in
`formal/mpv.lock.toml`) to safe Rust, checks the port against mpv's compiled
C by differential testing, translates it to Lean 4 with Charon and Aeneas,
and compares it with the machine translation made by c2rust. It is the Rust
half of the plan in `/home/user/achates/docs/pilot.md`: Achates turns each C
function into Lean, Aeneas turns the matching Rust function into Lean, and a
theorem says the two are equal. No equality is proved here.

## Results in short

| | Count |
| --- | --- |
| mpv functions in `misc/` (15 compiled files; `jni.c` is Android-only and not in this build) | 141 |
| Ported to safe Rust | 141 |
| Differentially tested against the C | 140 (all except `resize_append`) |
| Tested functions that agree with the C on every input | 140 |
| Generated inputs, sequences and rounds run in total (115 tests, 150 named runs) | 13,429,631 |
| Threaded properties whose schedules loom explores (14 models) | see "Concurrency: explored and sampled" |
| Aeneas body, default translation | 98 |
| Aeneas body, `-loops-to-rec` translation | 101 |
| ... of those 101, bodies that reach no axiom | 77 |

Both Lean translations type-check with `autoImplicit = false` and hold no
`sorry`. The 36 functions ported last are the three variadic ones of
`bstr.c` (a `printf` and a `scanf` engine that follow glibc 2.39), all of
`dispatch.c` and `thread_pool.c`, and `mp_cancel`. The threaded ones are
outside what Aeneas translates; loom explores their schedules instead.

Where the work is:

| Path | Contents |
| --- | --- |
| `crates/misc` | The port (`rmpv-misc`), `#![forbid(unsafe_code)]`. Its `README.md` holds the C-to-Rust conventions. `tests/loom.rs` holds the loom models of the threaded ports. |
| `crates/misc-difftest` | FFI to `libmpv.a`, shims that reach static C functions and call variadic ones, and the differential tests. `tools/mutate.py` plants 34 bugs in the port and checks that the tests catch each. |
| `pilot/misc-lean` | `translate.sh` (Charon and Aeneas), both generated Lean files, the Lake project, and `status.py`, which prints the table below. |
| `pilot/misc-c2rust` | Copy of the c2rust crate from `/home/user/spike-c2rust/misc-c2rust`, outside the workspace. |

## How to rerun

```bash
make difftest                      # builds formal/.mpv-build first; fails without it
make loom                          # the loom models, about 30 seconds with the build
crates/misc-difftest/tools/mutate.py    # about 15 minutes
pilot/misc-lean/translate.sh       # needs the Aeneas release, see pilot/misc-lean/README.md
(cd pilot/misc-lean && PATH=$PATH:$HOME/.local/lean/bin lake build)
pilot/misc-lean/status.py DIFFTEST_OUTPUT   # prints the per-function table
```

`make test` (`cargo test --workspace`) also runs the differential tests when
`formal/.mpv-build` exists, and compiles them out when it does not.
`make lint`, `make test`, `make difftest` and `make loom` pass. `make
difftest` and `make loom` run under `timeout`, and each threaded test
stops itself with a failure if it has not finished in time.

## Status of each function

Columns:

- **Ported**: the Rust item, or why there is none.
- **Differential test**: generated inputs on which the C and the port gave
  the same result. A test stops at the first difference, so "all agree"
  means every input of the run.
- **Aeneas**: *body* is a Lean `def` with the translated body. *axiom* is a
  declaration without a body. *left out* means the function was excluded
  from Charon's input; *rejected* means Aeneas could not translate it and it
  was then excluded. A *body* can still call an axiom: see "What a body
  rests on" below.

Fewer than 100,000 inputs, and why: `bstr_parse_utf8_code_length`, `h_to_i`
and the `ctype.h` helpers take one byte (all 256 values run). `ucdLookup` is
run on every code point 0 to U+10FFFF (1,114,112). `node_init` has 21 valid
(format, parent) pairs. `mp_getcwd` has 9 combinations of `$PWD` state.
`mp_hash_to_bstr`, `mp_normalize_path`, `mp_path_exists` and `mp_path_isdir`
run 20,000 inputs and `mp_mkostemps` 5,000, `mp_mkdirp` and
`mp_save_to_file` 2,000: each input touches the file system. The two
thread tests run 2,000 rounds, each starting two threads. For the 36
functions ported last, see their sections: the threaded ones count
sequences of operations or rounds of a threaded test, each of which runs
many calls.

### `misc/bstr.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `bstrcmp` | `bstr::bstrcmp` | 100,000 inputs, all agree | body | body |
| `bstrcasecmp` | `bstr::bstrcasecmp` | 100,000 inputs, all agree | body | body |
| `bstrchr` | `bstr::bstrchr` | 100,000 inputs, all agree | body | body |
| `bstrrchr` | `bstr::bstrrchr` | 100,000 inputs, all agree | body | body |
| `bstrcspn` | `bstr::bstrcspn` | 100,000 inputs, all agree | body | body |
| `bstrspn` | `bstr::bstrspn` | 100,000 inputs, all agree | body | body |
| `bstr_find` | `bstr::bstr_find` | 100,000 inputs, all agree | body | body |
| `bstr_lstrip` | `bstr::bstr_lstrip` | 100,000 inputs, all agree | body | body |
| `bstr_strip` | `bstr::bstr_strip` | 100,000 inputs, all agree | body | body |
| `bstr_split` | `bstr::bstr_split` | 100,000 inputs, all agree | body | body |
| `bstr_split_tok` | `bstr::bstr_split_tok` | 100,000 inputs, all agree | body | body |
| `bstr_splice` | `bstr::bstr_splice` | 100,000 inputs, all agree | body | body |
| `bstrtoll` | `bstr::bstrtoll` | 100,000 inputs, all agree | body | body |
| `bstrtod` | `bstr::bstrtod` | 100,000 inputs, all agree | left out: `f64` in the signature | left out: `f64` in the signature |
| `bstr_splitchar` | `bstr::bstr_splitchar` | 100,000 inputs, all agree | body | body |
| `bstr_strip_linebreaks` | `bstr::bstr_strip_linebreaks` | 100,000 inputs, all agree | body | body |
| `bstr_eatstart` | `bstr::bstr_eatstart` | 100,000 inputs, all agree | body | body |
| `bstr_eatend` | `bstr::bstr_eatend` | 100,000 inputs, all agree | body | body |
| `bstr_lower` | `bstr::bstr_lower` | 100,000 inputs, all agree | body | body |
| `bstr_sscanf` | `bstr::bstr_sscanf`, on `cscanf::vsscanf` | 100,000 inputs with mpv's callers' formats and 50,000 generated, all agree; `vsscanf` against glibc on 100,000 more | left out: `f32` and `f64` in `ScanArg` | left out: as default |
| `bstr_parse_utf8_code_length` | `bstr::bstr_parse_utf8_code_length` | 256 inputs, all agree | body | body |
| `bstr_decode_partial_utf8` | `bstr::bstr_decode_partial_utf8` | 100,000 inputs, all agree | body | body |
| `bstr_decode_utf8` | `bstr::bstr_decode_utf8` | 100,000 inputs, all agree | body | body |
| `bstr_split_utf8` | `bstr::bstr_split_utf8` | 100,000 inputs, all agree | body | body |
| `bstr_validate_utf8` | `bstr::bstr_validate_utf8` | 100,000 inputs, all agree | body | body |
| `bstr_sanitize_utf8_latin1` | `bstr::bstr_sanitize_utf8_latin1` | 100,000 inputs, all agree | body | body |
| `resize_append` | `bstr::resize_append` | not tested: it only grows capacity, which no caller can observe | body | body |
| `bstr_xappend` | `bstr::bstr_xappend` | 100,000 inputs, all agree | body | body |
| `bstr_xappend_asprintf` | `bstr::bstr_xappend_asprintf`, on `cprintf::vsnprintf` | 50,000 append sequences, all agree; `vsnprintf` against glibc on 205,000 more | body | body |
| `bstr_xappend_vasprintf` | `bstr::bstr_xappend_vasprintf` | 50,000 append sequences (same test), all agree | body | body |
| `bstr_case_startswith` | `bstr::bstr_case_startswith` | 100,000 inputs, all agree | body | body |
| `bstr_case_endswith` | `bstr::bstr_case_endswith` | 100,000 inputs, all agree | body | body |
| `h_to_i` | `bstr::h_to_i` | 256 inputs, all agree | body | body |
| `bstr_decode_hex` | `bstr::bstr_decode_hex` | 100,000 inputs, all agree | body | body |
| `bstr_find_in_list0` | `bstr::bstr_find_in_list0` | 100,000 inputs, all agree | body | body |
| `bstr_in_list0` | `bstr::bstr_in_list0` | 100,000 inputs, all agree | body | body |

### `misc/charset_conv.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `mp_charset_is_utf8` | `charset_conv::mp_charset_is_utf8` | 100,000 inputs, all agree | body | body |
| `mp_charset_is_utf16` | `charset_conv::mp_charset_is_utf16` | 100,000 inputs, all agree | body | body |
| `ms_bom_guess` | `charset_conv::ms_bom_guess` | 100,000 inputs, all agree | body | body |
| `mp_charset_guess` | `charset_conv::mp_charset_guess` | 100,000 inputs, all agree | body | body |
| `mp_iconv_to_utf8` | `charset_conv::mp_iconv_to_utf8` | 100,000 inputs, all agree | body | body |

The variadic functions take a typed argument list: `printf` arguments as
`cprintf::CArg`, each C argument after the default promotions (`Int`,
`UInt`, `Long`, `ULong`, `Double` as bits, `Str`, `Ptr`), and `scanf`
pointers as `cscanf::ScanArg` (`&mut` of the pointed-to type). The two
engines interpret the format as glibc 2.39 does in the "C" locale,
statement by statement after `vfprintf-internal.c`,
`vfprintf-process-arg.c`, `printf_fp.c`, `printf_fphex.c` and
`vfscanf-internal.c`. `vsnprintf` covers every flag (`-+ #0'`), widths and
precisions inline and by `*`, every length modifier glibc accepts with each
conversion (`hh h l ll L q j z Z t`), and `d i u o x X b B c s p f F e E g
G a A %`; `vsscanf` covers `*`, widths, the same modifiers, and `d i u o x X
b f F e E g G a A s c [ n %`. mpv's callers use `%d %u %x %f %lf %n` with
`bstr_sscanf`, and `bstr_xappend_vasprintf` carries every `mp_msg` and
FFmpeg log format, which is why the engine covers the whole language.
What they do not model they refuse with an error, listed under "Where the
port differs from the C".

The differential tests compare `vsnprintf` with glibc's `snprintf` on
100,000 generated formats of up to four conversions, 100,000 of float
conversions alone, 5,000 with precisions of 760 to 1,500 digits, 16 at the
`INT_MAX` limits and 5 with a `*` of `INT_MIN`; `vsscanf` with glibc's
`sscanf` (`__isoc23_sscanf`, as mpv links it) on 100,000 generated formats
and inputs, comparing the result and every argument buffer; and `strtof`,
which `%f` into a `float` needs, with glibc's on 100,000 strings. The
inputs are biased to limits, halfway points of rounding, `0x` and `0b`
prefixes, `nan` and `inf` spellings cut short, and unterminated sets.

The pinned build has no iconv, so `mp_iconv_to_utf8` is the build's
fallback: it returns its input or a copy. The port matches that build.

### `misc/codepoint_width.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `ucdLookup` | `codepoint_width::ucd_lookup` | 1,114,112 inputs, all agree | body | body |
| `ucdGraphemeJoins` | `codepoint_width::ucd_grapheme_joins` | 100,000 inputs, all agree | body | body |
| `ucdGraphemeDone` | `codepoint_width::ucd_grapheme_done` | 100,000 inputs, all agree | body | body |
| `ucdToCharacterWidth` | `codepoint_width::ucd_to_character_width` | 100,000 inputs, all agree | body | body |
| `is_valid_partial_sequence` | `codepoint_width::is_valid_partial_sequence` | 100,000 inputs, all agree | body | body |
| `term_disp_width` | `codepoint_width::term_disp_width` | 100,000 inputs, all agree | body | body |

### `misc/dispatch.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `queue_dtor` | `Drop for dispatch::MpDispatchQueue` | 10,000 sequences, all agree; a non-empty queue is refused, as the C asserts | rejected: `dyn` closures, `Mutex`, `Condvar` | rejected: as default |
| `mp_dispatch_create` | `dispatch::mp_dispatch_create` | 10,000 sequences, all agree | rejected: as above | rejected: as above |
| `mp_dispatch_set_wakeup_fn` | `dispatch::mp_dispatch_set_wakeup_fn` | 10,000 sequences (half with a wakeup callback), all agree | rejected: as above | rejected: as above |
| `mp_dispatch_set_onlock_fn` | `dispatch::mp_dispatch_set_onlock_fn` | 200 handshake rounds, all agree | rejected: as above | rejected: as above |
| `mp_dispatch_append` | `dispatch::mp_dispatch_append` | 10,000 sequences, all agree | rejected: as above | rejected: as above |
| `mp_dispatch_enqueue` | `dispatch::mp_dispatch_enqueue` | 10,000 sequences, all agree; 300 stress rounds | rejected: as above | rejected: as above |
| `mp_dispatch_enqueue_autofree` | `dispatch::mp_dispatch_enqueue_autofree` | 10,000 sequences, all agree, including when the data is freed | rejected: as above | rejected: as above |
| `mp_dispatch_enqueue_notify` | `dispatch::mp_dispatch_enqueue_notify` | 10,000 sequences, all agree; 300 stress rounds | rejected: as above | rejected: as above |
| `mp_dispatch_cancel_fn` | `dispatch::mp_dispatch_cancel_fn` | 10,000 sequences, all agree | rejected: as above | rejected: as above |
| `mp_dispatch_run` | `dispatch::mp_dispatch_run` | 300 stress rounds, all agree | rejected: as above | rejected: as above |
| `mp_dispatch_queue_process` | `dispatch::mp_dispatch_queue_process` | 10,000 sequences, all agree; 300 timed cases | rejected: as above | rejected: as above |
| `mp_dispatch_interrupt` | `dispatch::mp_dispatch_interrupt` | 10,000 sequences, all agree | rejected: as above | rejected: as above |
| `mp_dispatch_adjust_timeout` | `dispatch::mp_dispatch_adjust_timeout` | 300 timed cases, all agree; called from callbacks in the 10,000 sequences | rejected: as above | rejected: as above |
| `mp_dispatch_lock` | `dispatch::mp_dispatch_lock` | 200 handshake rounds and 300 stress rounds, all agree | rejected: as above | rejected: as above |
| `mp_dispatch_unlock` | `dispatch::mp_dispatch_unlock` | as `mp_dispatch_lock` | rejected: as above | rejected: as above |

The queue keeps the C's fields under one `Mutex` and `Condvar`. A callback
is a boxed closure; `DispatchKey` stands for the `(fn, fn_data)` pair that
`mp_dispatch_enqueue_notify` merges on and `mp_dispatch_cancel_fn`
removes by. The wakeup callback runs without the lock and the onlock
callback with it, as in the C; items run with the lock released and
`locked` set; autofree data is freed after the call, with the lock held
again.

The tests:

- Sequences (one thread, which is the target thread): 10,000 sequences
  of 10 steps of enqueues of the three kinds, cancels, interrupts and
  `mp_dispatch_queue_process` calls with timeouts 0, -1, -0 and NaN, on
  queues with and without a wakeup callback. Callbacks enqueue, cancel,
  interrupt or move the deadline in turn. The log of which callback ran
  with which data, when autofree data was freed and when the wakeup
  callback ran is compared after every step.
- Timed cases: `mp_dispatch_queue_process` with a timeout of 0, 20 or 40
  ms, or -1, and a callback that moves the deadline by -1 s to +60 ms or to
  "never"; both sides must return within [-1, +15] ms of when the C's rule
  says. 300 cases, C and port at the same time.
- Handshake: a target thread that enters `mp_dispatch_queue_process` only
  when the wakeup callback asks it to; 1 to 5 locks per round. The onlock
  callback must run once per lock, the wakeup callback at least once, and
  the target must not leave `mp_dispatch_queue_process` while the lock is
  held. 200 rounds.
- Stress: three threads enqueue, run synchronously, lock and enqueue
  notifies (60 operations each) while a fourth processes; every item must
  run once, in order per sender, on a processing thread and never while a
  lock is held; every run must return after its callback. 300 rounds.

### `misc/hash.c`, `misc/io_utils.c`, `misc/natural_sort.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `mp_hash_to_bstr` | `hash::mp_hash_to_bstr` | 20,000 inputs, all agree | body | body |
| `mp_mkostemps` | `io_utils::mp_mkostemps` | 5,000 inputs, all agree | body | body |
| `mp_save_to_file` | `io_utils::mp_save_to_file` | 2,000 inputs, all agree | body | body |
| `mp_natural_sort_cmp` | `natural_sort::mp_natural_sort_cmp` | 100,000 inputs, all agree | body | body |

`mp_mkostemps` draws random file names. The test gives the C and the port
the same random state and compares the names, the file contents and
`errno`.

### `misc/json.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `eat_c` | `json::eat_c` | 100,000 inputs, all agree | body | body |
| `eat_ws` | `json::eat_ws` | 100,000 inputs, all agree | body | body |
| `json_skip_whitespace` | `json::json_skip_whitespace` | 100,000 inputs, all agree | body | body |
| `read_id` | `json::read_id` | 100,000 inputs, all agree | body | body |
| `read_str` | `json::read_str` | 100,000 inputs, all agree | body | body |
| `read_sub` | `json::read_sub` | 100,000 inputs, all agree | axiom: Lean rejects recursion through `loop` | body |
| `json_parse` | `json::json_parse` | 100,000 inputs, all agree | body | body |
| `write_json_str` | `json::write_json_str` | 100,000 inputs, all agree | body | body |
| `add_indent` | `json::add_indent` | 100,000 inputs, all agree | body | body |
| `json_append` | `json::json_append` | 100,000 inputs, all agree | axiom: Lean rejects recursion through `loop` | body |
| `json_append_str` | `json::json_append_str` | 100,000 inputs, all agree | body | body |
| `json_write` | `json::json_write` | 100,000 inputs, all agree | body | body |
| `json_write_pretty` | `json::json_write_pretty` | 100,000 inputs, all agree | body | body |

### `misc/language.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `lang_compare` | `language::lang_compare` | 100,000 inputs, all agree | body | body |
| `canonicalize` | `language::canonicalize` | 100,000 inputs, all agree | body | body |
| `mp_match_lang` | `language::mp_match_lang` | 100,000 inputs, all agree | body | body |
| `mp_guess_lang_from_filename` | `language::mp_guess_lang_from_filename` | 100,000 inputs, all agree | body | body |

### `misc/node.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `node_init` | `node::node_init` | 21 inputs, all agree | body | body |
| `node_array_add` | `node::node_array_add` | 100,000 inputs, all agree | body | body |
| `node_map_add` | `node::node_map_add` | 100,000 inputs, all agree | body | body |
| `node_map_badd` | `node::node_map_badd` | 100,000 inputs, all agree | body | body |
| `node_map_add_string` | `node::node_map_add_string` | 100,000 inputs, all agree | body | body |
| `node_map_add_bstr` | `node::node_map_add_bstr` | 100,000 inputs, all agree | body | body |
| `node_map_add_int64` | `node::node_map_add_int64` | 100,000 inputs, all agree | body | body |
| `node_map_add_double` | `node::node_map_add_double` | 100,000 inputs, all agree | body | body |
| `node_map_add_flag` | `node::node_map_add_flag` | 100,000 inputs, all agree | body | body |
| `node_map_get` | `node::node_map_get` | 100,000 inputs, all agree | body | body |
| `node_map_bget` | `node::node_map_bget` | 100,000 inputs, all agree | body | body |
| `equal_mpv_value` | `node::equal_mpv_value` | 100,000 inputs, all agree | axiom: Lean rejects recursion through `loop` | body |
| `equal_mpv_node` | `node::equal_mpv_node` | 100,000 inputs, all agree | body | body |

### `misc/path_utils.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `mp_basename_bstr` | `path_utils::mp_basename_bstr` | 100,000 inputs, all agree | body | body |
| `mp_basename` | `path_utils::mp_basename` | 100,000 inputs, all agree | body | body |
| `mp_dirname` | `path_utils::mp_dirname` | 100,000 inputs, all agree | body | body |
| `mp_path_strip_trailing_separator` | `path_utils::mp_path_strip_trailing_separator` | 100,000 inputs, all agree | body | body |
| `split_ext` | `path_utils::split_ext` | 100,000 inputs, all agree | body | body |
| `mp_strip_ext` | `path_utils::mp_strip_ext` | 100,000 inputs, all agree | body | body |
| `mp_get_ext` | `path_utils::mp_get_ext` | 100,000 inputs, all agree | body | body |
| `mp_path_is_absolute` | `path_utils::mp_path_is_absolute` | 100,000 inputs, all agree | body | body |
| `mp_path_join_bstr` | `path_utils::mp_path_join_bstr` | 100,000 inputs, all agree | body | body |
| `mp_path_join` | `path_utils::mp_path_join` | 100,000 inputs, all agree | body | body |
| `mp_getcwd` | `path_utils::mp_getcwd` | 9 inputs, all agree | body | body |
| `mp_normalize_path` | `path_utils::mp_normalize_path` | 20,000 inputs, all agree | body | body |
| `mp_path_exists` | `path_utils::mp_path_exists` | 20,000 inputs, all agree | body | body |
| `mp_path_isdir` | `path_utils::mp_path_isdir` | 20,000 inputs, all agree | body | body |
| `mp_is_url` | `path_utils::mp_is_url` | 100,000 inputs, all agree | body | body |
| `mp_split_proto` | `path_utils::mp_split_proto` | 100,000 inputs, all agree | body | body |
| `mp_mkdirp` | `path_utils::mp_mkdirp` | 2,000 inputs, all agree | body | body |

### `misc/random.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `rotl_u64` | `random::rotl_u64` | 100,000 inputs, all agree | body | body |
| `splitmix64` | `random::splitmix64` | 100,000 inputs, all agree | body | body |
| `mp_rand_seed` | `random::mp_rand_seed` | 100,000 inputs, all agree | body | body |
| `mp_rand_next` | `random::mp_rand_next` | 100,000 inputs, all agree | body | body |
| `mp_rand_next_double` | `random::mp_rand_next_double` | 100,000 inputs, all agree | left out: `f64` in the signature | left out: `f64` in the signature |
| `mp_rand_in_range32` | `random::mp_rand_in_range32` | 100,000 inputs, all agree | body | body |

`mp_rand_seed(0)` seeds from the clock and the system's random bytes; the
test covers non-zero seeds only.

### `misc/rendezvous.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `mp_rendezvous` | `rendezvous::mp_rendezvous` | 2,000 rounds, all agree | rejected: `Unimplemented` (`Mutex`, `Condvar`) | rejected: as default |

### `misc/thread_pool.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `worker_thread` | `thread_pool::worker_thread` | 300 growth rounds and the idle timeout, all agree | rejected: `dyn` closures, `Mutex`, `Condvar` | rejected: as default |
| `thread_pool_dtor` | `Drop for thread_pool::MpThreadPool` | 500 rounds (growth and order), all agree | rejected: as above | rejected: as above |
| `add_thread` | `thread_pool::add_thread` | 300 growth rounds, all agree | rejected: as above | rejected: as above |
| `mp_thread_pool_create` | `thread_pool::mp_thread_pool_create` | 300 growth rounds with random initial, minimum and maximum counts, all agree | rejected: as above | rejected: as above |
| `thread_pool_add` | `thread_pool::thread_pool_add` | 300 growth rounds, all agree | rejected: as above | rejected: as above |
| `mp_thread_pool_queue` | `thread_pool::mp_thread_pool_queue` | 300 growth rounds and 200 order rounds, all agree | rejected: as above | rejected: as above |
| `mp_thread_pool_run` | `thread_pool::mp_thread_pool_run` | as `mp_thread_pool_queue` | rejected: as above | rejected: as above |

Work is a boxed closure, and a failed add hands it back. The work items of
the tests wait on a gate the test opens, so that how many threads start
and what each add returns follow from the C's counting rule and not from
the scheduler. Order rounds: one thread runs up to 40 items in the order
queued. Growth rounds: up to 6 items on pools of at most 4 threads; the
results of the adds, how many items run before the gate opens, on how many
threads, and how many finish must agree. The idle timeout (10 s) is run
once on both sides at the same time: a thread above the minimum ends, and
the next work needs a new thread.

### `misc/thread_tools.c`

| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |
| --- | --- | --- | --- | --- |
| `mp_waiter_wait` | `thread_tools::mp_waiter_wait` | 2,000 rounds, all agree | rejected: `Unimplemented` (`Mutex`, `Condvar`) | rejected: as default |
| `mp_waiter_wakeup` | `thread_tools::mp_waiter_wakeup` | 2,000 rounds (same test), all agree | rejected: as above | rejected: as above |
| `mp_waiter_poll` | `thread_tools::mp_waiter_poll` | 2,000 rounds (same test), all agree | rejected: as above | rejected: as above |
| `cancel_destroy` | `Drop for thread_tools::MpCancel` | 5,000 sequences of 20 operations, all agree | rejected: `dyn` closures, `Mutex`, `Condvar`, atomics | rejected: as default |
| `mp_cancel_new` | `thread_tools::mp_cancel_new` | 5,000 sequences, all agree | rejected: as above | rejected: as above |
| `trigger_locked` | `thread_tools::trigger_locked` | 5,000 sequences and a pipe filled by 70,000 triggers, all agree | rejected: as above | rejected: as above |
| `mp_cancel_trigger` | `thread_tools::mp_cancel_trigger` | as `trigger_locked`; 200 rounds of two threads and 200 of four | rejected: as above | rejected: as above |
| `mp_cancel_reset` | `thread_tools::mp_cancel_reset` | 5,000 sequences and the full pipe, all agree | rejected: as above | rejected: as above |
| `mp_cancel_test` | `thread_tools::mp_cancel_test` | 5,000 sequences, all agree (after every step, for every object) | rejected: as above | rejected: as above |
| `mp_cancel_wait` | `thread_tools::mp_cancel_wait` | 5,000 sequences (timeouts 0, -1, -1e300, -inf, 0.1 ms, NaN), all agree; 200 rounds of two threads | rejected: as above | rejected: as above |
| `retrigger_locked` | `thread_tools::retrigger_locked` | 5,000 sequences, all agree | rejected: as above | rejected: as above |
| `mp_cancel_set_cb` | `thread_tools::mp_cancel_set_cb` | 5,000 sequences, all agree (callback counts) | rejected: as above | rejected: as above |
| `mp_cancel_set_parent` | `thread_tools::mp_cancel_set_parent` | 5,000 sequences, all agree; 200 rounds of four threads | rejected: as above | rejected: as above |
| `mp_cancel_get_fd` | `thread_tools::mp_cancel_get_fd` | 5,000 sequences, all agree (bytes waiting and `poll` readiness) | rejected: as above | rejected: as above |

The sequences build forests of up to eight objects on both sides and apply
the same random operations: trigger, reset, test, wait, reparent (never
into a loop), set or remove the callback, make the pipe, free a leaf. After
every operation the flag of every object, how often its callback ran, and
for a pipe the bytes waiting (`FIONREAD`) and `poll` readiness are compared.
A slave holds its parent with an `Arc` and the parent holds its slaves
weakly. The pipe is a real non-blocking pipe (see "Where the port differs
from the C").

The waiter tests run the C and the port each in two threads and check the
handed-over values; the threaded `mp_cancel` tests do the same with a
waiter against a trigger and with a tree triggered, reset and reparented
from four threads. They sample thread orders; loom explores the port's
(next section).

### Concurrency: explored and sampled

`crates/misc/tests/loom.rs` runs the port under loom 0.7.3 (`make loom`):
with `--cfg loom` the port's `Mutex`, `Condvar`, atomics and threads are
loom's, and each model runs under every schedule loom's exploration
reaches, with a preemption bound of 3 where noted. A deadlock, a lost
wakeup or a failed assertion in any schedule fails it. loom does not model
time: a wait with a finite timeout releases the lock, lets others run and
times out; the models use no timeout or 0.

| Property | Explored by loom (schedules) | Sampled against the C |
| --- | --- | --- |
| `mp_waiter`: the waiter gets the value; a poll after the wakeup sees it | yes (23) | 2,000 rounds |
| A dispatch callback never runs while `mp_dispatch_lock` is held, runs once, and the target returns when interrupted | yes (140) | 300 stress rounds |
| Two lockers exclude each other and both get the lock | yes, bound 3 (9,542) | 300 stress rounds |
| `mp_dispatch_run` returns after its callback ran on the target thread | yes (28) | 300 stress rounds |
| `mp_dispatch_lock` with the target outside `mp_dispatch_queue_process`: onlock once, wakeup brings the target in | yes (21) | 200 handshake rounds |
| An interrupt is never lost | yes (3) | in every stress round |
| `mp_cancel_wait` against a trigger: never lost | yes (10) | 200 rounds |
| Linking a slave against a trigger of the parent, and reparenting between two triggered parents: the slave ends triggered | yes (5; bound 3, 146) | 200 rounds of four threads |
| `mp_cancel_set_cb` against a trigger: the callback runs | yes (5) | no |
| Trigger against reset: the flag and a wait of 0 agree | yes (5) | 200 rounds of four threads |
| Thread pool: an idle exit racing new work; shutdown runs queued work; a waiting worker is woken | yes (bound 3: 13; 14; 64) | growth and idle-timeout rounds |
| Everything above in the C | no: only sampled | as listed |
| Timeouts, deadlines and `mp_dispatch_adjust_timeout` | no (loom has no time) | 300 timed cases, idle timeout once |
| Orders of more than four threads, and the port under the real `std` primitives | no | the stress tests |

Seven of these models were checked against planted bugs (a lock that does
not wait for `locked`, a missing retrigger, a missing broadcast in
`mp_dispatch_queue_process`, `trigger_locked` and `mp_waiter_wakeup`, a
missing worker wakeup): loom fails each.

### Totals

| Column | Count |
| --- | --- |
| Ported | 141 of 141 (`jni.c`, Android-only, is not in this build and not counted) |
| Differentially tested | 140 of 141 (`resize_append` has no observable result) |
| Agree with the C on every input | 140 of 140 |
| Aeneas default: body | 98 |
| Aeneas default: axiom | 3 (`read_sub`, `json_append`, `equal_mpv_value`) |
| Aeneas default: left out or rejected | 40 (2 for `f64`; `bstr_sscanf` for `f32`/`f64` in `ScanArg`; 37 threaded: 4 for `Mutex`/`Condvar`, 33 for that and boxed closures) |
| Aeneas `-loops-to-rec`: body | 101 |
| Aeneas `-loops-to-rec`: axiom | 0 |
| Aeneas `-loops-to-rec`: left out or rejected | 40 (as above) |
| Not attempted | 0 |

The tests also cover what the port adds for the C library: `strtoll`,
`strtod`, `strtof`, `vsnprintf`, `vsscanf`, `mp_time_ns_add` (in the timed
tests), `printf("%f")`, `printf("%" PRId64)`, `memcmp`/`strcmp`/
`strcasecmp` families (sign only), `strchr`, the nine `ctype.h` helpers, and
the `common/` helpers `mp_log2`, `mp_round_next_power_of_2` and
`mp_append_utf8_bstr`. Each runs against glibc or mpv's own object code.

`tools/mutate.py` plants 34 bugs in the port, one at a time (wrong bounds,
dropped `0b` prefix, `ERANGE` dropped, NUL handling changed, a skipped
comparison, and for the ports added last the octal marker, `%a` rounding,
the `%#g` carry, the scanf sign and `0b` prefix, `strtof`'s sticky bits,
notify merging, the interrupt rule, the pool's thread limit, the retrigger
on reparenting and the pipe write); the differential tests catch all 34.
The first run missed the `strtof` one, and four inputs at halfway points
were added.

### What a body rests on

A Lean `def` can call a declaration without a body. In the `-loops-to-rec`
file, 77 of the 101 bodies reach no axiom at all, directly or through
callees. The other 24:

- 16 reach only `Vec::capacity` and `Vec::reserve_exact`, through
  `resize_append`, which mirrors talloc's growth policy, and the opaque
  modules: `bstr_xappend_asprintf` and `bstr_xappend_vasprintf` (also
  `cprintf::vsnprintf` and its types), `bstr_xappend`,
  `bstr_sanitize_utf8_latin1`, `resize_append`, `mp_iconv_to_utf8`,
  `mp_hash_to_bstr`, `read_str`, `write_json_str`, `add_indent`,
  and, with `cfloat::strtod` or `cfloat::format_d`/`format_f` and in three
  cases `Vec::truncate`, `read_sub`, `json_parse`, `json_append`,
  `json_append_str`, `json_write`, `json_write_pretty`.
- 8 reach the system-call module `os` (and the opaque type `std::fs::File`):
  `mp_mkostemps`, `mp_save_to_file`, `mp_getcwd`, `mp_normalize_path`,
  `mp_path_exists`, `mp_path_isdir`, `mp_mkdirp`, `mp_rand_seed`.

(An earlier version of this report gave 16 for a list of 14 names.) The `cfloat`, `cprintf` and `os` axioms are deliberate: they are library and system
calls, which are declarations on the C side as well. The `Vec` capacity
axioms are not; see "What to change".

## Conventions

The crate's `README.md` has the full mapping. The memory rule in short:

| C | Rust |
| --- | --- |
| `bstr` view | `&[u8]`; a returned view is a subslice of the argument, so position is kept and compared by the tests |
| NULL `bstr` | empty slice (each function that returns one never returns a non-NULL empty view, so nothing is lost) |
| `const char *` | `&[u8]` whose content ends at the first NUL or the end; `clib::cstr_at` reads past the end as 0 |
| `char **` list ending in NULL | `&[Vec<u8>]` (Aeneas rejects a slice of slices) |
| talloc allocation returned | `Vec<u8>`; the `talloc_ctx` parameter is dropped and the trailing NUL is not stored |
| "returns input or a new allocation" | `Option<Vec<u8>>`, `None` for the input |
| `bstr *` / `char **` advanced in place | new value returned in a tuple after the C return value |
| out-parameter written on some paths only | passed in by value and returned (the shape Aeneas gives `&mut`) |
| pointer into a caller's buffer | offset into it |
| `unsigned` wraparound | `wrapping_*`; signed arithmetic uses plain operators (C UB, Rust debug panic) |
| `double` passed on or stored | `u64` bit pattern, so only `bstrtod` and `mp_rand_next_double` have `f64` |
| `mpv_node` | `enum MpvNode` |
| `errno` | `Result<_, i32>` |
| file descriptor | `std::fs::File` |
| `mp_assert` | `assert!` |
| variadic `printf` arguments | `&[cprintf::CArg]`, each argument after the default promotions |
| variadic `scanf` pointers | `&mut [cscanf::ScanArg]`, a `&mut` of the pointed-to type |
| function pointer and `void *` callback | boxed closure; `DispatchKey` where the C compares the pointers |
| `mp_mutex`, `mp_cond`, `atomic_bool` | `Mutex`, `Condvar`, `AtomicBool` from `threads` (loom's under `--cfg loom`) |
| object freed with `talloc_free` and a destructor | `Drop` (`queue_dtor`, `thread_pool_dtor`, `cancel_destroy`) |
| `mp_cancel *` shared between threads | `Arc<MpCancel>`; a parent holds its slaves weakly |
| wakeup pipe | `os::WakeupPipe`, a real non-blocking pipe |

Other conventions:

- One module per C file, one Rust function per C function, mpv's name in
  snake case (`ucdLookup` is `ucd_lookup`). Helpers added for Aeneas say in
  their comment which C loop they stand for.
- C library calls are ported too, in `clib` (translated by Aeneas), or in
  `cfloat` and `os` (opaque to Aeneas). Each is tested against glibc as the
  pinned build links it.
- The crate has its own lint table; every relaxed lint has its reason in
  the README.

Restructurings made for Aeneas (all behaviour-preserving; the differential
tests and the mutation check were rerun after them): a loop the C leaves
with `return` or `goto` from inside another loop is a helper function; a
loop that returns a borrow ends through a flag; no local shares a module's
name; no byte-string literal of equal bytes (rustc gives Charon a repeat
constant); no derived trait impls under `--cfg aeneas`; `wrapping_neg` is
`0.wrapping_sub(x)`; the `bstr_decode_utf8` out-variable became the input
itself on an error (every caller in mpv passes its own string or ignores
it). The floating-point and formatting code moved from `clib` to `cfloat`
so that Aeneas can keep it opaque.

## Behavioural traps

Each was found by a differential test or by reading the C, and the port
follows the C. Line numbers are in mpv at the pinned commit.

1. **`strtoll` is C23.** `bstrtoll` (`misc/bstr.c:183`) calls `strtoll`,
   which glibc 2.39 redirects to `__isoc23_strtoll` under `_GNU_SOURCE`
   (`/usr/include/stdlib.h:238`), so `0b101` parses in base 0 and base 2.
   c2rust drops the redirect and gets this wrong; the port's
   `clib::strtoll` follows C23. `bstr_sscanf` has the same issue with
   `vsscanf` (`__isoc23_vsscanf`: `%i` takes `0b`, and `%b` exists);
   `cscanf` follows C23 too.
2. **`bstrtoll` reads at most 50 bytes** (`bstr.c:178-180`), and `bstrtod`
   100 (`:192-194`): a long digit run is cut there, and a NUL byte inside
   the view ends the number.
3. **`strncasecmp` stops at NUL.** `bstrcasecmp` (`bstr.c:51`) compares two
   views that agree up to an embedded NUL by length alone.
4. **`strchr(s, 0)` finds the terminator.** So `bstrcspn` (`bstr.c:91`)
   always stops at a NUL byte, and `bstrspn` (`:100`) always accepts one.
5. **`bstr_splice` mixes `int` and `size_t`** (`bstr.c:163-169`): `end` is
   compared with `str.len` as unsigned, so an `end` that is still negative
   after adding the length selects the whole string, not nothing.
6. **`bstr_case_endswith` with an empty suffix** cuts at `-suffix.len`
   (`bstr.c:423`), an unsigned negation of 0, which is 0 and keeps the whole
   string.
7. **`mp_mkostemps` compares `6 + suffixlen` as `size_t`**
   (`io_utils.c:37`), so a negative sum is a huge value and the template is
   rejected.
8. **`json` reads its buffer as a C string**: an escaped `\u0000` ends the
   stored string, and the parser writes NUL bytes into its input
   (`json.c:91`, `:121`).
9. **`memcmp` and `strncasecmp` results**: only the sign is defined; glibc's
   magnitude depends on the CPU and the length, so tests compare signs.
10. **`strtod` and `ERANGE`**: glibc sets `ERANGE` for a tiny result only if
    it is inexact, and does not clear `errno` after the `strtoull` it uses
    for a `nan(...)` payload. `cfloat::strtod` follows both; this matters to
    `bstrtod` and `json_parse`.
11. **`%#g` loses its digits when rounding carries** (glibc's
    `printf_fp.c`). glibc picks the `%f` style before rounding; if rounding
    then makes a new digit it switches to `%e` but keeps the `%f` style's
    fraction digits, none: `%#g` of 999999.5 is `1.e+06`, where ISO C asks
    for `1.00000e+06`. The port does the same.
12. **A `*` width of `INT_MIN` fails.** glibc negates it to `INT_MIN` and
    then subtracts from it in `int`, which wraps to about `INT_MAX`, so the
    padding overflows and `snprintf` returns -1 with `EOVERFLOW`. The port
    keeps glibc's width as a wrapping `i32` for this.
13. **glibc's `scanf` keeps what `strtod` does not use.** For a float it
    reads the longest text that may still become a number, then converts
    with `strtod`: `"1e"` stores 1 and consumes the `e`, so a `%n` after it
    counts 2. A width overflowing `int` means no width, and `"inf"` cut to
    `"infin"` is a matching failure, not infinity.
14. **`mp_cancel_wait` with a negative timeout returns at once**
    (`thread_tools.h` says it waits for ever): `mp_time_ns_add` makes a
    deadline in the past. The port does the same.
15. **`mp_thread_pool_run` queues when the pool is full**
    (`thread_pool.c:195-202`): it fails only when a thread cannot be
    started, not when every thread is busy at the maximum count, though
    the header says it succeeds only "if a thread can be reserved". The
    port does the same.
16. **Linking a slave to a triggered parent re-signals the parent**
    (`thread_tools.c:238`): `retrigger_locked(slave->parent)` runs the
    parent's callback and pipe write again and triggers all its slaves, not
    just the new one.
17. **`mp_waiter_wait` destroys the waiter** and fills it with `0xCA`
    (`thread_tools.c:55`). The port leaves it alone; like the C, it must not
    be waited on again.

## Suspected mpv defects

None of these is reached by an input that mpv itself produces, as far as
this pilot checked; all are unconfirmed as user-visible bugs.

- **`mp_append_utf8_bstr` (`common/common.c:233`) is undefined for values of
  2^31 and above**: `PUT_UTF8` shifts a 32-bit value by 36. Unconfirmed:
  not reproduced, as UB has no single result to reproduce. The generator
  avoids those values; the port panics on them in a debug build.
- **`rotl_u64` (`misc/random.c:33`) is undefined for `k` = 0 or 64**
  (`x >> 64`). mpv calls it only with constants, so it is not reachable.
  The generator avoids those values.
- **`ucdLookup` (`misc/codepoint_width.c:619-622`) reads out of bounds for a
  code point above U+10FFFF.** Its callers decode UTF-8 first, which does
  not produce such values. Unconfirmed; not reproduced. The port panics.
- **`mp_cancel_wait` with a negative timeout** (trap 14) contradicts its
  header. Reproduced; no caller in this build passes a negative timeout.
- **`mp_thread_pool_run`** (trap 15) queues work when all threads are busy
  at the maximum. Reproduced. Its one caller, `video/zimg.c:702`, makes a
  pool whose threads all exist from the start, so it never fails there.
- **`mp_dispatch_cancel_fn` also removes `mp_dispatch_run` items**, which
  live on the caller's stack: it frees a stack object (undefined) and the
  caller then waits for ever. The header limits it to enqueued items; the
  code does not check. Not reproduced (undefined); the port leaves such a
  caller waiting for ever.
- **`mp_time_ns_add` converts NaN to `int64_t`** (`osdep/timer.c`), which
  is undefined; x86-64 gives `INT64_MIN` and so a deadline of 1. The port
  uses that value. Not reachable from mpv's callers as far as checked.
- **`bstr_splice` with a negative `end`** (trap 5) returns the whole string.
  Reproduced (the C does this on every such input), but whether it is
  intended is unconfirmed; no caller in `misc/` passes such an `end`.

## Where the port differs from the C

Each of these is deliberate; none is reached by a valid call that the
tests make on both sides.

- **What the engines refuse.** `cprintf::vsnprintf` returns
  `PrintfError::Unsupported` for `%n`, `%m`, wide characters and strings
  (`%lc`, `%ls`, `%C`, `%S`, and `%zc`, `%Ls` and the like, which glibc reads
  as wide), `long double` (`%Lf`, `%llf`), positional arguments, the `I`
  flag, `wN` modifiers, and conversions glibc does not know, which glibc
  prints back literally. `PrintfError::BadArgument` stands for a missing
  argument or one of the wrong kind (an `int` where `%ld` reads a `long`),
  which is undefined in C. Signedness may differ, as `va_arg` allows.
  `cscanf::vsscanf` refuses positional arguments, `I`, `m`, `wN`, wide
  conversions, `long double` and `%p`, and a wrong or missing pointer, but
  only when glibc would use it: a conversion that fails first takes none.
  No mpv caller uses any of these with `bstr_*` (the `%ls` callers in
  `audio/out/ao_wasapi_changenotify.c` and `osdep/w32_register.c` are
  Windows-only).
- **`%s`, `%c` and `%[` store into a `Vec`.** The port sets it to the bytes
  read; the C writes them into the caller's buffer, which it can overrun.
- **The output is not written into the buffer's spare capacity first.**
  `bstr_xappend_vasprintf` formats once and then grows the buffer; the C
  formats into the free space and again after growing. The bytes, the
  length and the growth are the same.
- **`mp_dispatch_run` needs a `'static` closure.** The C passes data on
  the caller's stack; safe Rust cannot lend the queue a borrow that ends
  when the call returns. Callers share data with `Arc` instead.
- **Callback identity is a `DispatchKey`**, two numbers the caller picks,
  where the C compares two pointers.
- **The wakeup and onlock setters take `&mut`**, which turns the C's "set
  it before other threads see the queue" into a compile-time rule.
- **A cancelled `mp_dispatch_run` item** leaves its caller waiting for
  ever, where the C frees a stack object (suspected defects).
- **The thread pool hands rejected work back** (`Err(work)`), where the C
  leaves `fn_ctx` with its caller. A work item that panics ends its worker
  thread, which the C has no counterpart for; the pool still counts the
  thread.
- **`mp_cancel` holds its slaves weakly.** A parent therefore cannot be
  freed while it has slaves, which the C asserts against at run time. A
  slave whose last reference is being dropped is skipped by a concurrent
  trigger of its parent, one of the outcomes the C's race allows.
- **The wakeup pipe is opened twice.** `os::WakeupPipe` makes a pipe with
  `std::io::pipe` and opens each end again through `/proc/self/fd/N` with
  `O_NONBLOCK`, which needs no `unsafe` code; without `/proc` it fails and
  `mp_cancel_get_fd` returns -1, as the C does when `pipe` fails. It is a
  real pipe of the default size, so a full pipe and its drain behave as
  mpv's (tested with 70,000 triggers).
- **`mp_time_ns` counts from the port's first call, plus 1 ns.** mpv's
  counts from `mp_time_init`. Deadlines are only compared with the same
  clock, so only the absolute values differ.
- **loom's timed wait.** Under `--cfg loom` only, a wait with a finite
  timeout yields once and times out; loom has no time. Normal builds use
  `Condvar::wait_timeout`.
- **`rendezvous` is not under loom.** Its state is a `static Mutex`, and
  loom's `Mutex::new` is not `const`.

## The two Rust copies compared

The c2rust copy is `pilot/misc-c2rust` (c2rust 0.22.1, unchanged except a
three-line atomic fix, see its README). Its differential numbers are from
`/home/user/achates/docs/research/c2rust.md`.

| | Hand port (`crates/misc`) | c2rust (`pilot/misc-c2rust`) |
| --- | --- | --- |
| Lines of Rust (all / without blanks and comments) | 7,889 / 6,261, of which 690 generated tables (4,748 / 3,819 before the last 36 functions) | 15,165 / 15,165 |
| Function definitions | 332 (all 141 mpv functions, helpers, C library stand-ins including the printf and scanf engines) | 223 (all 141, plus header inline helpers copied into each file) |
| `unsafe` functions / blocks | 0 / 0 (`#![forbid(unsafe_code)]`) | 223 of 223 definitions are `unsafe fn` / 7 blocks |
| Raw-pointer types written (`*mut`, `*const`) | 0 | 1,657 |
| Toolchain | stable (the workspace toolchain) | `nightly-2023-04-15`; with Charon's `nightly-2026-09-17` it needs 10 edits (`VaListImpl`, atomic intrinsics) |
| Compiles on stable | yes | no (`c_variadic`, `core_intrinsics`, `extern_types`) |
| Links without mpv | yes | no: needs talloc, `mp_msg` and the rest of `libmpv.a` |
| Differential agreement with C | 140 of 140 tested functions, 13.4 M inputs | 21 of 22 tested functions; `bstrtoll` differs on `0b` input (C23 redirect lost) |
| Functions Aeneas translates with a body | 101 of 141 (98 without `-loops-to-rec`) | 0: Charon succeeds after 10 toolchain edits, Aeneas reports errors for 113 of the 141 functions and then aborts with no Lean output |
| Effort | most of one day of agent time for the first 105 functions with the test harness, and about as much again for the last 36 with their engines, threaded tests and loom models (neither measured precisely) | minutes to transpile, plus one three-line edit |
| Good for | the Rust side of per-function equality proofs, and the code rmpv ships | an executable reference that links where the C links; a check that a port's test harness can call the C |

What happened when the c2rust copy went through Charon and Aeneas
(`pilot/misc-c2rust/README.md` has the commands; the logs are in
`pilot/misc-c2rust/aeneas-run/`):

1. With its own toolchain the crate builds, but Charon uses
   `nightly-2026-09-17`, on which it does not: `VaListImpl` and
   `as_va_list` no longer exist, and the atomic intrinsics changed
   signature. With 10 lines changed in a scratch copy, Charon succeeds
   and writes a 25 MB LLBC file (one internal Charon panic on one item).
2. Aeneas ran for about 17 minutes, reached 7 GB of memory, logged 555
   `[Error]` lines, and then stopped with `Fatal error: exception
   Invalid_argument("List.combine")` in `translate_crate_to_pure`. It wrote
   no Lean file, so no function has a body.
3. Before the abort it reported a failure for 113 of the 141 mpv functions
   (215 items counting helpers and type declarations). The causes, by
   count: 401 internal errors, 33 type groups that could not be analysed
   (the pthread and `mpv_node` types), 24 dereferences of raw pointers,
   21 unions, 8 function-pointer types, 6 raw-pointer casts, 38 `&raw`
   operations, 3 early returns inside loops. The other 28 functions,
   mostly small `bstr` and `codepoint_width` helpers, had no error before
   the abort. A few of them use no pointers (`rotl_u64`, `ucdLookup`, the
   `ucdGrapheme*` helpers) and might have translated; most take or return
   a `bstr`, which holds a raw pointer.

This matches the rule-based estimate in
`/home/user/achates/docs/research/c2rust.md` (4 of 141 plausible as is) and
is worse in practice: the run produced nothing to check.

Recommendation: write the port by hand, in the shape this pilot settled
on, and use c2rust only as a throwaway reference where the C is hard to
reach from a test (static functions, odd linkage). The hand port is safe,
builds on stable, agrees with the C on every tested input, and gives Aeneas
bodies for 101 of 141 functions, 77 of them with no axiom underneath. The
c2rust copy cannot be a proof target: Aeneas cannot dereference raw
pointers or read unions, and almost everything in it is one. It also
silently changed behaviour once (`0b` in `bstrtoll`), which shows that a
mechanical translation is not a behavioural reference by itself.

## What to change before the next folder

1. **Stop modelling capacity.** `resize_append` mirrors talloc's growth
   with `Vec::capacity` and `Vec::reserve_exact`, which Aeneas leaves as
   axioms, and 16 bodies depend on them only for that. No caller can see
   capacity. Dropping it would leave those 16 axiom-free except for
   `cfloat`. The same goes for `Vec::truncate` (use a loop of `pop` or
   build the shorter vector).
2. **Use `-loops-to-rec` only.** It gives a body to every function the
   default leaves as an axiom, and the two files are otherwise the same
   functions. Keeping both doubles the build for no gain.
3. **Port the C library once, in a shared crate.** `clib`, `cfloat` and
   `os` will be needed by every folder; they should live in one crate with
   one set of glibc tests, and the C23 redirects (`strtoll`, `vsscanf`,
   others in `stdlib.h` and `stdio.h`) should be listed from the headers,
   not found one by one.
4. **Port threaded code with loom from the start.** `Mutex`, `Condvar`
   and boxed closures are outside Aeneas, and two-thread sampling is weak
   evidence. This pilot ended doing it: `threads` swaps the primitives for
   loom's under `--cfg loom`, and `tests/loom.rs` explores the races. A
   model of the C itself (TLA+ or similar) would still be needed to say
   the C has the same properties; the tests only sample it.
5. **Keep the mutation check.** It caught nothing missing here, but it is
   the only evidence that a test of 100,000 agreeing inputs actually reaches
   the branch it is meant to; extend `tools/mutate.py` for each new folder.
6. **Record the generator's avoided inputs in one place.** They are now in
   test comments (`PUT_UTF8` above 2^31, `rotl_u64` with 0 and 64,
   `mp_rand_seed(0)`); a list in the test crate would let a reviewer see
   every gap at once.
