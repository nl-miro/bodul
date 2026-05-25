.PHONY: fmt check test migrate run process entry-error-tracing truncate-dev truncate-test

DATABASE_URL ?= postgres://postgres:postgres@localhost:26001/retailer_sourcing
DATABASE_URL_TEST ?= postgres://postgres:postgres@localhost:26001/retailer_sourcing_test

fmt:
	cargo fmt

check:
	cargo check

test:
	cargo test

migrate:
	DATABASE_URL=$(DATABASE_URL) cargo run --bin migrate
	DATABASE_URL_TEST=$(DATABASE_URL) cargo run --bin migrate

run:
	cargo run --bin retailer-sourcing

process:
	cargo run --bin process-inbox
	cargo run --bin process-commands
	cargo run --bin process-events

entry-error-tracing:
	DATABASE_URL=$(DATABASE_URL) TEST_DATABASE_URL=$(DATABASE_URL_TEST) cargo run --bin entry-error-tracing

truncate-dev:
	DATABASE_URL=$(DATABASE_URL) cargo run --bin truncate-dev

truncate-test:
	TEST_DATABASE_URL=$(DATABASE_URL_TEST) cargo run --bin truncate-test
