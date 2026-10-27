.PHONY: install dev test lint lint-fix check

install:
	cargo build

dev:
	cargo run

test:
	cargo test

lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

lint-fix:
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged

check:
	cargo check --all-targets
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo test
	npx --yes fallow@3.33.1 dupes --quiet --no-fragments
	npx --yes --package @ast-grep/cli ast-grep scan
