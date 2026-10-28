# 0001. Record architecture decisions

Status: Accepted

## Context

Choices such as the web framework, the identity model and the directory layout
outlive the people and the chat sessions that made them. Without a record, the
next contributor guesses the reason or reverses the choice.

## Decision

We record each irreversible technical choice as an ADR in `docs/decisions/`, in
the MADR minimal format: Context, Decision, Consequences. An ADR is 20 to 40
lines. We never edit an accepted ADR. A later ADR that changes a choice declares
`Supersedes: NNNN`.

A new top-level directory or a new directory under `src/` needs an ADR and a
`layout.allow` change in `scripts/repo-policy.json` in the same pull request.

## Consequences

- The reason behind a choice stays one `git log` away from the code.
- Changing a choice costs one short document, which discourages churn.
- Reviewers can ask "where is the ADR?" for structural changes.
