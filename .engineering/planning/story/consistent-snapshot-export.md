---
format: aep.planning-md/1
id: story:consistent-snapshot-export
kind: story
status: draft
title: Export one consistent authorized snapshot
summary: Give projection clients a stable source position, bundle digest and deterministic ordering.
relations:
- decomposes: epic:projections-and-adoption
- serves: vision:O2
revision: 1
---
## Context

Projection clients need one coherent source position; enumerating while commands continue can create
a tree that never existed.

## Acceptance

An authorized client exports a complete workspace snapshot at one consistency position with bundle and source digests, stable ordering and no volatile rendering metadata.

