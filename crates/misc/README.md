# rmpv-misc

A safe Rust port of mpv's `misc/` folder at the commit pinned in
`formal/mpv.lock.toml`. It has two aims that sometimes pull apart:

1. Behave exactly like mpv's compiled C. `crates/misc-difftest` checks this
   by running both on the same inputs.
2. Be Rust that [Aeneas](https://github.com/AeneasVerif/aeneas) can translate
   to Lean, shaped like the C function by function, so that a proof that the
   two are equal has little to bridge.

Where they pull apart, behaviour wins. `docs/pilot-misc-rust.md` has the
status of every function.

## Layout

One module per C file with the same name, and one function per C function
with mpv's name in snake case (`ucdLookup` is `ucd_lookup`). These modules
have no C file in `misc/`:

| Module | Holds |
| --- | --- |
| `ctype` | The inline helpers of `misc/ctype.h`. |
| `common` | The functions of `common/common.c` that `misc/` calls. |
| `clib` | Stand-ins for the C library that Aeneas translates: `memcmp`, `strchr`, `strtoll` and others, as glibc behaves in the pinned build. |
| `cfloat` | Stand-ins for the C library that Aeneas is told to keep opaque: `strtod`, `printf("%f")` and `printf("%" PRId64)`. Their signatures carry a `double` as its bit pattern. |
| `cprintf` | `vsnprintf` as glibc formats in the "C" locale, for `bstr_xappend_asprintf`: a C argument list is a slice of `CArg`, and the format is interpreted as glibc does. What it does not model (`%n`, wide strings, `long double`, positional arguments) it refuses with an error. |
| `os` | The system calls: `stat`, `getcwd`, `realpath`, `mkdir`, `open`, `rename` and the source of random bytes. Opaque to Aeneas as well. |

A function in `cfloat` or `os` is a declaration without a body in the Lean
translation. That is what a library or system call is on the C side too, so
a proof treats both sides alike.

`codepoint_width_tables.rs` and `language_tables.rs` are copied from the C
source by `tools/gen_tables.py`.

## How C maps to Rust

| C | Rust |
| --- | --- |
| `bstr` view (pointer and length) | `&[u8]`. A returned view is a subslice of the argument, so its position in the buffer is kept. |
| NULL `bstr` (`(bstr){0}`) | The empty slice. Each function that returns one says so; in every such function a non-NULL result is never empty, so nothing is lost. |
| `const char *` | `&[u8]` holding a C string: the content ends at the first NUL byte, or at the end of the slice. `clib::cstr_at` reads past the end as 0, as C reads the terminator. |
| `char **` list ending in NULL | `&[Vec<u8>]`. Aeneas does not take a slice of slices. |
| Array of `bstr` views into one buffer (`lang_parts`) | `Vec<(usize, usize)>`: offset and length of each view. |
| talloc allocation returned to the caller | `Vec<u8>`. The `talloc_ctx` parameter is dropped. The NUL that mpv keeps after the data is not stored. |
| "returns its input or a new allocation" | `Option<Vec<u8>>`, `None` for the input. |
| `bstr *` or `char **` that the function advances | The new value is returned, in a tuple after the C return value. |
| Out-parameter that is always written | Returned in the tuple. A caller that passes NULL in C ignores it. |
| Out-parameter that is written only on some paths (`split_ext`, `mp_split_proto`, `term_disp_width`) | Passed in by value and returned: the caller gives the current value and gets the new one. This is the shape Aeneas gives a `&mut` parameter. |
| `bstr_decode_utf8(s, &next)`, `bstr_split_utf8` | The exception to the row above: on an error they return `s` itself as the rest. Aeneas cannot analyse a loop that calls a function with two slices of one lifetime, and every caller in mpv either passes its own string as the out-variable or ignores it after an error. |
| String constant returned by reference (`"."`, `"utf-8"`) | A `static` byte array. Aeneas treats a byte-string literal as a local value. |
| Pointer into a buffer the caller owns (`cut_pos`) | Offset from the start of that buffer. |
| Pointer to an element just appended (`node_array_add`) | Index of the element. |
| `int` position next to a `size_t` length | `i32` next to `usize`. Each implicit C conversion is an `as` cast at the same place. |
| `unsigned` arithmetic that may wrap | `wrapping_*`. |
| Signed arithmetic, shifts | Plain operators: overflow is undefined in C and panics in a Rust debug build. |
| `mp_assert`, `mp_require`, `MP_ASSERT_UNREACHABLE` | `assert!`, `unreachable!` |
| `double` stored or passed on | `u64` bit pattern (`MpvNode::Double`, `node_map_add_double`). Only `bstrtod` and `mp_rand_next_double` use `f64`. |
| `mpv_node` (format tag and union) | `enum MpvNode` |
| `errno` result | `Result<_, i32>` carrying the errno value. |
| `int` file descriptor | `std::fs::File` |
| `memcmp` and `strncasecmp` results | Difference of the first differing bytes. Only the sign is meaningful: glibc's value depends on the CPU and the length. |

## What Aeneas needed

Each of these was found by running Aeneas and the Lean type checker on the
crate. None changes behaviour; the differential tests were rerun after
each.

- A loop that the C leaves with `return`, or with a `goto`, from inside
  another loop is a helper function. Each helper's comment names the loop
  it stands for.
- A loop that returns a borrowed value, or returns after a condition built
  with `||`, ends through a flag and returns after the loop
  (`mp_append_escaped_string`, `mp_guess_lang_from_filename`,
  `mp_mkostemps_with`), or computes the condition into a variable first
  (`mp_is_url`).
- No local variable or parameter has the name of a module (`hash`, `node`):
  in Lean the name would hide the module's namespace.
- No byte-string literal whose bytes are all equal (`b".."`, `b"XXXXXX"`):
  rustc gives Charon a repeat constant, which Aeneas does not read. The
  bytes are compared one by one or written as an array.
- The types derive their traits only without `--cfg aeneas`; Aeneas rejects
  derived implementations on the recursive `MpvNode`.
- `x.wrapping_neg()` is `0.wrapping_sub(x)`, and a comparison is not bound
  to a variable inside an `if` expression: the first has no model, the
  second comes out as a proposition where Lean wants a boolean.

`json_parse`, `json_append` and `equal_mpv_value` walk a tree in a loop that
calls back into them. They are kept as loops, because recursion over the
entries would use stack in proportion to the length of an array where the
C uses none. Lean accepts them only in the translation made with Aeneas's
`-loops-to-rec`; see `pilot/misc-lean`.

## Lints

The crate has its own lint table, which is the workspace's with these
relaxed. The workspace bans `#[allow]`, and a table cannot inherit and
override.

| Lint | Why it is off |
| --- | --- |
| `indexing_slicing` | Aeneas models `a[i]` and `&a[i..j]` directly; `get` returns an `Option` that every proof would have to unwrap. The C indexes the same way. |
| `cast_lossless`, `cast_possible_truncation`, `cast_possible_wrap`, `cast_sign_loss`, `cast_precision_loss` | The casts are C's implicit conversions, written out. `From` and `TryFrom` come out of Aeneas as axioms. |
| `manual_range_contains`, `manual_rotate`, `manual_midpoint`, `manual_is_power_of_two`, `manual_ilog2`, `manual_div_ceil` | The suggested methods are library calls Aeneas has no model for; the manual form is the C expression. |
| `manual_assert_eq` | `assert!(a == b)` is `mp_assert(a == b)`. |
| `byte_char_slices` | A byte-string literal of equal bytes cannot be translated; see above. |
| `manual_let_else`, `manual_map` | Plain `match` is what the probes of Aeneas covered. |
| `comparison_chain`, `option_if_let_else`, `useless_let_if_seq`, `nonminimal_bool` | The suggested forms (`match` on `cmp`, closures, reordered conditions) move away from the C's control flow. |
| `similar_names`, `many_single_char_names` | The names are mpv's (`str1`, `str2`) or the hash algorithms' (`a`, `b`, `c`, `d`). |
