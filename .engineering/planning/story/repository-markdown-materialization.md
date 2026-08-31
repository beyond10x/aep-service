---
format: aep.planning-md/1
id: story:repository-markdown-materialization
kind: story
status: draft
title: Adopt service-authoritative Markdown materialization
summary: Render and drift-check repository Markdown without accepting it as canonical input.
relations:
- decomposes: epic:projections-and-adoption
- serves: vision:O2
revision: 1
---
## Context

Git review remains useful during the authority migration, but editing generated Markdown must not
become a second canonical write path.

## Acceptance

An adopting repository can render and drift-check its owned Markdown tree from a service snapshot, while manual Markdown edits are reported as drift and cannot alter canonical service state.

