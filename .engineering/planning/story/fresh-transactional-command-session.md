---
format: aep.planning-md/1
id: story:fresh-transactional-command-session
kind: story
status: implemented
title: Use one fresh PostgreSQL command session
summary: Decide and persist from current rows without request-time realm hydration.
relations:
- decomposes: epic:transactional-authority
- serves: vision:O2
revision: 5
---
## Context

The implemented atomic-batch story proves all-or-nothing persistence, but its bridge opens a provider and hydrates all AEP records for every request. Atlas ADR 0008 excludes that bridge from the central write path.

## Acceptance

Each command opens one PostgreSQL transaction, locks or loads only the records its EP decision needs, records all accepted or refused effects and replay memory in that transaction, and never hydrates the complete realm even when two service processes share it.

## Implementation

Every bound command service opens a fresh EP PostgreSQL session carrying the pinned lifecycle registry. ER's transactional provider loads and locks only command-relevant rows, persists decisions, events, audit records, relations, and replay memory atomically, and no longer hydrates a realm into process memory.
