---
format: aep.planning-md/1
id: epic:identity-and-access
kind: epic
status: draft
title: Identity, delegation and access
summary: Derive trusted actor/executor attribution and enforce workspace-scoped permissions on commands and reads.
relations:
- decomposes: initiative:central-aep-authority
- serves: vision:O1
revision: 2
---
# Epic: Identity, delegation and access

## Outcome

A human or delegated agent sees and changes exactly the work its verified authority permits, and
every decision distinguishes the authorizing actor from the executor that ran it.

## Scope

Trusted-principal mapping, workspaces, roles, delegated scopes, command authorization, query
filtering and security refusals. Token issuance and SSO UX remain outside the service.

## Done When

An agent cannot enlarge its owner's authority, restricted entity existence cannot be inferred
through queries or projections, and every authenticated denial names the deciding rule for auditors.

## What is already delivered, and where it is recorded

`README.md:8` says hosted human Identity verification is implemented. It is, and it is tested — but
none of this epic's three stories is what delivered it, which is why they are all still `draft`.

The delivered half is `story:trusted-command-context` (`implemented`, decomposing
`epic:application-service-boundary`): the service constructs actor, executor, request identity and
recorded time only from verified server context. `crates/aep-service/src/main.rs:167-321` is the
hosted `IdentityVerifier` — it resolves the bearer at the configured origin with an exact audience
header, refuses a cacheable credential response, refuses a session outside the configured tenant,
and derives the actor by digest rather than from the raw subject.
`crates/aep-service/src/main.rs:791-834`,
`hosted_identity_derives_the_actor_and_refuses_another_tenant`, stands up a stub Identity authority
and asserts both outcomes; `cargo test --workspace --locked` runs it inside `task check`
(`Taskfile.yml:16`), which the `gate` job runs (`.github/workflows/gate.yml:64`) and which reported
success at `83d447d` in run 34914495607. CHANGELOG `0.1.5` records it shipping.

What this epic still owes, all of it unbuilt at `ed4ef5b`:

- `story:delegated-agent-authority` — `README.md:9` states that delegated-agent proof verification
  "remains deliberately absent"; there is no delegation-scope intersection to test.
- `story:authorized-reads` — realm/workspace admission happens before dispatch
  (`crates/aep-service-http/src/lib.rs:89`, `crates/aep-service-postgres/src/lib.rs:149`), but the
  acceptance also covers snapshot export, and `story:consistent-snapshot-export` is `draft`.
- `story:workspace-scoped-identities` — realm and workspace are enforced and tested
  (`crates/aep-service-auth/src/lib.rs:144-160`), but "canonical references remain unambiguous
  across repositories" belongs to `story:cross-repository-oversight`, which is `draft`.

`story:mcp-agent-access`, which `docs/roadmap.md:42` also places in this wave, is untouched: `mcp`
appears zero times under `crates/`.

Recorded 2026-09-15 by the org-state review run-2 fix pass (ORG-0058). No status moved, because
`README.md:8` is accurate and the work it describes is already `implemented` under another epic.
