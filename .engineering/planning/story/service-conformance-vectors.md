---
format: aep.planning-md/1
id: story:service-conformance-vectors
kind: story
status: implemented
title: Publish service/client conformance vectors
summary: Hold the independently released EP client and service to identical wire outcomes.
relations:
- decomposes: epic:application-service-boundary
- serves: vision:O2
revision: 5
---
## Context

The independently released EP client and service need executable evidence that their wire and
failure taxonomies still agree.

## Acceptance

A versioned corpus covers accepted, replayed, refused, revision-conflicting, malformed and unavailable commands plus authorized and unauthorized queries, and both the service and EP client verify identical bytes.

## Implementation record — 2026-08-31

The service test iterates `aep_client::conformance::CASES` directly from pinned EP 0.35.0. For every constructed exchange it asserts status, selected media type, supported-version header and body bytes, then independently asserts whether command/query dispatch and trusted service binding occurred. The corpus proves human and delegated attribution, accepted and replayed commands, semantic and revision refusals, malformed/non-dispatched input, service unavailability, workspace admission, query admission and failed version negotiation without carrying real credentials or adopter data.
