# Rust Runtime Agent Guidelines (L1 Extension)

Inherits: Universal L0 Invariants (workflow/AGENTS.md).
Do not repeat global workflow rules here.

## 1. Project Technology Stack
- Runtime: Rust 2021 Edition
- Crates: Lib and Bin crate structure with serde and chrono
- Lint & Verification: cargo check, cargo clippy, cargo fmt, cargo test

## 2. Physical File Placement Rules
- Entry Point: Binary CLI wiring in src/main.rs
- Public Library: Module declarations in src/lib.rs
- Domain Modules: Place business logic and models in src/domain/<feature>.rs
- Configuration: Environment loader in src/core/config.rs
- Test Suites: Colocated #[cfg(test)] modules in domain source files

## 3. Data Access & Boundary Invariants
- Direct access to std::env::var in domain logic is strictly banned. Load config via core::config.
- Use typed error enums over unwrap() or expect() in production domain paths.

## 4. The 6 Verification Gates
Run these commands to prove task completion:
1. Gate 1 (Compile & Typecheck): cargo check --all-targets
2. Gate 2 (Format & Clippy): cargo fmt --check && cargo clippy --all-targets -- -D warnings
3. Gate 3 (Tests): cargo test
4. Gate 4 (Dead Code): cargo machete (or compiler dead_code checks)
5. Gate 5 (Duplication): npx --yes fallow@3.33.1 dupes --quiet --no-fragments
6. Gate 6 (Anti-Slop): npx --yes --package @ast-grep/cli ast-grep scan
