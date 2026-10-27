# renovate: datasource=crate depName=cargo-deny
CARGO_DENY_VERSION := 0.20.2
# renovate: datasource=crate depName=cargo-machete
CARGO_MACHETE_VERSION := 0.9.2
# renovate: datasource=npm depName=@ast-grep/cli
AST_GREP_VERSION := 0.51.0
# renovate: datasource=npm depName=jscpd
JSCPD_VERSION := 5.4.2
# renovate: datasource=npm depName=lefthook
LEFTHOOK_VERSION := 2.2.2
# renovate: datasource=npm depName=markdownlint-cli2
MARKDOWNLINT_VERSION := 0.24.0

AST_GREP := npx --yes --package @ast-grep/cli@$(AST_GREP_VERSION) ast-grep

.DEFAULT_GOAL := help

.PHONY: help setup tools up down dev test lint fix typecheck dead-code duplication \
	anti-slop policy audit template-debt verify check

help: ## List public make targets
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | sed -E 's/:.*## /\t/' | sort

setup: ## Compile once and install git hooks
	cargo check --all-targets
	npx --yes lefthook@$(LEFTHOOK_VERSION) install

tools: ## Install cargo-deny and cargo-machete at the pinned versions
	cargo install --locked cargo-deny@$(CARGO_DENY_VERSION) cargo-machete@$(CARGO_MACHETE_VERSION)

up: ## Start Postgres and wait until it is healthy
	docker compose up -d --wait

down: ## Stop Postgres
	docker compose down

dev: up ## Start Postgres and run the server
	cargo run

test: ## Run the test suite (needs Postgres)
	cargo test

lint: ## rustfmt, clippy, cargo-deny (bans, licenses, sources), Markdown lint
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
	cargo deny check bans licenses sources
	npx --yes markdownlint-cli2@$(MARKDOWNLINT_VERSION)

fix: ## Apply rustfmt and clippy fixes
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged

typecheck: ## cargo check for all targets
	cargo check --all-targets

dead-code: ## Gate 4: cargo-machete
	cargo machete

duplication: ## Gate 5: jscpd
	npx --yes jscpd@$(JSCPD_VERSION)

anti-slop: ## Gate 6: ast-grep rules and rule tests
	$(AST_GREP) scan
	$(AST_GREP) test --skip-snapshot-tests

policy: ## Layout, docs, dependency ledger and version drift checks
	node scripts/repo-policy.mjs all

audit: ## Dependency vulnerability scan (cargo-deny advisories)
	cargo deny check advisories

template-debt: ## List TEMPLATE-TODO markers a derived repo must resolve
	@git grep -n "TEMPLATE-TODO[(]" || true

verify: typecheck lint anti-slop policy ## Fast checks without a database
	@echo "verify: ok"

check: typecheck lint test dead-code duplication anti-slop policy ## All gates, as run in CI
	@echo "check: ok"
