---
format: aep.planning-md/1
id: story:versioned-command-query-api
kind: story
status: implemented
title: Serve the versioned AEP command/query API
summary: Realize the EP-owned service wire with explicit compatibility and structured results.
relations:
- decomposes: epic:application-service-boundary
- serves: vision:O2
revision: 5
---
## Context

EP owns command and query meaning; the service needs an explicit transport realization that can
remain compatible across independent client and server releases.

## Acceptance

The service accepts the published AEP service wire version, negotiates supported versions, and returns typed results and errors that the EP conformance client decodes without internal Rust types crossing the boundary.

## Implementation record — 2026-08-31

`aep-service-http` now realizes every EP v1 route over the runtime-neutral `aep_client::wire::Request` and `Response` exchange. It strictly negotiates the v1 media type, percent-decodes opaque path segments, rejects malformed closed documents, maps semantic failures through EP's stable problem taxonomy and never exposes a generic store operation. `ServiceProvider` binds the already-authorized trusted context to a normal EP `CommandService<Command = Command> + QueryService<AuditRecord = AuditRecord>` handle, preserving the semantic contract while making realm/workspace and query authorization impossible to omit at binding time.

The workspace declares EP 0.35.0 once; every consuming crate inherits that exact tag, and `Cargo.lock` pins its release commit.
