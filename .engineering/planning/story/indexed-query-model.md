---
format: aep.planning-md/1
id: story:indexed-query-model
kind: story
status: draft
title: Query durable state without whole-realm hydration
summary: Provide paginated indexed queries and read-after-write consistency from PostgreSQL.
relations:
- decomposes: epic:transactional-authority
- serves: vision:O2
revision: 1
---
## Context

Hydrating every entity into one process cannot support central cross-repository queries or several
service instances.

## Acceptance

PostgreSQL answers paginated entity, relation, history, audit and dependency queries from durable indexes with read-after-write consistency tokens and without hydrating the complete realm.

