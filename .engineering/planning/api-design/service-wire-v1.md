---
format: aep.planning-md/1
id: api-design:service-wire-v1
kind: api-design
status: draft
title: AEP service wire v1
summary: Map the EP-owned wire into verified service context without exposing a raw store.
relations:
- designs: story:versioned-command-query-api
- informed_by: story:trusted-command-context
- informed_by: story:service-conformance-vectors
revision: 2
---
## Purpose

Define how `aep-service` realizes the EP-owned service wire at its trust boundary without making
HTTP payloads, authentication claims or PostgreSQL records a second semantic command model.

The normative wire proposal lives in
`engineering-protocols/docs/design/aep-service-wire-v0.1.md`. This artifact records the service
side of that coordinated boundary. It must track a pinned EP contract version; it does not copy the
wire schema.

## Boundary

```text
untrusted HTTP bytes
  -> wire version and shape validation
  -> token and delegation verification
  -> realm/workspace authorization
  -> server-owned RequestContext
  -> EP CommandEnvelope or QueryService call
  -> typed wire response
```

The public request carries intent: realm, workspace, command identity, idempotency identity,
expected revision, correlation and causation, command type and payload. It does not carry actor,
executor, roles, request identity or the recorded time.

The adapter constructs those trusted values from the verified access token, an optional verified
delegation proof, the receiving request and the server clock. Fields with those names in a command
payload have domain meaning only; they never override request context.

## Authority and executor

A human request has the authenticated human as both authority and effective executor. A delegated
agent request has the owner named by the verified delegation as authority and the authenticated
agent as executor. Effective grants are the intersection of:

1. the authority's current realm and workspace grants;
2. the signed delegation's scope and expiry; and
3. restrictions attached to the executor identity.

Failure at any layer is an authentication or authorization refusal. It is not converted into a
domain validation error and no semantic command is dispatched.

## Versioning and compatibility

The HTTP adapter supports explicit EP wire versions. Unknown versions and unknown document fields
are refused; independent client and server releases do not silently reinterpret bytes. A wire
change lands as a new EP-owned version with pinned constructed conformance vectors and a coordinated
Atlas decision before this service serves it.

The server may serve two versions during a migration. Internally, each accepted version is adapted
to the same semantic `CommandService` and `QueryService` contracts. There is no discovery endpoint:
failed negotiation advertises the served set through `AEP-Supported-Versions`, and successful
responses identify the selected version through their media type.

Every nullable version-1 request member is mandatory and is explicitly `null` when absent. A missing
member is malformed rather than a second spelling for the same request.

## Error boundary

Transport and application failures remain distinguishable:

- malformed or unsupported wire documents never reach the application service;
- missing or invalid credentials are unauthenticated;
- insufficient workspace grants are unauthorized;
- after workspace admission, absent and entity-level-denied targets are both not found;
- semantic refusals preserve the stable EP command/query error code;
- optimistic concurrency remains a typed conflict;
- unavailable means a retry of unchanged intent may succeed later.

Error bodies contain stable codes and safe structured details. They do not expose token contents,
database errors, hidden entity data or whether an entity exists outside the caller's authorized
scope.

## Idempotency

Realm, workspace and authority scope an idempotency key. A different executor acting under the same
still-valid authority may retrieve the original result; the replay attempt remains separately
attributable. Reusing a key for different intent is a conflict.

## Conformance seam

The HTTP crate is tested with the EP-owned constructed corpus. Its application and authentication
dependencies are replaceable test ports, so vectors can prove both what crossed the network and
what was handed to the semantic service. The minimum corpus covers:

- an accepted command;
- an idempotent replay;
- a revision conflict;
- a semantic refusal;
- malformed and unsupported wire versions;
- missing authentication and insufficient authority;
- a human principal and a delegated agent principal; and
- authorized and unauthorized queries.

Every vector pins request bytes, verified principal input, expected semantic call or non-dispatch,
response status and response bytes.

## Review Record — 2026-08-31

The operator decided explicit nullable members, authority-scoped idempotency, entity privacy after
workspace admission and discovery through negotiation. The EP client-facade choice remains open, so
this design is still a review artifact rather than an implementation claim.

## Deliberately not decided here

This artifact does not choose an HTTP framework, async runtime, token encoding, identity provider,
database schema, deployment platform or Markdown projection policy. Those decisions belong to
their own stories after the wire and trust boundary have been reviewed.

## Acceptance

The EP proposal and this service mapping jointly account for every client-controlled field, every
server-derived field, version negotiation, non-dispatch refusals and the constructed vectors that
will prove the independently released client and service agree.
