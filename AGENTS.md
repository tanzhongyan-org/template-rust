# Rust Web Service Agent Guidelines (L1 Extension)

Inherits: Universal L0 Invariants (workflow/AGENTS.md).
Do not repeat global workflow rules here.

## 1. Project Technology Stack
- Runtime: Rust 2021 Edition
- Web Framework: Axum 0.8 with Tokio 1.43 async runtime
- Middleware: Tower and Tower-HTTP (tracing, CORS)
- Serialization & Time: Serde, Serde JSON, Chrono
- Logging & Observability: Tracing and Tracing-Subscriber
- Error Handling: Central typed `AppError` with `thiserror` and `anyhow` implementing `IntoResponse`
- Lint & Verification: `cargo check`, `cargo clippy`, `cargo fmt`, `cargo test`

## 2. Physical File Placement Rules
- Entry Point: Binary server bootstrap and Tokio runtime in `src/main.rs`
- Public Library: Module declarations in `src/lib.rs`
- Transport Handlers: Route handlers in `src/api/routes/<feature>.rs` and router wiring in `src/api/mod.rs`
- Domain Modules: Place business logic and models in `src/domain/<feature>.rs`
- Configuration: Strongly typed environment loader in `src/core/config.rs` (`AppConfig`)
- Error Types: Central error enum in `src/core/error.rs` (`AppError`)
- Integration Tests: High-level test suites in `tests/<feature>_test.rs`
- Unit Tests: Colocated `#[cfg(test)]` modules in domain and handler source files

## 3. Data Access & Boundary Invariants
- Direct access to `std::env::var` in domain and route logic is strictly banned. Load config via `core::config::AppConfig`.
- Route handlers must return typed responses or `Result<_, AppError>`. Unhandled unwrap() or expect() in request execution paths is strictly banned.
- Central error responses must implement Axum's `IntoResponse` to return consistent JSON errors.

## 4. The 6 Verification Gates
Run these commands to prove task completion:
1. Gate 1 (Compile & Typecheck): cargo check --all-targets
2. Gate 2 (Format & Clippy): cargo fmt --check && cargo clippy --all-targets -- -D warnings
3. Gate 3 (Tests): cargo test
4. Gate 4 (Dead Code): cargo machete (or compiler dead_code checks)
5. Gate 5 (Duplication): npx --yes fallow@3.33.1 dupes --quiet --no-fragments
6. Gate 6 (Anti-Slop): npx --yes --package @ast-grep/cli ast-grep scan
