---
format: aep.planning-md/1
id: story:transactional-command-unit
kind: story
status: draft
title: Commit one command as one transaction
summary: Make all state, relation, event, audit and idempotency effects visible together.
relations:
- decomposes: epic:transactional-authority
- serves: vision:O2
revision: 1
---
## Context

The existing adapter applies in memory and persists a sequence of records, which can latch after a
partial failure and cannot be the authority for a horizontally scaled service.

## Acceptance

One command locks and reads the state it decides on and atomically commits every affected entity, relation, event, audit and applied-command record, with an injected failure leaving none of them visible.

