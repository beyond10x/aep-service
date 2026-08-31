---
format: aep.planning-md/1
id: epic:transactional-authority
kind: epic
status: active
title: PostgreSQL is one transactional authority
summary: Evaluate from current durable state and atomically persist every effect of an accepted or refused command.
relations:
- decomposes: initiative:central-aep-authority
- serves: vision:O2
revision: 3
---
# Epic: PostgreSQL is one transactional authority

## Outcome

Every command decides against current durable state and makes its state, relation, event, audit and
idempotency effects visible together, even with several service processes writing concurrently.

## Scope

Transactional unit of work, optimistic concurrency, refusal ledger, AEP query indexes and recovery
from process or database failure.

## Done When

Concurrency and injected-failure tests demonstrate no partial command, silent overwrite or accepted
command without its attributable record.

