---
format: aep.planning-md/1
id: story:delegated-agent-authority
kind: story
status: draft
title: Delegated agents can only narrow authority
summary: Intersect owner grants, signed delegation scopes and executor restrictions.
relations:
- decomposes: epic:identity-and-access
- serves: vision:O1
revision: 1
---
## Context

An owner-authorized agent must be attributable as executor while remaining unable to exercise a
role or scope the owner did not delegate.

## Acceptance

The effective permission of a delegated request is the intersection of the authority's current grants, signed delegation scopes and executor restrictions, and tests show that widening any token claim cannot exceed that intersection.

