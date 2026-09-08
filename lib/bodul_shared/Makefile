.PHONY: fmt test check require-clean-worktree package publish

fmt:
	cargo fmt

test:
	cargo test

check:
	cargo check

MARGO_PROJECT := /home/nulllabs/work/labs/margo

PACKAGE := bodul_shared
VERSION := $(shell awk -F'"' '/^version/{print $$2; exit}' Cargo.toml)

# bodul_shared is a workspace member, so `cargo package` writes into the
# workspace root's target dir, not a local one -- ask cargo rather than
# hardcode the relative path up to the workspace root.
TARGET_DIR := $(shell cargo metadata --no-deps --format-version 1 | jq -r .target_directory)

# A `.crate` is only meaningful as a record of a commit. Packaging or
# publishing from a modified tree would ship source that exists on no
# branch, so both stop here first.
require-clean-worktree:
	@dirty="$$(git status --porcelain)"; \
	if [ -n "$$dirty" ]; then \
		echo "refusing: the worktree is dirty; commit or stash first" >&2; \
		echo "$$dirty" >&2; \
		exit 1; \
	fi

package: require-clean-worktree
	cargo package --no-verify

# Depends on `package`, so the `.crate` pushed to the registry is always
# rebuilt from the current commit first.
publish: package
	$(MAKE) -C $(MARGO_PROJECT) add CRATE=$(TARGET_DIR)/package/$(PACKAGE)-$(VERSION).crate
