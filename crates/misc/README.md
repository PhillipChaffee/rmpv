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
with mpv's name in snake case (`ucdLookup` is `ucd_lookup`). Three modules
have no C file in `misc/`:

| Module | Holds |
| --- | --- |
| `ctype` | The inline helpers of `misc/ctype.h`. |
| `common` | The functions of `common/common.c` that `misc/` calls. |
| `clib` | Stand-ins for the C library: `memcmp`, `strtoll`, `strtod`, `printf("%f")` and others, as glibc behaves in the pinned build. |

`codepoint_width_tables.rs` and `language_tables.rs` are copied from the C
source by `tools/gen_tables.py`.

## How C maps to Rust

| C | Rust |
| --- | --- |
| `bstr` view (pointer and length) | `&[u8]`. A returned view is a subslice of the argument, so its position in the buffer is kept. |
| NULL `bstr` (`(bstr){0}`) | The empty slice. Each function that returns one says so; in every such function a non-NULL result is never empty, so nothing is lost. |
| `const char *` | `&[u8]` holding a C string: the content ends at the first NUL byte, or at the end of the slice. `clib::cstr_at` reads past the end as 0, as C reads the terminator. |
| `char **` list ending in NULL | `&[&[u8]]` |
| talloc allocation returned to the caller | `Vec<u8>`. The `talloc_ctx` parameter is dropped. The NUL that mpv keeps after the data is not stored. |
| "returns its input or a new allocation" | `Option<Vec<u8>>`, `None` for the input. |
| `bstr *` or `char **` that the function advances | The new value is returned, in a tuple after the C return value. |
| Out-parameter that is always written | Returned in the tuple. A caller that passes NULL in C ignores it. |
| Out-parameter that is written only on some paths | Passed in by value and returned: the caller gives the current value and gets the new one. This is the shape Aeneas gives a `&mut` parameter. |
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

A loop that the C exits with `return`, or a `goto`, from inside another loop
is a helper function here, because Aeneas rejects that control flow. Each
helper's comment names the loop it stands for. Loops over tree nodes
(`equal_mpv_value`, `json_append`) are recursive helpers, because Lean does
not accept recursion through a translated loop.

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
| `comparison_chain`, `option_if_let_else`, `useless_let_if_seq`, `nonminimal_bool` | The suggested forms (`match` on `cmp`, closures, reordered conditions) move away from the C's control flow. |
| `similar_names`, `many_single_char_names` | The names are mpv's (`str1`, `str2`) or the hash algorithms' (`a`, `b`, `c`, `d`). |
