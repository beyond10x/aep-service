---
format: aep.planning-md/1
id: story:trusted-command-context
kind: story
status: implemented
title: Construct command context from verified identity
summary: Derive actor, executor and trusted request metadata at the server boundary.
relations:
- decomposes: epic:application-service-boundary
- serves: vision:O1
revision: 5
---
## Context

The current in-process command envelope lets its constructor populate actor, executor and time,
which is not a trust boundary for network requests.

## Acceptance

For human and delegated-agent requests, the service constructs actor, executor, request identity and recorded time only from verified server context and ignores or refuses any payload attempting to assert them.

## Implementation record — 2026-08-31

`aep-service-auth::CredentialVerifier` is the only port that receives an authorization value. It returns a credential-free `VerifiedPrincipal` with authority, a distinct executor when delegated, effective realm/workspace grants, roles and delegation identity. The HTTP adapter verifies and authorizes the path scope before decoding request bodies or binding a semantic service.

`aep-service-app::TrustedRequestContext` combines that verified principal with only server-owned request id and receive time. Its command-context constructor takes caller-controlled logical correlation fields but always sources actor, executor, request id and issued time from trusted context. Human executor equality is canonicalized to no distinct executor; delegated attribution remains `actor = owner`, `executor = agent`.
