---
format: aep.planning-md/1
id: story:preview-runtime-hardening
kind: story
status: implemented
title: Bound and identify every preview HTTP exchange
summary: Add finite queue/request/shutdown behaviour, durable request identities and an explicit container-only development override.
relations:
- decomposes: epic:public-developer-preview
- serves: vision:O1
revision: 5
---
## Context

The current body and database concurrency bounds still permit indefinite queueing, have process-restart request-id collision risk, and cannot be reached from a container without removing the development listener guard.

## Acceptance

Queue, request and shutdown waits are finite and configurable; valid overloads are typed unavailable responses; request ids are UUIDv7; SIGINT and SIGTERM drain; non-loopback development serving remains refused unless an explicit noisy override is present; data responses are non-cacheable and CORS stays closed.

## Implementation record

Read at `ed4ef5b`, one clause at a time:

- Finite and configurable waits: `crates/aep-service/src/main.rs:134-142` declares
  `--queue-timeout-ms`, `--request-timeout-ms` and `--shutdown-timeout-ms`, and `:387-391` refuses
  zero for any of the three.
- Typed unavailable responses: `crates/aep-service/src/main.rs:524-537` answers a queue-timeout and
  a request-timeout with `unavailable_response`.
- UUIDv7 request ids: `crates/aep-service/src/main.rs:609`, asserted by
  `transport_request_identities_are_uuid_version_seven_values` (`:736`).
- SIGINT and SIGTERM drain: `crates/aep-service/src/main.rs:623-632` selects on both, and `:436-444`
  bounds the drain by `--shutdown-timeout-ms`.
- Non-loopback development serving refused without the named override: `validate_listener`, asserted
  by `non_loopback_development_authentication_requires_the_named_override` (`:743`).
- Non-cacheable data responses and closed CORS: `crates/aep-service/src/main.rs:655-658` sets
  `Cache-Control: no-store` and `X-Content-Type-Options: nosniff`, and
  `crates/aep-service-http/src/lib.rs:632-634` asserts both headers and the absence of
  `Access-Control-Allow-Origin`.

CHANGELOG `0.1.0` records the same set; release 0.1.8 ships it and Gate run 34914495607 reports
`gate` success at `83d447d`.
