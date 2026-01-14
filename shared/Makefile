.PHONY: help fix cfix clippy fmt check build buildr qbuild test qtest run runr lint clean wip

help:
	@echo "Available targets:"
	@echo "  fix      Auto-fix issues with cargo fix"
	@echo "  cfix     Clippy fix with --allow-dirty"
	@echo "  clippy   Lint check with clippy"
	@echo "  fmt      Format code with rustfmt"
	@echo "  check    Check compilation"
	@echo "  build    Build the shared workspace"
	@echo "  buildr   Release build"
	@echo "  qbuild   Quick build (suppress warnings)"
	@echo "  test     Run all tests"
	@echo "  qtest    Quick test (suppress warnings)"
	@echo "  run      Run binaries"
	@echo "  runr     Release run"
	@echo "  lint     Alias for clippy"
	@echo "  clean    Remove build artifacts"
	@echo "  wip      Work-in-progress commit"

fix:
	set -e; cargo fix --lib --allow-dirty --workspace

cfix:
	set -e; cargo clippy --fix --allow-dirty --workspace --all-targets --all-features

clippy:
	set -e; cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
	set -e; cargo fmt --all

check:
	set -e; cargo check --workspace

build:
	set -e; cargo build --workspace

buildr:
	set -e; cargo build --workspace --release

qbuild:
	set -e; env RUSTFLAGS="-Awarnings" cargo build --workspace --target-dir target/q

test:
	set -e; cargo test --workspace

qtest:
	set -e; env RUSTFLAGS="-Awarnings" cargo test --workspace --target-dir target/q

run:
	set -e; cargo run

runr:
	set -e; cargo run --release

lint: clippy

clean:
	cargo clean

wip:
	git add TODO.md && git commit -m '** TODO **' || printf "No todo to commit"
	git add . && git commit -am 'wip'
