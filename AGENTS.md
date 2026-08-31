# AGENTS.md — aep-service

The contract for changing this repository. Read it before changing anything.

Org-wide naming, language, cross-repository migration and evidence rules live in `atlas/AGENTS.md`.

## Serves

- **O1 — governed reach.** Human and delegated-agent authority is authenticated, narrowed and
  checked at every command and query boundary; refused attempts are named and recorded.
- **O2 — decisions as data, with evidence.** AEP commands are evaluated centrally and their state,
  events, evidence, accepted decisions and refusals are persisted transactionally through Entity
  Runtime providers.
- **O6 — self-improvement, built into all of it.** The service keeps an attributable activity record
  and exposes deterministic snapshots from which agents and reviewers can measure what changed.

A change that moves none of these is a question for the operator, not a task.

## What this repository is

One deployable Rust service and the internal crates that compose it. It is the authenticated remote
implementation of `engineering-protocols`' `CommandService` and `QueryService`, backed by
`entity-runtime` stores. It is not the owner of the AEP vocabulary, an identity provider, a raw
Entity Runtime store endpoint, a Jira replacement UI, or a second Markdown persistence model.

## Normative documents

- [`docs/VISION.md`](docs/VISION.md) — the product boundary and success condition.
- [`docs/roadmap.md`](docs/roadmap.md) — the order in which the service becomes authoritative.
- `.engineering/planning/` — the governed initiative, epics and stories; mutate it only with
  `protocol artifact`.

Until an invariant below has executable evidence, its planning story is the claim—not the empty
crate scaffold. Do not describe planned enforcement as shipped behaviour.

## Architectural boundaries

1. **Public clients submit AEP intentions, never ER decisions or raw store commits.** The public
   boundary is the EP-owned command/query wire; storage is private implementation detail.
2. **The server constructs trusted attribution.** Actor, executor, roles, request identity and
   recorded time are derived from verified request context. A request payload cannot assert them.
3. **Delegation only narrows.** An agent's effective grant is the intersection of the authority's
   current grants, the delegated token scopes and executor restrictions; it cannot exceed any one.
4. **Authorization and domain validity are different decisions.** Permission to attempt an
   operation does not make the lifecycle operation legal, and lifecycle legality does not grant
   access.
5. **One accepted command is one database transaction.** Entity state, relations, events, audit,
   refusal records and idempotency memory do not land in a partially visible sequence.
6. **No hydrated process copy is authoritative.** Every command reads the revisions it decides on
   inside its transaction; several service processes may safely share one PostgreSQL authority.
7. **Definitions are immutable by identity.** A bundle is addressed by version and digest; activation
   and instance migration are recorded operations, never replacement of bytes under an old name.
8. **Authorization precedes materialization.** Queries, histories, relation traversal and projection
   snapshots are scoped before bytes leave the service; render-then-redact is not accepted.
9. **PostgreSQL is not a client API.** Ordinary users, agents and `protocol` never receive database
   credentials or bypass the service.
10. **Anything that runs is Rust.** New gates, migrations, generators, servers and CLIs are Rust;
    command-line surfaces use `clap` derive when they acquire arguments.

## Repository boundaries

- `engineering-protocols` owns semantic command/query types, their versioned service wire and the
  official client used by `protocol`; this repository implements that contract.
- `entity-runtime` owns the kernel and generic provider contracts; this repository supplies AEP
  application orchestration and deployment policy above them.
- `identity` or another trusted issuer supplies authenticated claims. This repository verifies and
  maps them; it does not mint human identities.
- A change to wire bytes, token audiences or another repository's verified fixture is coordinated
  through an atlas ADR and a new contract version.
- Company data, tokens, private transcripts and production configuration never enter this tree.

## Gate

```console
task check
```

The gate checks formatting, clippy with warnings denied, tests, rustdoc and
`protocol artifact validate`. Cargo commands use `--locked`. Add a new step to this one task rather
than creating a second definition of a green build.

## Planning artifacts

Planning items live under `.engineering/planning/<kind>/<slug>.md` and are owned by the `protocol`
CLI. Before the first store write in a session run `protocol artifact list`.

1. Create, relate, edit bodies and move artifacts only through `protocol artifact`.
2. Change status only with `protocol artifact move`.
3. New artifacts remain in their lifecycle's initial status unless the operator asked for a move.
4. After a batch run `protocol artifact validate` and relay its output verbatim.
5. A CLI refusal is the result; do not route around it by editing frontmatter.

## Dependencies

Dependencies on EP and ER will be pinned releases declared once in the workspace root when their
first consuming stories land. No crate independently selects another tag. Prefer no new dependency;
justify one beside its workspace declaration.

## Changelog and commits

Maintain `CHANGELOG.md` under `## [Unreleased]` for behaviour a service operator or client observes.
Use conventional commit prefixes and a body explaining what changed and why. Do not commit or push
unless the operator asks.

