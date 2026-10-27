# Rust Runtime Agent Guidelines (L0)

## 1. Universal Architectural Invariants
- **Standard Rust Project Layout:**
  - `src/main.rs`: Binary entry point & CLI wiring.
  - `src/lib.rs`: Library module declarations and public interface.
  - `src/domain/`: Cohesive domain boundary colocating types, logic, and tests.
  - `src/core/config.rs`: Validated configuration. Direct access to `std::env::var` in domain logic is strictly banned.
- **Search-Before-Author:** Search existing modules before authoring helpers or algorithms. Parameterize existing code; fork-and-duplicate is strictly prohibited.
- **Error Handling:** Use typed error enums (`thiserror` / `anyhow`) over `.unwrap()` or `.expect()` in domain paths.

## 2. Directory Layout
```
src/
├── main.rs                      # Binary entry point
├── lib.rs                       # Root library module
├── domain/                      # Cohesive Domain Boundaries (Colocated)
│   ├── mod.rs
│   └── <feature>.rs             # Domain logic, models & sister tests
└── core/                        # Infrastructure & Config
    ├── mod.rs
    └── config.rs                # Validated environment configuration
```

## 3. The 6 Verification Gates
Before claiming task completion or creating PRs, all gates must pass:
1. **Typecheck & Compile:** `cargo check`
2. **Lint & Format:** `cargo clippy -- -D warnings` and `cargo fmt --check`
3. **Tests:** `cargo test`
4. **Dead Code:** `cargo machete` or clean compiler warnings
5. **Duplication Audit:** `npx --yes fallow@3.33.1 dupes --quiet --no-fragments`
6. **Anti-Slop Scan:** `npx --yes --package @ast-grep/cli ast-grep scan`
