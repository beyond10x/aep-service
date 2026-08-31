---
format: aep.planning-md/1
id: story:trusted-command-context
kind: story
status: proposed
title: Construct command context from verified identity
summary: Derive actor, executor and trusted request metadata at the server boundary.
relations:
- decomposes: epic:application-service-boundary
- serves: vision:O1
revision: 2
---
## Context

The current in-process command envelope lets its constructor populate actor, executor and time,
which is not a trust boundary for network requests.

## Acceptance

For human and delegated-agent requests, the service constructs actor, executor, request identity and recorded time only from verified server context and ignores or refuses any payload attempting to assert them.

