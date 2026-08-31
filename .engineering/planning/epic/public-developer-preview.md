---
format: aep.planning-md/1
id: epic:public-developer-preview
kind: epic
status: draft
title: A credible public developer preview
summary: Publish a self-contained, inspectable and locally runnable service without claiming production authentication.
relations:
- decomposes: initiative:central-aep-authority
- serves: vision:O1
- serves: vision:O2
- serves: vision:O6
revision: 1
---
## Outcome

The repository, API contract, documentation site, release artifacts and local container experience are public and verifiable while every production limitation remains explicit.

## Acceptance

A clean public checkout passes its complete gate, renders its product site, serves generated OpenAPI, runs locally through the preview Compose path, and publishes reproducible release artifacts without a credential or private dependency.
