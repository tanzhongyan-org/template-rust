# Architecture Overview

`template-rust` is a production-ready asynchronous Rust web service template powered by Axum, Tokio, and SQLx.

## Physical Directory Structure
- `src/main.rs`: Application bootstrap, signal handling, and runtime configuration.
- `src/state.rs`: Central `AppState` holding configuration and connection pools.
- `src/core/`: Core cross-cutting primitives (`config.rs`, `error.rs`).
- `src/api/`: Axum route controllers and HTTP request/response DTOs.
- `migrations/`: Versioned database migration scripts executed via SQLx embedded migrations.
- `tests/`: Integration tests testing full HTTP routes and database round-trips.

## Tenant Isolation & Security
- All multi-tenant queries on `notes` strictly filter by `owner_id`.
- The `owner_id` is derived from incoming request headers (`x-user-id`) at the transport boundary.
- Production code disallows `unwrap()` and `expect()` via Clippy lints (`[lints.clippy] unwrap_used = "deny"`).