---
format: aep.planning-md/1
id: story:jira-intake
kind: story
status: draft
title: Ingest Jira without sharing lifecycle authority
summary: Record sourced external reports and observations while internal work changes only through AEP commands.
relations:
- decomposes: epic:projections-and-adoption
- serves: vision:O2
revision: 1
---
## Context

Jira remains where customers and product teams report issues, but its workflow must not silently
become a second authority for internal engineering state.

## Acceptance

The integration idempotently records a Jira report and sourced observations, relates it to internal work, and follows a declared field-authority map without allowing Jira updates to bypass AEP commands.

