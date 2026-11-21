.PHONY: setup fmt lint test mpv mpv-build proofs cover difftest ci

MPV := formal/.mpv
# Only the top-level keys; later tables such as [libplacebo] have their own.
MPV_REPO := $(shell sed -n '/^\[/q; s/^repo = "\(.*\)"/\1/p' formal/mpv.lock.toml)
MPV_COMMIT := $(shell sed -n '/^\[/q; s/^commit = "\(.*\)"/\1/p' formal/mpv.lock.toml)

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

# Build mpv as a static library with libplacebo from source, into
# formal/.mpv-build. Installs missing apt packages when run as root.
mpv-build: mpv
	formal/build-mpv.sh

# Kernel-check every proof, reject invalid or stale coverage claims, then run
# the model against real mpv code and confirm planted bugs are caught.
proofs: mpv-build
	cd formal && lake build --wfail
	cargo run -q --release --locked -p mpv-cover -- check
	cargo run -q --release --locked -p mpv-fidelity

cover: mpv-build
	cargo run -q --release --locked -p mpv-cover -- report

# Run the Rust port of mpv's misc/ folder and mpv's compiled C on the same
# generated inputs. `make test` runs these too when the mpv build exists; this
# target builds it first and fails without it.
difftest: mpv-build
	RMPV_REQUIRE_MPV_BUILD=1 cargo test --release --locked -p rmpv-misc-difftest -- --show-output

ci: lint test proofs difftest
