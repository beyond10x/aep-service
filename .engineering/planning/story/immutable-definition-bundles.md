---
format: aep.planning-md/1
id: story:immutable-definition-bundles
kind: story
status: draft
title: Register immutable definition bundles
summary: Address validated EP bundles by version and digest and retain every definition referenced by history.
relations:
- decomposes: epic:definition-bundle-lifecycle
- serves: vision:O2
revision: 1
---
## Context

Definitions must be retained as evidence for replay and cannot change beneath an identity already
named by stored decisions.

## Acceptance

The service registers a validated EP bundle under an immutable version and full digest, refuses different bytes under the same identity, and retrieves every definition snapshot referenced by durable history.

