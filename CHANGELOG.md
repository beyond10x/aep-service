# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

- Added a runnable Axum/Tokio service process with bounded request bodies, graceful shutdown,
  EP-owned pinned definition loading, bounded database concurrency, and a loopback-only development
  bearer verifier.

- Added a realm/workspace-scoped PostgreSQL service provider that evaluates every command through
  EP's fresh dependency-scoped transaction session and answers entities, relations, histories and
  audit queries through Entity Runtime's indexed provider interface instead of realm hydration.
- Added bounded history wire v2 alongside complete wire-v1 history compatibility, including strict
  media negotiation and opaque cursors bound to the complete indexed query.
- Real-PostgreSQL tests prove late-failure rollback, one-winner optimistic concurrency, paginated
  indexed reads and read-after-write visibility across fresh service handles.
- Added verified human/delegated-agent principals, server-owned command attribution and an
  authorized realm/workspace service-binding seam before semantic dispatch or materialization.
- Pinned EP 0.36.2 and Entity Runtime 0.17.3 and verified the v1 service byte-for-byte against EP's
  constructed corpus plus the coordinated v2 history exchange.
- Established the repository, architectural boundaries and governed delivery plan.
