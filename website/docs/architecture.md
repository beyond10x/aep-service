---
title: Architecture and boundaries
description: How AEP Service composes AEP, Entity Runtime and PostgreSQL without leaking their internal interfaces.
---

# One semantic boundary over one transactional authority

AEP Service is a modular Rust application, not a microservice collection. Its internal crates make
trust and transaction boundaries reviewable while producing one deployable process.

```text
┌──────────────────────────────── clients ────────────────────────────────┐
│ protocol CLI · agents · repository projections · oversight dashboards │
└───────────────────────────────┬─────────────────────────────────────────┘
                                │ strict AEP HTTP documents
                                ▼
┌────────────────────────── AEP Service ──────────────────────────────────┐
│ HTTP adapter → credential verifier → application service              │
│                                      │                                 │
│                                      ├─ trusted request context         │
│                                      ├─ realm/workspace admission       │
│                                      └─ fresh command/query handle      │
└──────────────────────────────────────┬──────────────────────────────────┘
                                       │ released provider interfaces
                                       ▼
┌──────────────── Entity Runtime + PostgreSQL ────────────────────────────┐
│ instances · revisions · events · relations · audit · idempotency      │
└─────────────────────────────────────────────────────────────────────────┘
```

## Contract ownership

| Boundary | Owner | What AEP Service consumes |
|---|---|---|
| Semantic vocabulary | AEP | Entity types, commands, queries, strict DTOs and route catalog |
| Deterministic entity mechanism | Entity Runtime | Kernel and generic store/query/PostgreSQL provider interfaces |
| Deployable authority | AEP Service | HTTP, trusted context, authorization, transactions, limits and deployment |

Changing route or DTO bytes starts in AEP and arrives here through a released
pin. Changing provider contracts starts in Entity Runtime. The service may enrich OpenAPI
presentation metadata, but it does not restate a path or payload schema.

## Command data flow

1. The HTTP adapter recognizes the versioned route and media type.
2. The credential verifier produces a principal; credential bytes go no further.
3. Realm and workspace admission happen before semantic document materialization.
4. The server creates request identity and received time and builds trusted command context.
5. A fresh PostgreSQL session reads the revisions the command will decide on.
6. The semantic command either commits its complete candidate record or refuses without mutation.
7. The response publishes a stable outcome, consistency token or typed problem document.

There is no process-wide hydrated realm whose stale state can authorize a write. Multiple service
processes may share the same authority because each command decides against database state inside
its transaction.

## Read data flow

Queries are authorized before result bytes are materialized. Indexed reads apply realm and
workspace scope in the authority; the service never renders a broad result and then redacts it.
Opaque cursors bound pagination without exposing database offsets as a public storage contract.

## Definition boundary

At startup the service loads the configured AEP definition tree, validates it and
compares its sorted source-byte digest with the expected digest. A mismatch prevents readiness.
Definitions are immutable behind that identity; future activation and instance migration are
recorded operations, not replacement of old bytes.

Read [core concepts](./concepts) for tenant/realm/workspace vocabulary and [reliability](./reliability)
for the transactional consequences.
