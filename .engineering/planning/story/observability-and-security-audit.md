---
format: aep.planning-md/1
id: story:observability-and-security-audit
kind: story
status: draft
title: Observe operation without leaking entity data
summary: Correlate service health, command decisions and security refusals with safe structured telemetry.
relations:
- decomposes: epic:operable-service
- serves: vision:O1
- serves: vision:O6
revision: 1
---
## Context

Operators need to distinguish unavailable dependencies, domain refusals and attacks without copying
restricted entity bodies into logs or metrics.

## Acceptance

Structured telemetry correlates requests, commands and decisions, reports latency and failure classes, exposes no token or restricted body data, and supports an audit query for authenticated security refusals.

