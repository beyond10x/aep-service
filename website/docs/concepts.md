---
sidebar_position: 3
title: Core concepts
---

# Core concepts

## Tenant, realm, and workspace

A tenant is the administrative owner. A realm is an isolated policy, definition, and data boundary
inside a tenant. A tenant may own multiple realms; ownership alone grants no cross-realm access. A
workspace scopes a repository or another collaboration surface inside one realm.

## Authority and executor

Authority is the person or organization on whose behalf an action occurs. Executor is the agent or
automation that performed it. A future delegated-agent token must be signed or minted under its
owner’s authority and can only narrow the owner’s current grants.

## Definitions and instances

Engineering Protocols publishes entity types, operations, states, and rules as immutable definition
bundles. Instances and their complete activity live in the central store. A definition change gets
a new digest and an explicit activation/migration record; bytes never change behind an old identity.

## Markdown projections

Markdown is a deterministic human review surface. It can be generated on demand or committed and
checked for drift, but edits become semantic commands before they affect the authority. This avoids
two sources of truth while retaining useful Git review.
