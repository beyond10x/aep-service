---
title: Reliability semantics
description: Atomic commands, revision guards, idempotent retries, consistency, typed refusals and overload behavior.
---

# The failure behavior is part of the API

## Atomic command publication

One accepted command is one PostgreSQL transaction. Entity state, revision history, events,
relations, audit records and idempotency memory become visible together. An injected failure at the
last write rolls the entire candidate back; there is no partial success document.

## Optimistic concurrency

A command that depends on current state carries `expected_revision`. The authority reads and guards
that revision inside the same transaction that writes the successor. When two fresh service
processes race, exactly one may commit; the loser receives `revision_conflict` with expected and
actual revisions.

## Idempotency

`idempotency_key` identifies a retry within the authority:

- identical intent returns the recorded result with outcome `replayed`;
- the retry does not emit events or audit a second accepted command; and
- reusing the key for different intent is a typed conflict.

Retries remain the client’s decision. The problem document publishes `retryable`; do not infer it
only from the HTTP status.

## Consistency tokens

Accepted commands return a token such as `seq:42`. A subsequent query may request `at_least` that
token when read-after-write visibility matters. The current PostgreSQL authority already exposes
the committed command to a fresh read handle.

## Typed problems

Every negotiated problem contains:

```json
{
  "request_id": "0198f03a-7b62-7000-8000-000000000001",
  "error": {
    "code": "revision_conflict",
    "message": "the entity changed since the expected revision",
    "retryable": false,
    "details": {"expected": 12, "actual": 13}
  }
}
```

The stable contract is `code`, `retryable`, and the code-specific safe `details`. Diagnostic
`message` text helps a person but is not a machine identifier.

## Capacity and deadlines

The listener bounds simultaneous database work. A request first waits for a slot up to the queue
deadline, then runs its database exchange up to the request deadline. Queue saturation, closed
capacity, unavailable authority and elapsed deadlines are named responses; they do not become
unbounded tasks detached from the request.

SIGINT or SIGTERM stops accepting new work and allows in-flight exchanges to drain up to the
configured shutdown timeout.
