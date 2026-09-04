# Changelog

All notable changes to this project will be documented in this file.

## [0.1.8] — 2026-09-04

- Consume AEP 0.53.0 as one coordinated dependency generation, including the current grouped
  `aep plan artifact` command surface and relation-removal correctness fixes, while preserving the
  service's existing HTTP contract.
- Install the pinned AEP CLI into a run-local CI root so a cached global binary from an older
  release cannot block or contaminate the service gate.
- Package the definition tree from the exact Cargo-locked AEP revision in the standalone image so
  hosted deployments can pin and load `/definitions` without a second source checkout.

## [0.1.7] — 2026-09-04

- Repin the service to AEP 0.51.0. That release moved AEP's crates under area directories
  (`crates/{govern,plan,edge}/…`) and renamed three crates the service does not pin, so all six
  workspace dependencies move as one generation and no API adaptation was needed: the released
  sources of `aep-client`, `aep-contract`, `aep-backend-postgres` are unchanged since 0.45.0, and
  `aep-domain`, `aep-backend-entity` and `aep-project` changed only documentation and diagnostic
  text. The generated OpenAPI document and the AEP-owned HTTP conformance corpus still derive from
  the released contract and are byte-identical to the ones 0.45.0 produced.
- Move the Entity Runtime provider pin to 0.17.6, the generation AEP 0.51.0 selects. The service
  and the AEP backends exchange `entity-core` values, so a second Entity Runtime tag in the graph
  would be two incompatible sets of the same types rather than a duplicate build. 0.17.6 changes
  File Store behavior only; the PostgreSQL provider contract this service uses is unchanged.
- Refuse a partial re-pin at the gate: a workspace test now reads `Cargo.toml` and `Cargo.lock` and
  requires one exact AEP tag, one exact Entity Runtime tag, a lockfile resolved at both, and no
  crate-local git pin, so a generation moved in one declaration and not the rest fails a test
  instead of reaching a build.

## [0.1.6] — 2026-09-03

- Upgrade the service to AEP 0.45.0 while retaining the current Entity Runtime 0.17.5 provider
  contract.

## [0.1.5] — 2026-09-02

- Added hosted Identity session verification with exact audience and tenant admission so central
  AEP queries retain server-derived human attribution without a shared development bearer.
- Committed the deterministic OpenAPI 3.1 projection as a passive public documentation source and
  added a gate check that refuses drift from the service-owned route and schema definitions.
- Reduced tagged delivery to the supported Linux diagnostic archive and multiarch OCI image, and
  moved release and manifest finalization into their producing jobs so account limits cannot strand
  already-built artifacts in new post-build runners.

## [0.1.4] — 2026-09-01

- Accelerated tagged delivery with cached gate tools and native release builds, replaced emulated
  multiarch image compilation with parallel native architecture builds, and retired the unsupported
  Windows archive while retaining Linux, macOS and multiarch OCI artifacts.

## [0.1.3] — 2026-09-01

- Repin the service to AEP 0.40.3 and Entity Runtime 0.17.5, and use the canonical `aep` command
  for the governed planning gate while retaining compatibility with `protocol` through AEP.

## [0.1.2] — 2026-09-01

- Pinned the service to sanitized AEP 0.40.1 and Entity Runtime 0.17.4 releases at their canonical
  repositories, and removed retired ESS-owned conformance fields while preserving the HTTP contract.

## [0.1.1] — 2026-08-31

- Rebuilt the public site around the service's authority, evidence and refusal model, with a
  task-oriented guide, architecture and reliability material, and a responsive visual system.
- Replaced the runtime-fetched API page with a static, searchable reference rendered from the
  generated contract, including curl examples, response examples and the complete schema catalog.
- Enriched the deterministic OpenAPI projection with operation guidance, parameter descriptions,
  typed examples, stable schema anchors and explicit problem responses.
- Added a published-image Docker Compose evaluation path that starts PostgreSQL and the unprivileged
  service image, pins the EP definition bundle, and demonstrates create, replay, read and history.
- Moved synchronous PostgreSQL authority preparation off the async runtime so the runnable service
  starts cleanly instead of panicking during provider initialization.

## [0.1.0] — 2026-08-31

- Published the developer-preview repository contract, human README, contributing/security policy,
  curated Docusaurus site, generated API reference, issue templates, and Apache-2.0 license.
- Added deterministic OpenAPI 3.1 generation from EP-owned route metadata and JSON Schema DTOs,
  served the same document at `/openapi.json`, and made route/projection agreement executable.
- Added UUIDv7 request identities, queue and exchange deadlines, typed overload responses,
  no-store/nosniff headers, SIGTERM handling with bounded graceful drain, and a built-in health
  probe. Non-loopback development authentication now requires an explicit warned override.
- Added `aep-service definitions digest` so deployments can validate an EP tree and derive the
  exact immutable bundle identity they pin at startup.
- Added dependency policy and scheduled security auditing, PostgreSQL-backed CI, Pages publishing,
  tagged Linux binary releases, an unprivileged OCI image, and a local Docker Compose preview.
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
- Pinned EP 0.38.1 and Entity Runtime 0.17.3 and verified the v1 service byte-for-byte against EP's
  constructed corpus plus the coordinated v2 history exchange.
- Established the repository, architectural boundaries and governed delivery plan.
