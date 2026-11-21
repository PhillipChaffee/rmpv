# rmpv-misc-difftest

Differential tests of `rmpv-misc` against mpv's C. Each test runs one C
function and its port on the same generated inputs and compares every
result and every output: returned values, the position of returned views in
their buffer, buffers written in place, and files created.

```bash
make difftest      # builds mpv if needed, then runs every test
```

The tests need mpv's static build in `formal/.mpv-build` (`make mpv-build`).
Under a plain `cargo test` they run if that build exists. If it does not,
the build script prints a warning and the crate compiles without them.
`RMPV_REQUIRE_MPV_BUILD=1` turns the warning into an error;
`RMPV_MPV_BUILD` names another build directory.

## What is compared with what

- A function mpv exports is called in `libmpv.a`: the object mpv's own build
  compiled.
- A `static` function is not in the archive. `shim/static_*.c` includes the
  mpv source file and adds a `dt_*` wrapper for each static function. The
  file is compiled with the flags in `harness.rsp`, mpv's own code-generation
  flags. `objcopy` then makes every symbol but the wrappers local, so the
  second copy of the file's other functions is not used by anything else.
- A `static inline` helper from a header gets a wrapper in
  `shim/inline_shim.c`, which also holds wrappers for the functions that take
  or return `mpv_node` trees. Trees cross the boundary as bytes
  (`shim/dt_node.h`, `src/node_wire.rs`).
- `shim/interpose.c` replaces libavutil's `av_random_bytes`, so that a test
  can fix the state `mp_rand_seed(0)` returns.

Inputs come from splitmix64 with a fixed seed per test, 100,000 per
function unless the test says why it runs fewer. They are biased towards
what the C branches on: empty strings, NUL bytes, bytes from 0x80 up,
ill-formed UTF-8, long digit runs, `0x` and `0b` prefixes, the limits of
`int` and `long long`, and the smallest and largest `double` values.

Each test prints `difftest <function>: <n> inputs agree`.

## Lints

This crate holds the only `unsafe` in the workspace's port: the `extern`
declarations in `src/ffi.rs`, and the `c!` macro that every C call goes
through. Its lint table is the workspace's with these changed:

| Lint | Change and reason |
| --- | --- |
| `unsafe_code` | Allowed: calling C needs it. Each `unsafe` block still needs a `SAFETY` comment. |
| `macro_metavars_in_unsafe` | Allowed: `c!` evaluates its arguments inside its `unsafe` block so that one safety argument covers every call. |
| `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` | Allowed: the crate is test code, including its library and build script, and a failed expectation should stop the test. |
| `cast_*`, `similar_names`, `too_many_lines` | Allowed: test inputs are built with casts, and a test of several helpers is long. |
