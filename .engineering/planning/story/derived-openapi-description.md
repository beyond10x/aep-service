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
revision: 1
---
## Context

The service implements a strict versioned wire but has no machine-readable HTTP description; a hand-maintained schema would create a second contract beside the EP-owned DTOs.

## Acceptance

EP wire DTOs and route metadata generate one deterministic OpenAPI 3.1 document, the service parser and document paths are proven identical, `/openapi.json` is public without touching auth or PostgreSQL, and the website renders the committed checked artifact.
