# Contributing

<!-- TEMPLATE-TODO(license): MIT with holder "Tan Zhong Yan". Change the holder in LICENSE if your organization owns the code. -->

## Prerequisites

| Tool | Version source | Install |
| --- | --- | --- |
| Rust (rustc, cargo, clippy, rustfmt) | `rust-toolchain.toml` | [rustup.rs](https://rustup.rs). rustup reads the file and installs the right toolchain. |
| Node | `.node-version` | [nodejs.org](https://nodejs.org). Runs the policy checker and pinned tools. |
| Docker | none | [docs.docker.com](https://docs.docker.com/get-docker/). Runs Postgres. |
| make | none | Your package manager. |
| cargo-deny, cargo-machete, cargo-llvm-cov | `Makefile` | Run `make tools`. |
| gitleaks (optional locally) | The pinned shared workflow (see the `uses:` line in `.github/workflows/ci.yml`) | [github.com/gitleaks/gitleaks](https://github.com/gitleaks/gitleaks#installing). The pre-commit hook and `make security` skip the scan and print a hint without it. |
| osv-scanner | `Makefile` (`OSV_SCANNER_VERSION`) | Downloaded automatically by `make security` into `target/bin`. |

## Set up

```bash
make setup
make up
```

`make setup` compiles the crate and installs the Lefthook git hooks. `make up`
prints `Container ... Healthy` when Postgres accepts connections.

## Daily loop

| Command | When |
| --- | --- |
| `make verify` | After each edit. It needs no database and finishes in seconds once built. |
| `make check` | Before you push. It runs every gate, including tests. CI runs the same command. |

Both end with `verify: ok` or `check: ok`. Any other ending names the failing gate.

## Add a feature

This walkthrough adds a slice named `<feature>`. Copy the shape of the notes slice.

1. Write a failing integration test in `tests/<feature>_test.rs`. Build the app with `app_router`.
2. Add a numbered migration in `migrations/`.
3. Add `src/api/<feature>.rs` with handlers. Take `OwnerId` for the caller and `ApiJson` for bodies. Return `AppError` for failures.
4. Register the module and routes in `src/api/mod.rs`.
5. Filter every query by `owner_id`.
6. Annotate each handler with `#[utoipa::path]`, list it in `src/api/openapi.rs`, run `make openapi`, and extend `tests/openapi_test.rs` with the new route.
7. Run `make check`.
8. Update `ARCHITECTURE.md` (code map) and `README.md` (if configuration or commands change).
9. Add a row to `docs/dependencies.md` for each new dependency.

A new directory needs an ADR in `docs/decisions/` and an entry in `layout.allow` in `scripts/repo-policy.json`, in the same pull request.

## Commits and pull requests

- Use [Conventional Commits](https://www.conventionalcommits.org): `feat(notes): add pagination`. The `commit-msg` hook checks the first line.
- Keep one logical change per commit.
- Open a pull request against `main`. The template asks for gate output and a rollback note.
- The `ci-passed` check must pass before merge.

## Where docs go

See [docs/README.md](../docs/README.md).

## Maintainers

### Import the branch ruleset

`.github/rulesets/main.json` is an export of the `main` branch ruleset. The repository does not apply it for you.

1. Open Settings, then Rules, then Rulesets.
2. Choose **New ruleset**, then **Import a ruleset**, and select the file.
3. Check that `ci-passed` appears as the required status check.

Or use the API:

```bash
gh api repos/OWNER/REPO/rulesets --method POST --input .github/rulesets/main.json
```

### Renovate

Install the Renovate GitHub App on the repository. `.github/renovate.json` groups non-major
updates, waits three days after a release and reads `# renovate:` comments in the
`Makefile` and the workflows.

### Template debt

Run `make template-debt` after you create a repository from this template. Resolve
each `TEMPLATE-TODO` marker and delete it.
