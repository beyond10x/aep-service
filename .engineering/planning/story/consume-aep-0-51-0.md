---
format: aep.planning-md/1
id: story:consume-aep-0-51-0
kind: story
status: implemented
title: AEP Service consumes AEP 0.51.0
summary: Re-pin the six aep crates from 0.45.0 to 0.51.0 and follow any API change between them.
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: crates
revision: 5
---
# Story: AEP Service consumes AEP 0.51.0

## Context

`Cargo.toml` pins aep-client, aep-backend-entity, aep-backend-postgres, aep-contract, aep-domain
and aep-project at tag 0.45.0. AEP 0.51.0 (2026-09-04) moved crates under area directories and
renamed adp-domain, aop-domain and protocol-cli; none of the six pinned crates changed name, and
git dependencies resolve by package name, so the re-pin is a tag change plus whatever API moved
between 0.45.0 and 0.51.0 (`aep/CHANGELOG.md` sections 0.46.0–0.51.0). AEP's entity-runtime pin is
now 0.17.6; `cargo xtask`-style dependency checks here may require one Entity Runtime generation.

## Acceptance

The six pins read `tag = "0.51.0"`, the lockfile resolves them from the area-qualified paths, and
`task check` exits 0; the HTTP conformance corpus and generated OpenAPI still derive from the
released contract.

## Notes

Cross-repository: aep 0.51.0 is released (e6a3118). Devcenter composes AEP Service; its re-pin is
a later story in devcenter.
