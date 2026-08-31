---
format: aep.planning-md/1
id: story:preview-release-and-oci
kind: story
status: draft
title: Ship binaries and a local-only preview image
summary: Publish checksummed binaries, an attested non-root OCI image and a loopback-only Compose demonstration.
relations:
- decomposes: epic:public-developer-preview
- serves: vision:O2
- serves: vision:O6
revision: 1
---
## Context

A public source tree without a reproducible runnable path makes the architectural preview unnecessarily expensive to evaluate, while the development verifier must not become an accidental remote deployment mode.

## Acceptance

A tag publishes cross-platform checksummed binaries and a multi-architecture non-root GHCR image with provenance/SBOM; Compose supplies PostgreSQL and a pinned preview bundle, publishes only host loopback, requires caller-supplied secrets and uses the explicit insecure-development listener override.
