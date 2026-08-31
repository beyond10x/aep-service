---
format: aep.planning-md/1
id: story:derived-openapi-description
kind: story
status: draft
title: Derive and serve the AEP OpenAPI description
summary: Generate OpenAPI from EP-owned wire types and expose the same deterministic bytes over HTTP and the website.
relations:
- decomposes: epic:public-developer-preview
- serves: vision:O2
- depends_on: epic:application-service-boundary
revision: 2
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
