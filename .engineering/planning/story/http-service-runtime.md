---
format: aep.planning-md/1
id: story:http-service-runtime
kind: story
status: implemented
title: Compose the runnable HTTP service
summary: Start and stop the authenticated application and PostgreSQL provider with truthful readiness.
relations:
- decomposes: epic:operable-service
- serves: vision:O1
revision: 5
---
## Context

The repository currently has library boundaries but no process that composes them into a service.

## Acceptance

One Rust binary starts the HTTP adapter, authentication verifier, application service and PostgreSQL provider from explicit configuration, reports readiness only after dependencies and the active bundle are usable, and shuts down without abandoning accepted requests.

## Implementation

The new Rust service binary composes configuration, the pinned EP bundle, PostgreSQL preparation, authentication, application dispatch, and the HTTP listener. It becomes reachable only after startup dependencies succeed and uses graceful shutdown so accepted requests can complete.
