# AGENTS.md — aep-service

The change contract for this repository. Read it before editing anything; `README.md` is for humans
trying the service, while this file names what an agent must preserve.

## Serves

- **O1 — governed reach.** Authentication-derived authority is checked before command/query work,
  and requests outside it are named refusals.
- **O2 — decisions as data, with evidence.** AEP commands commit state, history and refusal evidence
  through one Entity Runtime-backed transaction.
- **O6 — self-improvement, built into all of it.** Attributable activity and deterministic
  projections make before/after engineering work inspectable by people and agents.

## Mission and boundary

This repository is one deployable Rust service and its internal crates. It implements Engineering
Protocols' `CommandService` and `QueryService` over Entity Runtime PostgreSQL providers. It is not
the owner of AEP vocabulary, an identity provider, a raw Entity Runtime store endpoint, a Jira
replacement UI, or a Markdown persistence model.

The work advances governed reach (O1), decisions with evidence (O2), and attributable feedback
(O6). A change that advances none of those needs an operator decision before implementation.

## Normative project records

- `docs/VISION.md` defines the product boundary and success condition.
- `docs/roadmap.md` orders delivery waves without duplicating status.
- `.engineering/planning/` is the governed work record; only `protocol artifact` may change it.
- `CHANGELOG.md` records user- and operator-visible behavior under `Unreleased` with the change.

Public documentation under `website/docs/` is a curated projection. It must not become a second
definition of the wire or copy private organizational records.

## Invariants

1. Public clients submit AEP intentions, never Entity Runtime decisions, batches, SQL, or provider
   operations. `aep-client`'s released route catalog and strict DTOs are the HTTP source of truth.
2. Actor, executor, roles, request identity, and received time come from verified server context.
   Request documents cannot assert trusted attribution.
3. Delegation only narrows. Effective authority is the intersection of owner grants, delegated
   scopes, and executor restrictions.
4. Authorization precedes decode/materialization where the request scope is sufficient to decide
   it. Permission to attempt and lifecycle legality remain separate decisions.
5. One accepted command is one PostgreSQL transaction, including entity state, events, relations,
   audit, refusal evidence, and idempotency memory. A refusal publishes no partial mutation.
6. Every command reads the revisions it decides on inside its transaction. No hydrated process copy
   is authoritative; multiple processes may share one authority safely.
7. Definition bundles are immutable by digest. Activation and migration are recorded operations,
   never replacement of bytes behind an existing identity.
8. Queries and snapshots are authorized before bytes leave the service. Render-then-redact is not
   accepted.
9. Development bearer authentication is loopback-only unless
   `--allow-insecure-dev-listener` is explicit. That override must remain visibly named and warned.
10. Listener work is bounded: request bodies, execution concurrency, queue wait, exchange duration,
    and graceful shutdown all have explicit limits. Overload uses a typed AEP problem document.
11. `/openapi.json` is generated from EP route/schema APIs. A handwritten payload or path model in
    this repository is a defect.
12. PostgreSQL credentials never reach clients. Credentials, tokens, company data, private
    transcripts, and production configuration never enter this tree.
13. Anything that runs is Rust. Shell is orchestration only; command-line surfaces use `clap`
    derive. No Python scripts.
14. Every public Rust item is documented and no crate uses `unsafe`. All workspace members opt into
    workspace lints.

## Cross-repository changes

Engineering Protocols owns semantic types, strict wire bytes, routes, and the official client.
Entity Runtime owns its kernel and provider interfaces. If bytes another repository verifies, a
token audience, or a provider contract changes, coordinate a versioned migration before updating
the pin here. Workspace dependencies select one exact EP tag and one exact ER tag; crate-local pins
are not allowed.

## Planning store

Before the first planning write in a session run `protocol artifact list`. Create, relate, edit
bodies, record evidence, and move only through `protocol artifact`; never edit planning files. New
items remain at their initial status unless the operator explicitly requested a move. After every
batch run `protocol artifact validate` and relay its output verbatim. A CLI refusal is the result,
not an invitation to edit frontmatter.

## Gate

```console
task check
```

The gate checks formatting, clippy with warnings denied, all tests, rustdoc, planning validity,
OpenAPI determinism and dependency policy. PostgreSQL tests run when `ENTITY_POSTGRES_URL` is set and
print one explicit skipped line otherwise. `task site-build` generates the OpenAPI asset and builds
the Docusaurus site; it is separate because `npm ci` uses the network.

Cargo gate commands use `--locked`. Read the task's exit status directly. Add a new required check
to `Taskfile.yml` and CI together so pull requests and releases cannot exercise shorter gates.

## Implementation conventions

- Tests are named for behavior and assert a typed reason, never only `is_err()`.
- Comments explain why; public docs explain what the item is for.
- Prefer no dependency. Justify every new direct dependency beside its workspace declaration.
- Preserve unrelated work in a dirty tree. Use `apply_patch` for intentional source edits.
- Do not commit or push unless the operator asks. Commits use a conventional prefix, a blank line,
  and a body explaining what changed and why.
