---
format: aep.planning-md/1
id: epic:definition-bundle-lifecycle
kind: epic
status: draft
title: Definition bundles evolve without rewriting history
summary: Register, preflight, activate and migrate immutable EP bundles while retaining verifiable historical definitions.
relations:
- decomposes: initiative:central-aep-authority
- serves: vision:O2
revision: 1
---
# Epic: Definition bundles evolve without rewriting history

## Outcome

The service can register, preflight, activate and migrate immutable EP definition bundles while old
decisions remain replayable against the exact definitions that produced them.

## Scope

Bundle identity and digest, active-version policy, compatibility facts, preflight over stored
instances, migrations, retained definitions and forward-only recovery.

## Done When

An instance created under one definition version is deliberately migrated to the next with a
recorded event, while unchanged historical replay still uses the old bytes.

