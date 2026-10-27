# Rust Runtime Template

Universal Rust application boilerplate with standard project layout, agentic quality guardrails, and verification gates.

## Features

- **Runtime:** Rust (2021 edition) with standard binary + library crate layout
- **Serialization & Time:** `serde`, `serde_json`, `chrono`
- **Config & Validation:** Centralized environment configuration module
- **Agent Guardrails:**
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

### 2. Local Setup
```bash
cargo run
```

### 3. Run Verification Gates
```bash
# Gate 1: Typecheck & Compile
cargo check

# Gate 2: Format & Lint
cargo fmt --check
cargo clippy -- -D warnings

# Gate 3: Test
cargo test

# Gate 5: Duplication Audit
npx --yes fallow@3.33.1 dupes --quiet --no-fragments

# Gate 6: Anti-Slop Scan
npx --yes --package @ast-grep/cli ast-grep scan
```
