---
title: HTTP contract
description: Routing, media negotiation, authentication and response conventions for the AEP Service API.
---

# A strict HTTP realization of the EP wire

All semantic routes sit below:

```text
/aep/v1/realms/{realm}/workspaces/{workspace}
```

The path scopes the attempted operation. The verified principal must admit that realm and workspace
before the service decodes a semantic body.

## Authentication

Semantic routes require an HTTP bearer credential:

```http
Authorization: Bearer <credential>
```

The developer preview compares the complete header to one configured local token. Production token
verification is not implemented. Probes and `/openapi.json` do not require semantic authentication.

## Media types

Version 1 operations use:

```http
Accept: application/vnd.aep.service+json;version=1
Content-Type: application/vnd.aep.service+json;version=1
```

The bounded history query uses version 2 for both headers. GET requests have no `Content-Type`.
Unsupported negotiation is refused rather than silently reinterpreted as another version.

## Server-owned response metadata

Answered documents include a server-generated request id. HTTP responses are `no-store` and carry
`X-Content-Type-Options: nosniff`. Request ids are UUIDv7 values created at the process boundary;
received timestamps come from the server clock.

## Public non-semantic routes

| Route | Meaning |
|---|---|
| `/livez` | The process listener is alive |
| `/readyz` | Definitions and PostgreSQL were prepared before the listener became available |
| `/openapi.json` | Deterministic OpenAPI 3.1 generated from EP routes and schemas |

## Limits

Request bodies are limited to 1 MiB. Only GET and POST are accepted by the semantic fallback.
Database concurrency, queue wait, exchange duration and graceful drain are bounded by process
configuration. Capacity and deadline failures use the typed problem document where the selected
wire version permits one.
