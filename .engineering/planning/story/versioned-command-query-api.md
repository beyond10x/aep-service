---
format: aep.planning-md/1
id: story:versioned-command-query-api
kind: story
status: proposed
title: Serve the versioned AEP command/query API
summary: Realize the EP-owned service wire with explicit compatibility and structured results.
relations:
- decomposes: epic:application-service-boundary
- serves: vision:O2
revision: 2
---
## Context

EP owns command and query meaning; the service needs an explicit transport realization that can
remain compatible across independent client and server releases.

## Acceptance

The service accepts the published AEP service wire version, negotiates supported versions, and returns typed results and errors that the EP conformance client decodes without internal Rust types crossing the boundary.

