---
format: aep.planning-md/1
id: story:paginated-history-wire-v2
kind: story
status: implemented
title: Serve paginated history wire v2
summary: Implement EP's bounded history route alongside complete version-1 compatibility.
relations:
- decomposes: epic:application-service-boundary
- serves: vision:O2
revision: 5
---
## Context

The service must implement the EP-owned migration from Atlas ADR 0009 without taking ownership of wire meaning.

## Acceptance

The service negotiates both EP wire versions, serves bounded cursor-based version-2 history, and returns byte-compatible complete version-1 history by draining the indexed page source rather than truncating it.

## Implementation

The HTTP adapter now negotiates EP wire versions 1 and 2. Version 2 serves bounded, cursor-based history pages through the indexed authority query path, while the unchanged EP version-1 corpus remains byte-compatible. The conformance suite pins both behaviours.
