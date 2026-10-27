# Documentation

This directory holds the documents that explain and record the service. The root
holds the living documents: `README.md`, `ARCHITECTURE.md`, `CONTRIBUTING.md` and
`SECURITY.md`.

| Path | What goes there | Lifespan |
| --- | --- | --- |
| `docs/decisions/` | One ADR per irreversible technical choice, named NNNN-slug.md. 20 to 40 lines: Context, Decision, Consequences. | Permanent. Never edit an accepted ADR. Write a new one that declares `Supersedes: NNNN`. |
| docs/specs | A functional spec for a feature: business rules, edge cases, acceptance scenarios. | Medium. Fold the rules into `ARCHITECTURE.md` or tests, then archive or delete. |
| docs/plans | A tactical plan for one change: checklist and requirement ledger. | Short. Commit it, then `git rm` it when the change merges. |
| docs/archive | Retired specs and old lesson rows. | Reference only. |
| `docs/dependencies.md` | The dependency ledger: one row per direct dependency. | Living. Update it with `Cargo.toml`. |

The specs, plans and archive directories do not exist until you need them.
Create the directory with its first document, and keep the path in the table above.

## Add a decision record

1. Copy the shape of `docs/decisions/0001-record-architecture-decisions.md`.
2. Use the next number and a short slug.
3. Link the ADR from the pull request that depends on it.
