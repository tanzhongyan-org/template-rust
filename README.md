# template-rust

[![CI](https://github.com/tanzhongyan-org/template-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/tanzhongyan-org/template-rust/actions/workflows/ci.yml)
[![CodeQL](https://github.com/tanzhongyan-org/template-rust/actions/workflows/codeql.yml/badge.svg)](https://github.com/tanzhongyan-org/template-rust/actions/workflows/codeql.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A GitHub template for a Rust web service on Axum and Postgres, with a worked
`notes` example and the quality gates already wired. Use it when you start a new
HTTP service and want CI, security scans and agent guardrails from the first commit.

## Quick start

You need Rust (via [rustup](https://rustup.rs)), Docker, Node and `make`.

```bash
gh repo create my-service --template tanzhongyan-org/template-rust --private --clone
cd my-service
make setup
TRUST_IDENTITY_HEADER=true make dev
```

You should see `Application my-service [development] listening on http://0.0.0.0:8000`.
The service reads the process environment only. It does not load `.env` files.

In a second terminal:

```bash
curl -s http://localhost:8000/health
curl -s -H 'x-user-id: alice' http://localhost:8000/notes
```

The first call returns `{"status":"ok","timestamp":"...","version":"0.1.0","environment":"development"}`.
The second returns `[]`.

## What you get

- Axum 0.8 on Tokio, SQLx 0.9 with embedded migrations, one `AppError` type with a single JSON error envelope.
- A `notes` example slice with tenant isolation by `owner_id`.
- Fail-closed identity: no valid caller means `401`.
- Request body limit, request timeout and a CORS allow-list.
- Six verification gates behind `make check`, a fast `make verify`, and Lefthook git hooks.
- CI with a single required check (`ci-passed`), CodeQL, secret and dependency scanning, and a weekly ZAP scan.
- Hooks for Claude Code and Gemini CLI that run `make verify` before an agent finishes a turn.

## After creating your repo

1. Run the **Initialize Template Repository** workflow from the Actions tab. It renames the template placeholders and opens a "Template follow-ups" issue.
2. Run `make template-debt` to list every `TEMPLATE-TODO` marker you must resolve: authentication, code owners, security contact, license holder and the example notes slice.

## Configuration

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `HOST` | No | `0.0.0.0` | Address to bind. |
| `PORT` | No | `8000` | Port to bind. |
| `APP_NAME` | No | `template-rust` | Name shown in startup logs. |
| `APP_ENV` | No | `development` | Environment label returned by `/health`. |
| `LOG_LEVEL` | No | `info` | Log filter. `RUST_LOG` takes precedence when set. |
| `DATABASE_URL` | No | `postgres://postgres:password@localhost:5432/template_rust` | Postgres connection string. |
| `TRUST_IDENTITY_HEADER` | No | `false` | When `true`, the `x-user-id` header identifies the caller. Otherwise every protected route returns `401`. |
| `CORS_ALLOWED_ORIGINS` | No | empty | Comma-separated origins allowed by CORS. Empty sends no CORS headers. |

## Commands

| Command | What it does |
| --- | --- |
| `make setup` | Compile once and install git hooks. |
| `make tools` | Install `cargo-deny` and `cargo-machete` at the pinned versions. |
| `make up` / `make down` | Start or stop Postgres. |
| `make dev` | Start Postgres and run the server. |
| `make test` | Run all tests (needs Postgres). |
| `make verify` | Fast checks without a database: typecheck, lint, anti-slop, policy. |
| `make check` | Every gate, as CI runs it. |
| `make lint` / `make fix` | Check or apply formatting and clippy fixes. |
| `make audit` | Scan dependencies for known vulnerabilities. |
| `make template-debt` | List `TEMPLATE-TODO` markers. |

## Project layout

| Path | Contents |
| --- | --- |
| `src/` | Library and binary. See [ARCHITECTURE.md](ARCHITECTURE.md) for the code map. |
| `migrations/` | SQLx migrations. |
| `tests/` | Integration tests that call the router. |
| `docs/` | Decisions, dependency ledger and documentation rules. See [docs/README.md](docs/README.md). |
| `scripts/` | Policy checker and agent hook. |

More: [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), [LICENSE](LICENSE).
