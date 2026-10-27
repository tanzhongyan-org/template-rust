# Rust Web Service Agent Guidelines (L1 Extension)

Inherits: Universal L0 Invariants (workflow/AGENTS.md).
Do not repeat global workflow rules here.

## 1. Project Technology Stack
- Runtime: Rust 2024 Edition (`rust-toolchain.toml` pinned to 1.90.0)
- Web Framework: Axum 0.8 with Tokio 1.43+ async runtime
- Application State: Strongly typed `AppState` shared via `axum::extract::State`
- Middleware: Tower-HTTP (`TraceLayer`, `CorsLayer`)
- Serialization & Time: Serde, Serde JSON, Chrono (minimal features)
- Observability: Tracing and Tracing-Subscriber (`EnvFilter` + `fmt`)
- Error Handling: Typed `AppError` via `thiserror` implementing `IntoResponse`
- Quality Guardrails: Makefile, Lefthook, Ast-Grep, GitHub Actions CI

## 2. Physical File Placement Rules
- Entry Point: Binary server bootstrap and Tokio runtime in `src/main.rs`
- Application State: Central state container in `src/state.rs` (`AppState`)
- Configuration: Strongly typed environment loader in `src/core/config.rs` (`AppConfig`)
- Error Types: Central error enum in `src/core/error.rs` (`AppError`)
- Transport Handlers: Route handlers in `src/api/<feature>.rs` and router wiring in `src/api/mod.rs`
- Domain Modules: Feature business logic in `src/domain/<feature>.rs` (create on-demand)
- Integration Tests: High-level test suites in `tests/<feature>_test.rs`
- Unit Tests: Colocated `#[cfg(test)]` modules in domain and handler source files

## 3. Architecture & Security Invariants
- Configuration Boundary: Direct access to `std::env::var` in domain and route handlers is banned [enforced by ast-grep]. Load configuration via `AppConfig::from_env()` at bootstrap.
- Error Handling: `unwrap()` and `expect()` are forbidden outside tests [enforced by clippy].
- Shared Infrastructure: Handlers requiring shared infrastructure or config must use Axum`s `State(state): State<AppState>` extractor [review-only].
- Graceful Shutdown: Server lifecycle must implement graceful shutdown handling `SIGINT` (Ctrl+C) and `SIGTERM` [review-only].
- Typed Error Responses: Route handlers must return typed responses or `Result<_, AppError>` [enforced by compiler].
- Error Masking: `AppError::into_response` must never leak raw internal errors (`AppError::Internal`) to HTTP clients [review-only].

## 4. Verification Gates
Run `make check` to verify task completion:
1. Gate 1 (Compile & Typecheck): `cargo check --all-targets`
2. Gate 2 (Format & Clippy): `cargo fmt --check && cargo clippy --all-targets -- -D warnings`
3. Gate 3 (Tests): `cargo test`
4. Gate 4 (Dead Code): Compiler `dead_code` analysis under `-D warnings` and `cargo-machete`
5. Gate 5 (Duplication): N/A (TypeScript/Web specific)
6. Gate 6 (Anti-Slop): `npx --yes --package @ast-grep/cli@0.51.0 ast-grep scan && ast-grep test`
