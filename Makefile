.PHONY: setup dev test lint fix typecheck check up down build audit

setup:
	cargo check
	npx --yes lefthook@2.2.2 install

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

audit:
	cargo audit

check:
	cargo check --all-targets
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo test
	cargo machete || true
	npx --yes --package @ast-grep/cli@0.51.0 ast-grep scan
	npx --yes --package @ast-grep/cli@0.51.0 ast-grep test --skip-snapshot-tests
