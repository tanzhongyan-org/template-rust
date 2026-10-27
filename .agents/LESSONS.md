# Agent Lessons Learned

This file records operational lessons, invariants, and failure modes observed during development.

## Rust Operational Lessons
- Multi-Tenant Invariant: Every read and write query on `notes` must include `owner_id = $X` to prevent cross-tenant data leaks.
- Zero Panic Policy: Never use `.unwrap()` or `.expect()` in `src/`. Handle errors via `Result<_, AppError>` or `?`.
- SQLx Offline & Migrations: Use runtime `sqlx::query_as::<_, T>` with `FromRow` to avoid compile-time DB coupling, and embed migrations via `sqlx::migrate!()`.