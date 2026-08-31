---
format: aep.planning-md/1
id: initiative:central-aep-authority
kind: initiative
status: proposed
title: Central AEP authority
summary: One authenticated command/query authority for AEP entities, history, definition bundles and authorized projections.
relations:
- serves: vision:O1
- serves: vision:O2
- serves: vision:O6
revision: 2
---
# Initiative: Central AEP authority

## Outcome

Humans, agents and integrations operate every planning entity through one authenticated AEP command
and query authority, while repositories retain deterministic Markdown projections for local review.

## Scope

The deployed service, trusted identity/delegation mapping, authorization, transactional PostgreSQL
persistence, definition-bundle operation, authorized queries and projection snapshots. EP continues
to own the semantic service contract and official client; ER continues to own the kernel and generic
providers.

## Success

One repository uses the service as its sole canonical write path, several processes safely share
the database, cross-repository relations are queryable, and the committed Markdown tree is
reproducible from an authorized service snapshot.

## Not This

A raw remote ER store, an identity provider, Jira's collaboration product surface, or a universal
company ontology.

