.PHONY: help fix cfix clippy fmt check build buildr qbuild test qtest lint clean wip

help:
	@echo "Available targets:"
	@echo "  fix      Auto-fix issues with cargo fix"
	@echo "  cfix     Clippy fix with --allow-dirty"
	@echo "  clippy   Lint check with clippy"
	@echo "  fmt      Format code with rustfmt"
	@echo "  check    Check compilation"
	@echo "  build    Build the elemental crate"
	@echo "  buildr   Release build"
	@echo "  qbuild   Quick build (suppress warnings)"
	@echo "  test     Run all tests for elemental"
	@echo "  qtest    Quick test (suppress warnings)"
	@echo "  lint     Alias for clippy"
	@echo "  clean    Remove build artifacts"
	@echo "  wip      Work-in-progress commit"

fix:
	set -e; cargo fix -p elemental --lib --allow-dirty

cfix:
	set -e; cargo clippy --fix -p elemental --allow-dirty --all-targets --all-features

clippy:
	set -e; cargo clippy -p elemental --all-targets --all-features -- -D warnings

fmt:
	set -e; cargo fmt

check:
	set -e; cargo check -p elemental

build:
	set -e; cargo build -p elemental

buildr:
	set -e; cargo build -p elemental --release

qbuild:
	set -e; env RUSTFLAGS="-Awarnings" cargo build -p elemental --target-dir target/q

test:
	set -e; cargo test -p elemental

qtest:
	set -e; env RUSTFLAGS="-Awarnings" cargo test -p elemental --target-dir target/q

lint: clippy

clean:
	cargo clean

wip:
	git add TODO.md && git commit -m '** TODO **' || printf "No todo to commit"
	git add . && git commit -am 'wip'
