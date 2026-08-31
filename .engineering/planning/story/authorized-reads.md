---
format: aep.planning-md/1
id: story:authorized-reads
kind: story
status: draft
title: Authorize before reading or materializing
summary: Prevent entity, relation, history and projection side channels for restricted data.
relations:
- decomposes: epic:identity-and-access
- serves: vision:O1
revision: 1
---
## Context

Filtering after graph traversal or Markdown rendering leaks titles, relation existence, counts and
history even when fields are later redacted.

## Acceptance

Entity lookup, listings, histories, relation traversal and snapshot export apply authorization before reading or materializing results and follow one documented policy for concealing restricted existence.

