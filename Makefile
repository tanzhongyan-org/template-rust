.PHONY: setup dev test lint fix typecheck check up down build

setup:
	cargo check

dev:
	cargo run

test:
	cargo test

lint:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings

fix:
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged

typecheck:
	cargo check --all-targets

check:
	cargo check --all-targets
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo test
	npx --yes --package @ast-grep/cli ast-grep scan
