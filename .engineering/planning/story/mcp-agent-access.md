---
format: aep.planning-md/1
id: story:mcp-agent-access
kind: story
status: draft
title: Expose governed AEP tools to agents over MCP
summary: Project the authenticated command/query boundary as MCP tools and authorized resources.
relations:
- decomposes: epic:application-service-boundary
- serves: vision:O1
- serves: vision:O2
- depends_on: story:versioned-command-query-api
- depends_on: story:delegated-agent-authority
revision: 1
---
## Context

Agents need a tool-oriented surface for discovering and operating the central planning authority.
Making MCP a second command model, or exposing Entity Runtime storage primitives through it, would
bypass the service boundary and duplicate authorization, idempotency and refusal semantics.

The MCP adapter is therefore a projection over the same EP-owned command/query client contract used
by `protocol`. It authenticates the agent and its owner-authorized delegation, derives actor and
executor server-side, and dispatches every mutation through the application command service. Read
tools authorize before traversal or materialization.

## Scope

- discoverable MCP tools for the AEP command and query operations agents actually need;
- MCP resources for authorized read-only entity, graph, history and projection views where useful;
- the same realm/workspace scope, idempotency, revision conflicts and stable refusals as HTTP;
- delegated-agent attribution and scope enforcement; and
- conformance cases proving MCP and the official client reach the same semantic calls.

No tool accepts raw ER decisions, storage batches, audit records, actor/executor overrides or
database credentials. MCP transport sessions are not lifecycle state and do not become a second
activity log.

## Acceptance

An authenticated delegated agent can discover, query and execute its permitted AEP operations through MCP with the same semantic outcomes, attribution and refusals as the versioned service client, while an out-of-scope tool call is refused before any entity is read or changed.
