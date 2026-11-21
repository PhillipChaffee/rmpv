# rmpv

A from-scratch media player in Rust, specified against [mpv](https://mpv.io).

Nothing plays yet. The repository currently holds the formal model of mpv and the
tool that measures how much of mpv that model accounts for. See [formal/](formal/README.md).

## Development

```bash
make setup    # install lint tools, enable the pre-commit hook
make fmt      # format Rust and TOML
make lint     # rustfmt, taplo, typos, clippy (-D warnings), cargo-deny
make test
make proofs   # kernel-check the Lean proofs, validate coverage claims
make cover    # coverage report against the pinned mpv commit
make ci       # lint + test + proofs, exactly what CI runs
```
