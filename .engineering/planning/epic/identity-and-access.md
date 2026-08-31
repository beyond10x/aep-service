---
format: aep.planning-md/1
id: epic:identity-and-access
kind: epic
status: draft
title: Identity, delegation and access
summary: Derive trusted actor/executor attribution and enforce workspace-scoped permissions on commands and reads.
relations:
- decomposes: initiative:central-aep-authority
- serves: vision:O1
revision: 1
---
# Epic: Identity, delegation and access

## Outcome

A human or delegated agent sees and changes exactly the work its verified authority permits, and
every decision distinguishes the authorizing actor from the executor that ran it.

## Scope

Trusted-principal mapping, workspaces, roles, delegated scopes, command authorization, query
filtering and security refusals. Token issuance and SSO UX remain outside the service.

## Done When

An agent cannot enlarge its owner's authority, restricted entity existence cannot be inferred
through queries or projections, and every authenticated denial names the deciding rule for auditors.

