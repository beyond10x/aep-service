---
title: Why AEP Service exists
description: One central, attributable authority for engineering entities without turning Markdown or PostgreSQL into competing write paths.
---

# Engineering decisions need an authority, not another copy

Repository-local Markdown made planning easier for agents. It is versioned, close to the code and
available to the same tools that implement the work. It also fragmented the organizational view:
cross-repository dependencies are hard to query, references drift, and approvals or refusals live
in whichever file happened to receive them.

AEP Service makes those engineering entities centrally addressable. Humans, agents and integrations
submit semantic Agentic Engineering Protocol commands and queries. The service supplies trusted
request context, decides access, opens the transactional authority and records the complete result.

```text
human / agent / integration
          │
          │ versioned AEP command or query
          ▼
     AEP Service
     ├─ trust boundary
     ├─ authorization
     ├─ transaction boundary
     └─ operational policy
          │
          ▼
Entity Runtime providers + PostgreSQL
```

## What it owns

AEP Service owns the deployable boundary:

- verified request context and workspace admission;
- one fresh PostgreSQL transaction for every command;
- bounded HTTP execution, typed overload and graceful shutdown;
- immutable definition-bundle verification at startup; and
- public deployment artifacts, OpenAPI and operational documentation.

[AEP](https://github.com/beyond10x/aep) owns the semantic
entity vocabulary, commands, queries, strict wire documents and official client.
[Entity Runtime](https://github.com/beyond10x/entity-runtime) owns deterministic entity execution
and generic storage/query providers. The service composes their released contracts; it does not
copy or hide them.

## What it deliberately is not

It is not a sprint board, customer issue queue, comment stream, notification marketplace or generic
workflow editor. It does not expose SQL or a raw Entity Runtime store. It does not make Markdown a
second persistence mechanism.

Markdown remains useful as a deterministic human review surface. A repository may render it on
demand or commit and check the projection for drift. Changes become semantic commands before they
affect the authority.

## Preview truth

The transactional, storage and wire boundaries are available for evaluation. Production identity
is not. The current verifier accepts one exact development bearer token and is safe only on host
loopback or behind an explicitly isolated preview boundary.

Start with the [OCI quickstart](./quickstart), then read the [architecture](./architecture) and
[security model](./security) before deciding what the preview proves.
