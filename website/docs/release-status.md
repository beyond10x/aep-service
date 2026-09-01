---
title: Preview status
description: What AEP Service proves today, what remains unfinished and which artifacts are published.
---

# Developer preview status

The preview is intended for architecture and contract review, local integration, and isolated
evaluation. It is not a hosted multi-tenant control plane or production authentication boundary.

## Available now

- AEP-owned strict command/query HTTP contracts with deterministic OpenAPI;
- trusted-context orchestration seams for humans and delegated agents;
- fresh PostgreSQL command transactions and indexed bounded queries through Entity Runtime;
- complete revision, relation, audit and durable idempotency records;
- definition digest pinning, typed refusals, concurrency/deadline limits and graceful shutdown;
- loopback-bound OCI evaluation with liveness/readiness probes;
- checksummed Linux, macOS and Windows binaries; and
- public documentation, source, issues and private vulnerability reporting.

## Next boundaries

The roadmap continues with:

1. production SSO and delegated-agent token verification;
2. tenant/realm provisioning and definition-bundle activation;
3. authorized Markdown and company-internal projections;
4. remote `protocol` transport and MCP exposure;
5. PostgreSQL pooling/TLS, backup/restore and migrations; and
6. production observability, SLOs, load and fault evidence.

## Published artifacts

| Artifact | Location |
|---|---|
| Source | [GitHub repository](https://github.com/beyond10x/aep-service) |
| Release archives | [GitHub Releases](https://github.com/beyond10x/aep-service/releases) |
| Container | `ghcr.io/beyond10x/aep-service` |
| OpenAPI | [Generated JSON](pathname:///openapi.json) |
| API explorer | [Static reference](/aep-service/api) |

The versioned image is the reproducible evaluation target. `preview` follows the newest preview
release and may move.
