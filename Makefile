.PHONY: setup fmt lint test mpv proofs cover ci

MPV := formal/.mpv
MPV_REPO := $(shell sed -n 's/^repo = "\(.*\)"/\1/p' formal/mpv.lock.toml)
MPV_COMMIT := $(shell sed -n 's/^commit = "\(.*\)"/\1/p' formal/mpv.lock.toml)

# One-time: install the lint tools and route git hooks through .githooks.
# Lean is installed separately, see formal/README.md.
setup:
	cargo install --locked cargo-deny typos-cli taplo-cli
	git config core.hooksPath .githooks

fmt:
	cargo fmt --all
	taplo fmt

lint:
	cargo fmt --all --check
	taplo fmt --check
	typos
	cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
	cargo deny check

test:
	cargo test --workspace --all-features --locked

# Fetch mpv at the commit pinned in formal/mpv.lock.toml.
$(MPV)/.git:
	git init -q $(MPV)

mpv: $(MPV)/.git
	git -C $(MPV) fetch -q --depth 1 $(MPV_REPO) $(MPV_COMMIT)
	git -C $(MPV) checkout -q --detach FETCH_HEAD

# Kernel-check every proof, reject invalid or stale coverage claims, then run
# the model against real mpv code and confirm planted bugs are caught.
proofs: mpv
	cd formal && lake build --wfail
	cargo run -q --release --locked -p mpv-cover -- check
	cargo run -q --release --locked -p mpv-fidelity

cover: mpv
	cargo run -q --release --locked -p mpv-cover -- report

ci: lint test proofs
