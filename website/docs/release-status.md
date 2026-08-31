---
sidebar_position: 6
title: Release status
---

# Developer preview status

The preview is intended for contract review, local integration, and isolated evaluation. It is not
yet a production authentication boundary or a hosted multi-tenant control plane.

Available now:

- EP-owned strict command/query HTTP contracts with derived OpenAPI;
- trusted-context orchestration seams for humans and delegated agents;
- fresh PostgreSQL transactions and indexed queries through Entity Runtime;
- definition digest pinning, typed refusals, concurrency/deadline limits, probes, and graceful
  SIGINT/SIGTERM handling; and
- source builds, container packaging, public documentation, and issue-based feedback.

Next major work includes real SSO/token verification, provisioning and bundle activation control
planes, authorized repository projections, MCP exposure, and production observability/backup proof.
