# Agent Lessons

Load this file only when a gate fails or a pain point matches. Cap: 200 lines / 25 KB; archive older rows to `docs/archive/`.

## Pain-point rules

| # | Rule | Owner | Detection |
|---|---|---|---|
| 1 | Every read and write query on `notes` includes `owner_id = $N`. | `src/api/notes.rs` | `tests/notes_test.rs` (cross-tenant read and delete return 404) |
| 2 | No `unwrap()` or `expect()` in `src/`; return `Result<_, AppError>` and use `?`. | `src/` | clippy `unwrap_used`, `expect_used` |
| 3 | Use runtime `sqlx::query_as` with `FromRow` so the build needs no live database. | `src/api/` | review-only |
| 4 | Never default a missing identity to a shared tenant; return 401. | `src/api/extract.rs` | `tests/security_test.rs` |
| 5 | Never use `CorsLayer::permissive()`; build the layer from `CORS_ALLOWED_ORIGINS`. | `src/api/mod.rs` | `tests/security_test.rs` |
| 6 | Never hard-code a tool version in a workflow or a toolchain in CI; read the pinned file. | `.github/workflows/` | `make policy` (version drift), Renovate |

## Incident log

| Date | Task | Mistake | Fix | Detection |
|---|---|---|---|---|
| 2026-10-27 | Fleet remediation | an or-true suffix after `cargo machete` hid Gate 4 findings | Removed the suffix; the gate is blocking | `make dead-code` |
| 2026-10-27 | Fleet remediation | Hook settings used keys no agent reads, so no hook ran | Switched to `PostToolUse`/`Stop` and `AfterTool`/`AfterAgent` | Hook fires in a live session |
