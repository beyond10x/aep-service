---
format: aep.planning-md/1
id: story:http-service-runtime
kind: story
status: draft
title: Compose the runnable HTTP service
summary: Start and stop the authenticated application and PostgreSQL provider with truthful readiness.
relations:
- decomposes: epic:operable-service
- serves: vision:O1
revision: 1
---
## Context

The repository currently has library boundaries but no process that composes them into a service.

## Acceptance

One Rust binary starts the HTTP adapter, authentication verifier, application service and PostgreSQL provider from explicit configuration, reports readiness only after dependencies and the active bundle are usable, and shuts down without abandoning accepted requests.

