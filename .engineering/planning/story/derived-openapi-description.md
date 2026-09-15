---
format: aep.planning-md/1
id: story:derived-openapi-description
kind: story
status: implemented
title: Derive and serve the AEP OpenAPI description
summary: Generate OpenAPI from EP-owned wire types and expose the same deterministic bytes over HTTP and the website.
relations:
- decomposes: epic:public-developer-preview
- serves: vision:O2
- depends_on: epic:application-service-boundary
revision: 6
---
## Context

The deterministic OpenAPI projection covers every released EP route and strict DTO, but its
presentation metadata is too sparse for the public reference: operations are ungrouped, parameters
are unexplained and no valid requests or typed refusals are shown. The current website fetches the
document after hydration, allowing the API page to render as an empty shell.

## Delivery

Keep EP's route catalog and DTO schemas authoritative while enriching the service-owned OpenAPI
presentation with tags, descriptions, parameter documentation, response meanings and examples
constructed from typed Rust values and the released conformance corpus. The service, downloadable
asset and website must continue to consume the same deterministic bytes.

Replace the client-fetched card list with a native static explorer. It renders all operations and
schemas during the Docusaurus build, then hydrates only search, filtering and copy affordances. It
does not execute requests or retain bearer tokens.

## Acceptance

- Every EP route appears exactly once with its stable method, path and operation id; no route or DTO
  is restated in TypeScript.
- Every example deserializes as its declared strict DTO, and a command example's `command_type`
  names its semantic payload.
- The explorer groups commands, entities, relations, history, audit and types; supports stable
  operation anchors; and shows authentication, media type, parameters, request/response schemas,
  examples, statuses and problem documents.
- The generated API HTML contains every operation before client hydration, and failed JavaScript
  cannot erase the reference content.
- OpenAPI bytes are deterministic, `/openapi.json` and the Pages asset are identical, and the
  enriched document ships with AEP Service 0.1.1.

## Implementation record

Read at `ed4ef5b`, one clause at a time:

- Routes and ids: `website/static/openapi.json` is OpenAPI 3.1.0 at `info.version` `0.1.8` with 9
  operations, every `operationId` distinct, 65 component schemas and a `bearerAuth` security scheme.
- No route or DTO restated in TypeScript: `website/src/pages/api.tsx` is 20 lines — it imports
  `../../static/openapi.json` at build time and hands it to
  `@beyond10x/docs-system/renderers`'s `OpenApiReference`.
- Grouping and anchors: the document's `tags` are exactly `Commands`, `Entities`, `Relations`,
  `History`, `Audit`, `Types`.
- Rendered before hydration: the document is a build-time import rather than a fetch, so the static
  HTML carries the operations.
- Deterministic bytes, identical to the Pages asset: `Taskfile.yml:32-39` (`openapi-check`) regenerates
  the document and `cmp`s it against `website/static/openapi.json`, and `:18` puts that check inside
  `task check`. `crates/aep-service/src/main.rs:423` serves the same bytes at `/openapi.json`.

CHANGELOG `0.1.1` records the enriched projection as the acceptance requires; Gate run 34914495607's
`gate` job (`task check`) reports success at `83d447d`.
