---
title: Core concepts
description: Tenant, realm, workspace, authority, executor, definitions, entities, revisions and projections in AEP Service.
---

# The vocabulary of one governed authority

## Tenant, realm and workspace

A **tenant** is the administrative owner. A **realm** is an isolated policy, definition and data
boundary inside a tenant. A tenant may own multiple realms; common ownership alone grants no
cross-realm access. A **workspace** scopes a repository or another collaboration surface inside one
realm.

The developer-preview process serves one configured realm and workspace. The data model preserves
the wider multi-tenant hierarchy; provisioning and routing many realms is later control-plane work.

```text
tenant
├─ realm: product-engineering
│  ├─ workspace: api
│  └─ workspace: web
└─ realm: security
   └─ workspace: controls
```

## Authority and executor

**Authority** is the person or organization on whose behalf an action occurs. **Executor** is the
agent or automation that performed it. A human acting directly has authority and no separate
executor. An agent acting for that human records both.

Future delegated-agent tokens must be signed or minted under the owner’s authority and may only
narrow current grants. Request JSON cannot assert either trusted identity.

## Definitions and bundles

AEP declares entity types, fields, lifecycles, commands, relations and rules as
data. One validated definition tree becomes an immutable **bundle** identified by a digest. The
service pins the expected digest at startup so a deployment never silently changes meaning because
a mounted directory changed.

## Entity identity and locator

An entity receives an immutable machine identity. It may also carry a logical `ep://` locator—the
stable address an organization knows, such as:

```text
ep://demo/plan/story/first-governed-story
```

Clients resolve a locator when they know the organizational address and use the entity identity
when guarding revisions or reading history.

## Revision and consistency

Every accepted state change advances an entity revision. A command that depends on current state
may carry `expected_revision`; a mismatch is a named conflict, not a last-write-wins overwrite.

Successful commands return a consistency token. A later query can demand a view at least as fresh
as that token. `current` asks the authority for its current committed view.

## Activity and evidence

The record is more than current state:

- **history** holds immutable entity revisions;
- **events** say what an accepted command emitted;
- **relations** connect entities with typed edges;
- **audit** attributes accepted and refused activity; and
- **idempotency memory** makes a retry distinguishable from new intent.

## Markdown projections

Markdown is a deterministic human review surface. It may be rendered on demand or committed and
checked for drift, but an edit becomes a semantic command before it changes the authority. This
keeps Git review useful without creating two sources of truth.
