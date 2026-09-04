---
format: aep.planning-md/1
id: story:consume-aep-0-53-0
kind: story
status: active
title: AEP Service consumes AEP 0.53.0
summary: Move the hosted service and its deterministic gate onto the current released AEP contract.
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: .github/workflows/gate.yml
- confidence: cited
  path: .github/workflows/pages.yml
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Dockerfile
- confidence: cited
  path: crates/aep-service/tests/dependency_generation.rs
- confidence: cited
  path: website/static/openapi.json
revision: 9
---
## Outcome

AEP Service consumes the current released AEP 0.53.0 contract and its release gate uses the same CLI generation without depending on mutable global Cargo binaries.

## Acceptance

- Every AEP workspace dependency resolves from the exact `0.53.0` tag.
- The dependency-policy gate proves one coordinated AEP generation remains in the graph.
- CI installs the pinned AEP CLI into a run-local binary root, so a restored older global binary cannot block or contaminate the gate.
- The standalone image contains the exact Cargo-locked AEP definition tree at `/definitions`.
- The service gate, site build, and tagged release complete for version 0.1.8.

## Scope

- `Cargo.toml` and `Cargo.lock`
- `.github/workflows/gate.yml`
- `CHANGELOG.md`

