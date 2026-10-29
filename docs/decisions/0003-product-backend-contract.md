# 0003. Product backend contract: JWKS auth and a committed OpenAPI spec

Status: Accepted

## Context

A backend template serves products whose web app owns sign-in (Better Auth)
and whose TypeScript clients need a stable contract. The template must accept
the product's users without owning credentials, and must publish an API
contract that cannot drift from the code.

## Decision

- Callers authenticate with a product JWT validated against the product JWKS
  (`PRODUCT_JWKS_URL`, RS256 or ES256). The dev-only `x-user-id` header stays
  for local work and refuses to boot in production.
- The OpenAPI document is generated from `utoipa` handler annotations and
  committed at `openapi/openapi.json`. `make openapi` also generates the
  TypeScript client in `clients/ts`. `make openapi-check` fails on any drift.

## Consequences

- Products point the template at their JWKS endpoint and issuer/audience; no
  auth code changes are needed per product.
- Every new route needs an annotation, a regen and a `tests/openapi_test.rs`
  row, or `make check` fails.
- `jsonwebtoken`, `reqwest` and `utoipa` join the dependency closure.
