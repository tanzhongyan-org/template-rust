# Rust Web Service Agent Guidelines (L1 Extension)

Inherits: Universal L0 Invariants (workflow/AGENTS.md).
Do not repeat global workflow rules here. Architecture, code map and invariants live in [ARCHITECTURE.md](ARCHITECTURE.md).
This is the only agent instruction file. Claude Code, Codex, Cursor, Copilot and Gemini read it. Do not add CLAUDE.md, GEMINI.md or other per-tool copies: Claude Code ignores this file once a CLAUDE.md exists. `make policy` rejects them.

## 1. Project Technology Stack

- Runtime: Rust 2024 Edition; the toolchain version is pinned in `rust-toolchain.toml`
- Web: Axum on Tokio; middleware from Tower-HTTP
- Database: SQLx (Postgres) with embedded migrations
- Errors: typed `AppError` implementing `IntoResponse`
- Quality: Makefile, Lefthook, ast-grep, cargo-deny, GitHub Actions

## 2. Where things go

- Find an existing helper in the Code map in [ARCHITECTURE.md](ARCHITECTURE.md) before you write one.
- Handlers live in `src/api/<feature>.rs`; routes are registered in `src/api/mod.rs`.
- Environment variables are read only in `src/core/config.rs`.
- Integration tests live in `tests/<feature>_test.rs`; unit tests sit in `#[cfg(test)]` modules.
- Nest new code freely inside existing top-level directories. A new top-level directory needs a `layout.allow` entry in `scripts/repo-policy.json`; record the reason in `docs/decisions/` if it changes the architecture.
- A new dependency needs a row in `docs/dependencies.md`.

## 3. Security invariants

- Callers are identified only through the `OwnerId` extractor: a product JWT, or `x-user-id` when `TRUST_IDENTITY_HEADER=true`. Never read either directly [enforced by `tests/security_test.rs` and `tests/jwt_test.rs`].
- Every query on `notes` filters by `owner_id` [enforced by `tests/notes_test.rs`].
- `unwrap()` and `expect()` are forbidden outside tests [enforced by clippy].
- `AppError::Internal` never reaches a client [review-only].

## 4. Verification Gates

Run `make verify` after edits and `make check` before you finish:

1. Gate 1 (Typecheck): `cargo check --all-targets`
2. Gate 2 (Lint & Format): `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo deny check bans licenses sources`, markdownlint
3. Gate 3 (Tests): `cargo test`
4. Gate 4 (Dead Code): `cargo machete`
5. Gate 5 (Duplication): `make duplication` (jscpd, zero clones)
6. Gate 6 (Anti-Slop): ast-grep scan and rule tests

`make policy` (layout, docs, dependency ledger, version drift) runs inside both targets. `make check` also runs `make openapi-check`, and CI additionally runs `make licenses` and `make security`.
