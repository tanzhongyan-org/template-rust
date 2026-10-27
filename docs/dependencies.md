# Dependency ledger

Every direct dependency in `Cargo.toml` has one row. Add the row in the same pull
request that adds the dependency. `make policy` fails when the two disagree.

| Dependency | Purpose | Why the standard library is not enough | Replaced |
| --- | --- | --- | --- |
| `tokio` | Async runtime, TCP listener, signal handling | The standard library has no async executor | Nothing |
| `axum` | HTTP routing, extractors, JSON responses | A hand-written HTTP server would repeat what hyper and axum already test | Nothing |
| `tower-http` | Request tracing, CORS allow-list, request timeout | These layers are small, tested and compose with axum | `CorsLayer::permissive()` use, hand-written timeout |
| `serde` | Derive (de)serialization for request and response types | No derive-based serialization in the standard library | Nothing |
| `serde_json` | JSON bodies and the error envelope | JSON parsing is not in the standard library | Nothing |
| `chrono` | `DateTime<Utc>` timestamps in rows and responses | `std::time` has no calendar or ISO 8601 formatting | Nothing |
| `tracing` | Structured log events | `println!` has no levels, fields or spans | Nothing |
| `tracing-subscriber` | Log filtering (`LOG_LEVEL`) and formatting | `tracing` needs a subscriber to emit anything | Nothing |
| `thiserror` | Derive for `AppError` | Removes boilerplate `Display` and `From` impls | Nothing |
| `anyhow` | Opaque error carrier for `AppError::Internal` and `main` | Keeps internal causes out of the public error type | Nothing |
| `sqlx` | Postgres pool, queries and embedded migrations | No database driver in the standard library | Nothing |
| `uuid` | Note identifiers (v4) | UUID generation is not in the standard library | Nothing |
| `tower` | `ServiceExt::oneshot` in integration tests (dev only) | Calls the router without binding a socket | Nothing |
