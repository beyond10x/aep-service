---
format: aep.planning-md/1
id: story:workspace-scoped-identities
kind: story
status: draft
title: Scope identities by realm and workspace
summary: Make repository-local short ids and cross-repository canonical references coexist without collision.
relations:
- decomposes: epic:identity-and-access
- serves: vision:O1
- serves: vision:O2
revision: 2
---
## Context

Repository-local ids such as `story:x` need a stable containing scope before relations can cross
repositories without collision. A tenant is the control-plane owner of one or more globally unique
realms; realm is the immutable AEP authority and storage boundary, workspace is the repository or
project scope inside it, and common tenant ownership creates no cross-realm access.

## Acceptance

Every entity and relation belongs to an immutable realm and workspace identity, short ids resolve only within the caller's current workspace, and canonical references remain unambiguous across repositories.

