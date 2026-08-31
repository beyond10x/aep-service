---
title: Commands and queries
description: The semantic write and bounded read surfaces exposed by AEP Service.
---

# Intent goes in; attributable outcomes come back

The public API is deliberately semantic. A client never submits a provider batch, Entity Runtime
decision or generic `PATCH status`. It names an AEP command whose payload carries the facts needed
to decide that intention.

## Command envelope

Every command request carries:

| Member | Purpose |
|---|---|
| `command_id` | Logical identity of this command |
| `idempotency_key` | Authority-scoped retry identity |
| `command_type` | Versioned semantic name, such as `aep.entity.create/v1` |
| `target` | Entity being changed, or explicit `null` for creation |
| `expected_revision` | Optimistic guard, or explicit `null` |
| `correlation_id` | Wider activity connecting related work |
| `causation` | Direct cause, or explicit `null` |
| `execution_id`, `task` | Optional governed execution context, written explicitly as `null` when absent |
| `payload` | Strict semantic command document |

Actor, executor, roles, request identity and received time are absent by design. The server derives
them after credential verification.

The command result distinguishes `accepted`, `replayed`, and any other contract outcome. It returns
affected entity revisions, event and audit identities, plus the consistency token for subsequent
reads.

## Query families

The current wire exposes bounded questions rather than a general query language:

- read an entity by identity;
- resolve an `ep://` locator;
- query entities by type, organization, space, exact body fields or relation;
- query typed relations;
- read complete or paginated entity history;
- query attributable audit records; and
- describe the active definition of one entity type.

Every structured query uses strict JSON: unknown members are refused. Nullable members are still
required where the wire needs to distinguish “the client explicitly asked for no filter” from a
malformed omitted field.

## Pagination

Paginated queries accept a positive `limit` or explicit `null` for the server default, plus an
opaque `after` cursor. Clients must replay the cursor exactly and must not parse it as a stable
offset contract.

## Relations

Relations are first-class typed records, not embedded links hidden inside body data. Query outgoing
edges with `source`, incoming edges with `target`, and optionally narrow by relation kind. Creation
and removal remain semantic commands so audit and history stay complete.

Browse the [API reference](/aep-service/api) for schemas and canonical examples, then read the
[HTTP contract](./http-contract) for negotiation and authentication details.
