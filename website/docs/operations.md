---
title: Operate the preview
description: Startup, PostgreSQL, health probes, graceful shutdown and container boundaries for AEP Service.
---

# Operate the developer preview

## Startup sequence

The process refuses to listen until it has:

1. validated the listener and explicit development-authentication boundary;
2. read the database URL and non-empty development token;
3. loaded, validated and digest-verified the AEP definition tree;
4. prepared the PostgreSQL authority schema; and
5. constructed the scoped application service.

This order prevents a reachable process from advertising readiness against unknown definitions or
an unprepared database.

## PostgreSQL

Use one schema dedicated to the configured realm authority and a credential able to create or
prepare its required objects. The preview uses fresh blocking PostgreSQL sessions behind bounded
execution slots; production pooling and TLS policy remain future hardening work.

The public API never returns database credentials, SQL errors or provider rows. Database failures
are mapped to safe AEP problems.

## Probes

`/livez` means the listener task is running. `/readyz` means startup dependencies were prepared; the
preview does not yet run a continuous downstream health query on every request.

Use the binary rather than curl inside a container health check:

```console
aep-service probe --address 127.0.0.1:8080 --readiness
```

## Containers

The published runtime image:

- supports linux/amd64 and linux/arm64;
- runs as UID/GID `10001:10001`;
- contains CA certificates but no shell-based entrypoint logic;
- expects definitions mounted read-only; and
- receives the database URL and token at runtime.

Pin a version tag or immutable digest for reproducible deployment. The floating `preview` tag is for
evaluation only.

## Shutdown and overload

SIGINT or SIGTERM starts graceful drain. In-flight work gets the configured shutdown window; when
that expires, the server task is aborted and startup reports the timeout. Queue and request
deadlines are independent, allowing an operator to distinguish waiting for capacity from a slow
database exchange.

## Backup and recovery status

The developer preview does not yet publish a production backup, restore, point-in-time recovery or
disaster-recovery proof. Treat the Compose volume as disposable. Production adoption requires that
operational evidence, PostgreSQL TLS/pooling, retention policy and tested migrations.
