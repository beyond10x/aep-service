---
format: aep.planning-md/1
id: story:pinned-startup-definition-bundle
kind: story
status: implemented
title: Require a pinned startup definition bundle
summary: Become ready only with the validated EP definitions named by configuration.
relations:
- decomposes: epic:operable-service
- serves: vision:O2
revision: 5
---
## Context

A runnable service must say which immutable definitions it evaluates before dynamic bundle activation exists.

## Acceptance

Startup loads one local EP definition bundle, verifies its configured SHA-256 digest, supplies that registry to command and type-description paths, and keeps readiness false for a missing, invalid or mismatched bundle.

## Implementation

The runtime loads EP's pinned definition bundle before binding the listener, verifies it with EP's bundle loader, and passes the resulting lifecycle registry into both command sessions and type-description queries. Startup therefore fails before readiness for a missing, invalid, or digest-mismatched bundle.
