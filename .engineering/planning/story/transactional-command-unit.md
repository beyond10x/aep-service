---
format: aep.planning-md/1
id: story:transactional-command-unit
kind: story
status: implemented
title: Commit one command as one transaction
summary: Make all state, relation, event, audit and idempotency effects visible together.
relations:
- decomposes: epic:transactional-authority
- serves: vision:O2
revision: 6
---
## Context

EP 0.35.0 already projects one complete semantic command into Entity Runtime's atomic batch store,
but the service had no concrete realm-scoped provider or evidence that this composition remains whole
under a late PostgreSQL failure and competing service handles. The EP adapter also hydrates candidate
state, so the service must ensure that no long-lived process copy becomes the authority.

## Acceptance

One command locks and reads the state it decides on and atomically commits every affected entity, relation, event, audit and applied-command record, with an injected failure leaving none of them visible.

## Implementation record — 2026-08-31

`aep-service-postgres::PostgresAuthority` binds the already verified realm/workspace context to a
fresh `aep_backend_postgres::PostgresBackend` for every request. Wave 1 admits exactly one configured
workspace in one realm-scoped PostgreSQL schema, refuses any other scope before connecting, rejects
schema identifiers PostgreSQL would truncate beyond 63 bytes, and redacts the database URL from debug
output.

The service reuses EP 0.35.0's complete command-to-batch projection and Entity Runtime 0.14.0's
PostgreSQL atomic batch provider. The provider reads expected revisions under `FOR UPDATE` inside the
commit transaction; therefore a hydrated candidate is never authoritative and a stale decision is
refused rather than overwriting current durable state. State, relation, event, audit and applied-command
records share that transaction. A new handle is opened for each request so no process-wide hydrated
copy survives as service authority. Whole-realm hydration remains a temporary read cost and is removed
by `story:indexed-query-model`; it is not used to settle concurrent writes.

`transactional_authority.rs` installs a PostgreSQL trigger that fails the final applied-command insert
after the earlier batch writes, then proves all instance categories and events remain at their exact
pre-command counts. Its second test opens two fresh handles at revision 1, races both writers and proves
exactly one reaches revision 2 while the loser names expected revision 1 and found revision 2. Both
cases were run against PostgreSQL 16 as well as remaining conditional on `ENTITY_POSTGRES_URL` for the
repository gate; the gate explicitly reports when that server-backed evidence is skipped.
