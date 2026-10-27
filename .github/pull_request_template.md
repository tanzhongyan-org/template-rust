# Pull request

## What and why

<!-- One or two sentences. Link the issue: Closes #123 -->

## How you verified it

<!-- Paste the quiet output of `make check`, or explain why a gate does not apply. -->

```text

```

## Risk and rollback

<!-- What could break? How do you undo it? -->

## Checklist

- [ ] Tests cover the new behavior at a public entry point.
- [ ] Docs change in the same pull request (`README.md`, `ARCHITECTURE.md`, or `docs/`).
- [ ] No new dependency without a row in `docs/dependencies.md`.
- [ ] No new directory without an ADR in `docs/decisions/` and a `layout.allow` change in `repo-policy.json`.
