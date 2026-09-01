---
title: Security and trust model
description: Enforced preview boundaries, unfinished production identity and safe vulnerability reporting.
---

# Security claims stop where the evidence stops

Hosted human authentication resolves every bearer through Identity at an exact audience and admits
only the configured tenant. Delegated-agent proof verification remains the principal unfinished
trust boundary. Loopback development can use one exact bearer token from an environment variable;
it refuses a non-loopback bind unless `--allow-insecure-dev-listener` is explicit.

## Enforced today

- Request documents cannot assert actor, executor, roles, request id or received time.
- Hosted sessions are resolved through Identity and narrowed to one configured tenant.
- Realm/workspace admission occurs before semantic dispatch where the route provides enough scope.
- Authority and executor remain separate facts in trusted context and audit.
- Commands use fresh transactional state and publish no partial candidate on refusal.
- Definition trees are validated and pinned by immutable digest before readiness.
- PostgreSQL/provider operations and credentials are not public APIs.
- Request bodies, database concurrency, queue wait, exchange duration and shutdown are bounded.
- Responses are `no-store` and carry `X-Content-Type-Options: nosniff`.
- The container runs without root privileges.

## Not yet a production claim

- Delegated-agent owner signatures and scope intersection are ports, not deployed verification.
- Multi-realm provisioning and policy administration are not exposed.
- PostgreSQL TLS, pooling, backup/restore and retention policy are not production-certified.
- Metrics, traces, SLOs, load limits and fault-injection evidence are not yet published.

## Credential handling

Never place a token or database URL in a repository, image layer, issue, generated projection or
command transcript intended for sharing. Prefer a runtime secret source over a checked environment
file. Use only disposable data with the development verifier.

## Reporting vulnerabilities

Use GitHub private vulnerability reporting from the repository’s **Security** tab. Do not include
credentials, company records, customer data or production configuration in a public issue.

See the repository [SECURITY policy](https://github.com/beyond10x/aep-service/blob/main/SECURITY.md)
for supported versions and the documented website dependency advisory exception.
