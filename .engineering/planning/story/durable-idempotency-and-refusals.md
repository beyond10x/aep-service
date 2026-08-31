---
format: aep.planning-md/1
id: story:durable-idempotency-and-refusals
kind: story
status: active
title: Persist retries and refusals truthfully
summary: Return original replay results and durably attribute authenticated refusals without changing entity state.
relations:
- decomposes: epic:transactional-authority
- serves: vision:O2
revision: 4
---
## Context

Network retries are normal, and a refusal without an attributable durable record is indistinguishable
from an attempt that never happened.

## Acceptance

Repeating one logical command returns its original result, reusing its idempotency identity for different bytes is refused, and every authenticated domain or authorization refusal is durably attributable without changing entity state.

## Progress

Fresh PostgreSQL command sessions now persist domain decisions, domain refusals, and idempotent replay memory atomically. The story remains active because durable attribution of authenticated authorization refusals belongs to the later identity/authorization wave and is not yet implemented.
