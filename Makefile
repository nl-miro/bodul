.PHONY: align-markdown-table-columns wip fmt check test up down reset run process process-test entry-error-tracing truncate-dev truncate-test

align-markdown-table-columns:
	./etc/dev/align-markdown-table-columns/align-markdown-table-columns .

wip:
	git add . && git commit -am 'wip'

up:
	docker compose up -d --wait
	cd apps/retailer-sourcing && make migrate

run:
	cd apps/retailer-sourcing && cargo run --bin retailer-sourcing

process:
	cd apps/retailer-sourcing && make process

process-test:
	cd apps/retailer-sourcing && make process-test

entry-error-tracing:
	cd apps/retailer-sourcing && make entry-error-tracing

truncate-dev:
	cd apps/retailer-sourcing && make truncate-dev

truncate-test:
	cd apps/retailer-sourcing && make truncate-test

down:
	docker compose down

reset:
	docker compose down -v
	docker compose up -d --wait
	cd apps/retailer-sourcing && make migrate
	cd apps/retailer-sourcing && make migrate_test

fmt:
	cd apps/retailer-management && make fmt
	cd apps/retailer-sourcing && make fmt
	cd apps/retailer-data-ingestion && make fmt
	cd apps/product-information-management && make fmt
	cd apps/retailer-offer && make fmt
	cd lib/retailer-guild && make fmt
	cd lib/retailer-parsing && make fmt

check:
	cd apps/retailer-management && make check
	cd apps/retailer-sourcing && make check
	cd apps/retailer-data-ingestion && make check
	cd apps/product-information-management && make check
	cd apps/retailer-offer && make check
	cd lib/retailer-guild && make check
	cd lib/retailer-parsing && make check

test:
	cd apps/retailer-management && make test
	cd apps/retailer-sourcing && make test
	cd apps/retailer-data-ingestion && make test
	cd apps/product-information-management && make test
	cd apps/retailer-offer && make test
	cd lib/retailer-guild && make test
	cd lib/retailer-parsing && make test
