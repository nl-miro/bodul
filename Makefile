.PHONY: fmt test check cargo-fix clippy-fix check-strict docker-build docker-up docker-down db-dump gomd wip package publish

DB_SERVICE ?= db
DB_USER ?= bodul
DB_NAME ?= bodul
DUMP_DIR ?= dumps

fmt:
	$(MAKE) -C apps/mvp fmt
	$(MAKE) -C lib/money fmt
	$(MAKE) -C lib/bodul_shared fmt
	$(MAKE) -C lib/retailer-sourcing fmt

test:
	$(MAKE) -C apps/mvp test
	$(MAKE) -C lib/money test
	$(MAKE) -C lib/bodul_shared test
	$(MAKE) -C lib/retailer-sourcing test

check:
	$(MAKE) -C apps/mvp check
	$(MAKE) -C lib/money check
	$(MAKE) -C lib/bodul_shared check
	$(MAKE) -C lib/retailer-sourcing check

cargo-fix:
	cd apps/mvp && cargo fix --tests
	cd lib/money && cargo fix --tests
	cd lib/bodul_shared && cargo fix --tests
	cd lib/retailer-sourcing && cargo fix --tests

clippy-fix:
	cd apps/mvp && cargo clippy --fix --tests
	cd lib/money && cargo clippy --fix --tests
	cd lib/bodul_shared && cargo clippy --fix --tests
	cd lib/retailer-sourcing && cargo clippy --fix --tests

check-strict:
	RUSTFLAGS="-Awarnings" cargo check

docker-build:
	docker build -t bodul-mvp .

docker-up:
	docker compose up --build

docker-down:
	docker compose down

gomd:
	gomd all .

wip:
	git add . && git commit -am 'wip'

db-dump:
	@mkdir -p $(DUMP_DIR)
	@out="$(DUMP_DIR)/$(DB_NAME)_$$(date +%Y%m%d-%H%M%S).sql"; \
	echo "dumping $(DB_NAME) -> $$out"; \
	docker compose exec -T $(DB_SERVICE) pg_dump -U $(DB_USER) -d $(DB_NAME) > "$$out" \
		&& echo "done: $$out" \
		|| { echo "dump failed (is the '$(DB_SERVICE)' service running? try: make docker-up)"; rm -f "$$out"; exit 1; }


CRATES := $(HOME)/work/labs/crates
CARGO_DIR := lib/bodul_shared

# Name and version come from the manifest, so they cannot drift from what is
# published. Recursive rather than `:=` so `cargo metadata` runs only when a
# target actually reads them, and the $(eval) caches the result after the first
# read. $(shell) swallows failures into an empty string, hence the guard below.
# bodul is one workspace, so `cargo metadata` from CARGO_DIR still returns
# every member -- select by manifest_path rather than assuming there's only
# one package.
CRATE = $(eval CRATE := $(shell cd $(CARGO_DIR) && cargo metadata --no-deps --format-version 1 \
	| jq -er --arg manifest "$$(pwd)/Cargo.toml" \
	          '[.packages[] | select(.manifest_path == $$manifest)] \
	          | if length == 1 then "\(.[0].name) \(.[0].version)" \
	            else error("package selection is not unique") end'))$(CRATE)

PACKAGE = $(word 1,$(CRATE))
VERSION = $(word 2,$(CRATE))

# A `.crate` is only meaningful as a record of a commit. Packaging or publishing
# from a modified tree would ship source that exists on no branch, so both stop
# here first.
require-clean-worktree:
	@dirty="$$(git status --porcelain)"; \
	if [ -n "$$dirty" ]; then \
		echo "refusing: the worktree is dirty; commit or stash first" >&2; \
		echo "$$dirty" >&2; \
		exit 1; \
	fi

package: require-clean-worktree
	@test -n "$(PACKAGE)" -a -n "$(VERSION)" \
		|| { echo "cannot resolve package/version from $(CARGO_DIR)" >&2; exit 1; }
	cd lib/bodul_shared && cargo package --locked -p "$(PACKAGE)" --list
	cd lib/bodul_shared && cargo package --locked -p "$(PACKAGE)" && \
	test -f "target/package/$(PACKAGE)-$(VERSION).crate" && \
	margo add --registry $(CRATES) "target/package/$(PACKAGE)-$(VERSION).crate"

# Depends on `package`, so the `.crate` in the registry is always rebuilt from
# the current commit before it is pushed. Staging names the registry paths this
# crate owns rather than `git add -A`, which would sweep up unrelated edits.
publish: package
	git -C $(CRATES) add -- config.json index.html \
		':(glob)crates/*/*/$(PACKAGE)/*' ':(glob)*/*/$(PACKAGE)'
	git -C $(CRATES) diff --cached --quiet || \
			git -C $(CRATES) commit -m "publish $(PACKAGE) $(VERSION)"
	git -C $(CRATES) push
