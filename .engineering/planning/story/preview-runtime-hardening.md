---
format: aep.planning-md/1
id: story:preview-runtime-hardening
kind: story
status: draft
title: Bound and identify every preview HTTP exchange
summary: Add finite queue/request/shutdown behaviour, durable request identities and an explicit container-only development override.
relations:
- decomposes: epic:public-developer-preview
- serves: vision:O1
revision: 1
---
## Context

The current body and database concurrency bounds still permit indefinite queueing, have process-restart request-id collision risk, and cannot be reached from a container without removing the development listener guard.

## Acceptance

Queue, request and shutdown waits are finite and configurable; valid overloads are typed unavailable responses; request ids are UUIDv7; SIGINT and SIGTERM drain; non-loopback development serving remains refused unless an explicit noisy override is present; data responses are non-cacheable and CORS stays closed.
