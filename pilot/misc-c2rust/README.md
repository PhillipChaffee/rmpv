# misc-c2rust

Machine translation of mpv's `misc/` folder (commit d11ffba253def00b5172c4b8b47e6b2eb765cae1) to unsafe Rust with c2rust. It is a measurement artifact, not a port.

## How it was produced

1. `c2rust` 0.22.1, built with `LLVM_CONFIG_PATH=/usr/lib/llvm-18/bin/llvm-config cargo install --locked c2rust --version 0.22.1` (Ubuntu packages `llvm-18-dev libclang-18-dev`).
2. `compile_commands.json` from the real meson build (`.mpv-build`), filtered to the 15 `misc/*.c` entries, with `file` made absolute. `misc/jni.c` is not in the build and was not translated.
3. `c2rust transpile cc/compile_commands.json -e -o /home/user/spike-c2rust/misc-c2rust`
4. One manual edit (see below). Nothing else was changed.

## Building

`cargo build` uses the pinned toolchain in `rust-toolchain.toml` (`nightly-2023-04-15`, written by c2rust). It does not build on stable: it needs `c_variadic`, `core_intrinsics`, `extern_types` and `raw_ref_op`.

The crate compiles to an rlib/staticlib. Linking it into a program needs the rest of mpv (talloc, `mp_msg`, `common/` helpers), FFmpeg and libc/pthread. `../difftest` shows one way: link against `libmpv.a` and mpv's link libraries.

## Manual edits

`src/thread_tools.rs`, three lines: c2rust emits `core::intrinsics::atomic_{store_seqcst,load_relaxed}` on a `bool` field (`mp_cancel.triggered`, a C `atomic_bool`). rustc rejects this (E0511, the intrinsics take integer types). The edit casts the pointer to `*mut u8` and the stored value to `u8`. `bool` and `u8` have the same size and layout, so the behaviour does not change.

Compare with the untouched output in `../misc-c2rust.pristine` (`diff -r`).

## This copy

`pilot/misc-c2rust` in rmpv is a copy of `/home/user/spike-c2rust/misc-c2rust`
(sources, manifest and toolchain file; no `target/`). It is outside the rmpv
workspace and is not built by `make test`. `docs/pilot-misc-rust.md`
compares it with the hand-written port in `crates/misc`.

## Charon and Aeneas

Charon builds with its own toolchain (`nightly-2026-09-17`), on which this
crate does not compile as is: `core::ffi::VaListImpl` and
`VaList::as_va_list` are gone (`src/bstr.rs`, 7 lines), and the atomic
intrinsics are now `atomic_store::<T, ORD, VOLATILE>` and
`atomic_load::<T, ORD>` (`src/thread_tools.rs`, 3 lines). The run recorded
in the report used a scratch copy with those 10 lines changed and
`rust-toolchain.toml` removed:

```bash
sed -i 's/::core::ffi::VaListImpl/::core::ffi::VaList/g; s/va\.as_va_list()/va/; s/ap\.as_va_list())/ap)/; s/copy\.as_va_list()/copy/; s/ap\.as_va_list(),/ap.clone(),/' src/bstr.rs
sed -i 's/::core::intrinsics::atomic_store_seqcst(/::core::intrinsics::atomic_store::<u8, { ::core::intrinsics::AtomicOrdering::SeqCst }, false>(/; s/::core::intrinsics::atomic_load_relaxed(/::core::intrinsics::atomic_load::<u8, { ::core::intrinsics::AtomicOrdering::Relaxed }, false>(/' src/thread_tools.rs
charon cargo --preset=aeneas --dest-file OUT/misc_c2rust.llbc
aeneas -backend lean -no-progress-bar -max-recdepth 100000 -dest OUT OUT/misc_c2rust.llbc
```

The copy here is left as c2rust wrote it (plus the original three-line
edit), so it still builds with its pinned toolchain.
