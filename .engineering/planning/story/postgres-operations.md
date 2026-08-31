---
format: aep.planning-md/1
id: story:postgres-operations
kind: story
status: draft
title: Operate and recover PostgreSQL safely
summary: Provide forward migrations, backup/restore evidence and credential boundaries.
relations:
- decomposes: epic:operable-service
- serves: vision:O2
- serves: vision:O6
revision: 1
---
## Context

A central record needs schema migration, backup, restore and connection policy before it can replace
repository-local authority.

## Acceptance

Documented and tested procedures migrate the database forward, restore a backup into an isolated instance, verify entity/history consistency, and keep credentials outside repository and client configuration.

