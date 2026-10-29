# Security policy

## Report a vulnerability

Report vulnerabilities privately through
[GitHub private vulnerability reporting](https://github.com/tanzhongyan-org/template-rust/security/advisories/new).
Do not open a public issue.

<!-- TEMPLATE-TODO(security): confirm private vulnerability reporting is enabled and add a contact for your team -->

## What to expect

| Step | Target |
| --- | --- |
| Acknowledge your report | 3 business days |
| Triage and severity decision | 10 business days |

## Supported versions

Only the `main` branch receives security fixes.

## Automated checks

Every pull request and the `main` branch run these layers.

| Layer | Tool | Where |
| --- | --- | --- |
| Static analysis in the gates | clippy with `-D warnings`, `cargo deny check bans licenses sources` | `make lint` |
| Cross-language static analysis | CodeQL with the `security-extended` queries (`rust` and `actions`) | `.github/workflows/codeql.yml` |
| Secret scanning | gitleaks over the full history, and staged files in the pre-commit hook | `security` job, `lefthook.yml` |
| Dependency vulnerabilities | `cargo deny check advisories`, dependency review at `high` severity | `make audit`, `dependency-review` job |
| Licence compliance | `cargo deny check licenses` plus a generated `NOTICE` | `make licenses` |
| Supply chain | OSV scan of `Cargo.lock`; every direct dependency must exist, be 14 days old and have a ledger row | `make security` |
| Workflow lint | zizmor at `medium` severity and above | `security` job |
| Dynamic scan | OWASP ZAP baseline against a running server, weekly | `.github/workflows/dast.yml` |

## Built-in protections

- Requests without a valid identity get `401`. Bearer tokens validate against the product JWKS (RS256 or ES256, expiry, issuer and audience enforced). The `x-user-id` header counts only when `TRUST_IDENTITY_HEADER=true`, which production refuses.
- Request bodies over 64 KiB get `413`. Requests that run longer than 30 seconds get `408`.
- CORS allows only the origins in `CORS_ALLOWED_ORIGINS`. An empty list sends no CORS headers.
- Internal errors return `Internal server error` and log the cause server-side.
