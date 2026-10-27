# 0002. Axum, SQLx and Tokio for the service stack

Status: Accepted

## Context

The template needs an HTTP service with a Postgres database that a derived
project can extend without replacing the foundations. The stack must run on
stable Rust, keep the dependency count low and make misuse a compile error where
possible.

## Decision

- Axum on Tokio for HTTP routing and extractors. Handlers share behavior through
  extractors (`OwnerId`, `ApiJson`) and one `AppError` type.
- SQLx with runtime-checked queries (`query_as` and `FromRow`) and embedded
  migrations. Runtime queries keep the build independent of a live database.
- Tower-HTTP for tracing, the CORS allow-list and the request timeout.
- One crate with a library (`src/lib.rs`) and a thin binary (`src/main.rs`) so
  integration tests call the router directly.

## Consequences

- Tests need Postgres, and `make up` starts it.
- Query typos surface at test time, not compile time.
- A second deployable that shares this code means splitting the library into a
  workspace crate. That split needs a new ADR.
