.PHONY: fmt test check cargo-fix clippy-fix check-strict docker-build docker-up docker-down db-dump gomd wip

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
