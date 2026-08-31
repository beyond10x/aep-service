---
format: aep.planning-md/1
id: story:public-ci-and-supply-chain
kind: story
status: draft
title: Make the public gate and supply chain visible
summary: Run the real PostgreSQL gate, contract drift, MSRV, site and dependency policy in public automation.
relations:
- decomposes: epic:public-developer-preview
- serves: vision:O1
- serves: vision:O6
revision: 1
---
## Context

The private repository has no CI, license file, dependency policy, contribution posture or public security-reporting path.

## Acceptance

Pull requests and releases share one pinned-action gate with PostgreSQL, MSRV, OpenAPI and planning checks; Pages and scheduled audits are separate visible jobs; Apache-2.0 licensing and issues-only participation are explicit; private vulnerability reporting and branch protection are enabled.
