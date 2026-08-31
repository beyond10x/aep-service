---
format: aep.planning-md/1
id: epic:application-service-boundary
kind: epic
status: proposed
title: The authenticated AEP application boundary
summary: Implement the EP-owned command/query wire without exposing a raw Entity Runtime store.
relations:
- decomposes: initiative:central-aep-authority
- serves: vision:O1
- serves: vision:O2
revision: 2
---
# Epic: The authenticated AEP application boundary

## Outcome

The service accepts the EP-owned versioned command/query protocol, turns verified transport
identity into trusted application context and returns stable structured results and refusals.

## Scope

Wire negotiation, server-side adapters, compatibility tests and conformance vectors. The official
client remains in engineering-protocols.

## Done When

An EP loopback client and the service agree byte-for-byte on accepted, refused, conflicting and
unavailable command/query cases without exposing a raw store operation.

