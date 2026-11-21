# rmpv

A from-scratch media player in Rust, specified against [mpv](https://mpv.io).

Nothing plays yet. The repository currently holds the formal model of mpv and two
tools: `mpv-cover` measures how much of mpv that model accounts for, and
`mpv-fidelity` runs the model against mpv's real code. See [formal/](formal/README.md).

## Development

```bash
make setup    # install lint tools, enable the pre-commit hook
make fmt      # format Rust and TOML
make lint     # rustfmt, taplo, typos, clippy (-D warnings), cargo-deny
make test
make proofs   # build mpv, kernel-check the Lean proofs, validate coverage claims, run the fidelity check
make cover    # coverage report against the pinned mpv commit
make ci       # lint + test + proofs, exactly what CI runs
```

`make proofs`, `make cover` and `make ci` fetch mpv and libplacebo from the
network and build them. They run on Linux and need the packages listed in
[formal/README.md](formal/README.md#setup); `make proofs` and `make ci` also
need Lean.
