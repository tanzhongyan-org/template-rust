# Rust Web Service Template

Production-grade Rust web microservice boilerplate powered by Tokio and Axum, featuring structured configuration, centralized error handling, agentic quality guardrails, and automated verification gates.

## Features

- **Web Stack:** [Axum](https://github.com/tokio-rs/axum) 0.8 + [Tokio](https://tokio.rs) async runtime
- **Middleware:** `tower` and `tower-http` (HTTP request tracing, CORS)
- **Serialization & Time:** `serde`, `serde_json`, `chrono`
- **Observability:** `tracing` and `tracing-subscriber` (structured logging via `RUST_LOG` / `LOG_LEVEL`)
- **Error Handling:** Central `AppError` using `thiserror` and `anyhow` implementing `IntoResponse`
- **Configuration:** Strongly typed `AppConfig` loaded from environment variables (`HOST`, `PORT`, `APP_NAME`, `APP_ENV`, `LOG_LEVEL`)
- **Testing:** Unit tests and integration tests with `tower::ServiceExt` and live `axum::serve` listeners
- **Agent Guardrails & Verification:**
  - `Makefile`: Universal standard targets (`install`, `dev`, `test`, `lint`, `lint-fix`, `check`)
  - `lefthook.yml`: Sub-second local Git pre-commit hooks
  - `fallow.jsonc`: AST-aware code duplication and cognitive complexity limits
  - `sgconfig.yml` + `.ast-grep/`: Automatic detection of placeholder stubs and unfulfilled code
  - `.github/workflows/ci.yml`: Automated CI running the 6 Universal Verification Gates
  - `AGENTS.md`: Runtime-specific agent invariants

## Getting Started

### 1. Create a New Repository
Click **"Use this template"** on GitHub or use the GitHub CLI:
```bash
gh repo create my-rust-service --template tanzhongyan-org/template-rust --private --clone
cd my-rust-service
```
*(On first push to `main`, GitHub Actions automatically renames all template placeholders to your new repository name and self-deletes the initialization workflow).*

### 2. Local Setup & Development
```bash
# Run using Makefile
make dev

# Or directly with Cargo
cargo run
```
The service starts listening on `http://0.0.0.0:8000`.

### 3. Endpoints

- `GET /health`: Returns JSON health status, current timestamp, and service version:
  ```json
  {
    "status": "ok",
    "timestamp": "2026-10-27T07:00:00Z",
    "version": "0.1.0"
  }
  ```

### 4. Run Verification Gates
```bash
# Universal check target
make check

# Or individual gate commands:
# Gate 1: Typecheck & Compile
cargo check --all-targets

# Gate 2: Format & Lint
cargo fmt --check
cargo clippy --all-targets -- -D warnings

# Gate 3: Test
cargo test

# Gate 5: Duplication Audit
npx --yes fallow@3.33.1 dupes --quiet --no-fragments

# Gate 6: Anti-Slop Scan
npx --yes --package @ast-grep/cli ast-grep scan
```
