---
format: aep.planning-md/1
id: story:indexed-query-model
kind: story
status: implemented
title: Query durable state without whole-realm hydration
summary: Provide paginated indexed queries and read-after-write consistency from PostgreSQL.
relations:
- decomposes: epic:transactional-authority
- serves: vision:O2
revision: 5
---
## Context

Hydrating every entity into one process cannot support central cross-repository queries or several
service instances.

## Acceptance

PostgreSQL answers paginated entity, relation, history, audit and dependency queries from durable indexes with read-after-write consistency tokens and without hydrating the complete realm.

## Implementation

The PostgreSQL authority now implements bounded entity, relation, history, audit, dependency, point-read, and type queries over ER's durable document indexes. Query-bound opaque cursors prevent cross-query reuse, and command consistency tokens are accepted only in their canonical sequence form on the same primary.
