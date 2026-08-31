---
format: aep.planning-md/1
id: story:instance-definition-migrations
kind: story
status: draft
title: Migrate instances between definition versions
summary: Carry fields and states forward with recorded, resumable migrations without changing prior decisions.
relations:
- decomposes: epic:definition-bundle-lifecycle
- serves: vision:O2
revision: 1
---
## Context

Existing instances stay governed by their original definition until fields, states and operation
semantics are deliberately carried forward.

## Acceptance

A resumable migration maps one instance version to the next, increments its revision, records the definition transition and transformed values, refuses stranded instances by name, and never changes prior decisions.

