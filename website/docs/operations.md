---
sidebar_position: 4
title: Operating the preview
---

# Operating the preview

The service refuses to become reachable until its immutable definition bundle verifies and its
PostgreSQL schema is prepared. `/livez` means the listener is running; `/readyz` means startup
dependencies were prepared.

Runtime limits are explicit CLI arguments:

- database concurrency bounds simultaneous blocking sessions;
- queue timeout bounds how long a request waits for a slot;
- request timeout bounds one database-backed exchange;
- shutdown timeout bounds graceful drain after SIGINT or SIGTERM; and
- request bodies are limited to 1 MiB.

Overload and deadlines use the same typed AEP problem document as semantic unavailability. Every
response is `no-store` and carries `X-Content-Type-Options: nosniff`.

Use `aep-service probe` for container health checks. The provided image runs as an unprivileged user
and expects database URL, development token, definition path, and digest at deployment time.
