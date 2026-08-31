---
format: aep.planning-md/1
id: story:service-conformance-vectors
kind: story
status: proposed
title: Publish service/client conformance vectors
summary: Hold the independently released EP client and service to identical wire outcomes.
relations:
- decomposes: epic:application-service-boundary
- serves: vision:O2
revision: 2
---
## Context

The independently released EP client and service need executable evidence that their wire and
failure taxonomies still agree.

## Acceptance

A versioned corpus covers accepted, replayed, refused, revision-conflicting, malformed and unavailable commands plus authorized and unauthorized queries, and both the service and EP client verify identical bytes.

