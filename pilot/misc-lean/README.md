# The misc port in Lean

`crates/misc` translated to Lean 4 by Charon and Aeneas, and a Lake project
that type-checks the result. Nothing is proved here; this is the Rust half
of the statements "this Rust function equals that C function".

| Path | Contents |
| --- | --- |
| `translate.sh` | Runs Charon and Aeneas on the crate and writes the two files below. |
| `MiscLoops/RmpvMisc.lean` | Aeneas's default translation: a Rust loop is a call of the library's `loop` combinator. `json::read_sub`, `json::json_append` and `node::equal_mpv_value` are declarations without a body here. |
| `MiscRec/RmpvMisc.lean` | The translation with `-loops-to-rec`: a Rust loop is a recursive Lean function. Every function Aeneas was given has a body. |
| `status.py` | Lists each mpv function with what it became in the two files. |

Both files are generated. Do not edit them; change the crate and run
`translate.sh`.

## Running it

It needs an Aeneas release (the pilot used `nightly-2026.11.21-2e3d448`),
the Rust toolchain Charon asks for (`nightly-2026-09-17`), and Lean 4.36.0
with Mathlib `v4.36.0`. `lakefile.toml` and `translate.sh` point at the copy
of Aeneas under `/home/user/spike-aeneas/dist`; change both paths on another
machine. `/home/user/achates/docs/research/aeneas.md` says how that copy was
set up.

```bash
pilot/misc-lean/translate.sh           # about 1 minute
cd pilot/misc-lean && lake build       # about 2 minutes once Mathlib is built
```

`lake build` must end without an error and without `declaration uses
'sorry'`. Aeneas reports success on a file that still holds a `sorry` or an
axiom for one of the crate's own functions, so also read `aeneas.log` and
run `status.py`.

## What is opaque, and why

- `rmpv_misc::cfloat` (`strtod`, `strtof`, `printf` conversions),
  `rmpv_misc::cprintf` (the `printf` engine of `bstr_xappend_asprintf`) and
  `rmpv_misc::os` (system calls): library and system calls, declarations on
  the C side too. A derived `Clone` in `cprintf` stops Aeneas with an
  internal error even there, so its `Clone` impls are written out.
- `bstr::bstrtod` and `random::mp_rand_next_double` are left out entirely:
  their signatures have an `f64`, and the Aeneas library has no float type.
- `thread_tools`, `rendezvous`, `threads`, `dispatch` and `thread_pool`
  are left out as well. They use `Mutex` and `Condvar`, for which Aeneas
  reports `Unimplemented` and emits declarations that do not type-check,
  and `dispatch`, `thread_pool` and `mp_cancel` hold boxed closures
  ("Dynamic trait types are not supported yet"). With these modules in,
  Aeneas stops with a fatal error and writes no Lean file.
- `cscanf` (the `scanf` engine) and `bstr::bstr_sscanf` are left out:
  their argument type holds `&mut f32` and `&mut f64`.
- In `MiscLoops` only, the three tree walkers named above. Lean cannot show
  that a function is well defined when it calls itself from inside `loop`
  ("could not prove ... to be monotone in its recursive calls").
