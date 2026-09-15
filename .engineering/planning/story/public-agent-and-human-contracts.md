---
format: aep.planning-md/1
id: story:public-agent-and-human-contracts
kind: story
status: draft
title: Give agents and humans truthful public entry points
summary: Make AGENTS.md self-contained and README.md useful without private context.
relations:
- decomposes: epic:public-developer-preview
- serves: vision:O2
revision: 2
---
## Context

The current control and orientation documents assume access to the private Atlas repository and do not explain how a public contributor safely changes or runs the service.

## Acceptance

AGENTS.md is a self-contained change contract, README.md leads with developer-preview status and a runnable path, and public architecture documents replace private citations without widening the service boundary.

## What is still owed

Two of the three acceptance clauses hold at `ed4ef5b`. `README.md:8-10` leads with developer-preview
status and `:25-63` is a runnable path; `website/docs/architecture.md` and its siblings carry no
private citation (`grep -rni atlas website/docs/` returns nothing).

The first clause does not. `AGENTS.md:119-137`, the generated `b10x-docs-operations` block, tells a
contributor to "verify the contract with a clean Atlas checkout at the current remote `main`" and
runs `cargo run --manifest-path "$atlas_checkout/Cargo.toml" … docs reconcile --workspace . --check`.
`beyond10x/atlas` is private (`gh api repos/beyond10x/atlas --jq .visibility` → `private`), so a
public contributor cannot execute the documented verification and AGENTS.md is not yet a
self-contained change contract. That is what keeps this story open.

Recorded 2026-09-15 by the org-state review run-2 fix pass; the other four preview stories moved to
`implemented` in the same pass.
