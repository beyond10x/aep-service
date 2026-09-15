---
format: aep.planning-md/1
id: story:preview-release-and-oci
kind: story
status: draft
title: Ship binaries and a local-only preview image
summary: Take every preview credential from the caller; the binaries, the attested multiarch image and the loopback-only Compose demonstration already ship.
relations:
- decomposes: epic:public-developer-preview
- serves: vision:O2
- serves: vision:O6
revision: 4
---
## Context

Binaries, the image and the loopback demonstration shipped with the preview releases:
`.github/workflows/release.yml:69-75` publishes a multi-architecture GHCR image with
`provenance: mode=max` and `sbom: true`, GitHub carries releases 0.1.3 through 0.1.8, and
`compose.preview.yaml` publishes only `127.0.0.1:8080:8080` (`:44-45`) behind the explicit
`--allow-insecure-dev-listener` override (`:28`).

One acceptance line is still unmet, and it is what keeps this story open: the demonstration does not
require caller-supplied secrets. `compose.preview.yaml:7` hardcodes `POSTGRES_PASSWORD: postgres`
and `:22` repeats it inside `AEP_DATABASE_URL`, while the bearer token (`:23`) and the definition
digest (`:38`) already use the `${VAR:?...}` form this line needs.

## Acceptance

The preview demonstration takes every credential from the caller: `POSTGRES_PASSWORD` and the
password inside `AEP_DATABASE_URL` are `${VAR:?...}` variables with no default, and a
`docker compose -f compose.preview.yaml up` with none of them set refuses by naming the missing
variable rather than starting on a known password.
