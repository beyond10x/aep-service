---
format: aep.planning-md/1
id: story:public-product-documentation
kind: story
status: draft
title: Publish a curated AEP service documentation site
summary: Explain the service, its trust boundary, local use and limits through a public Docusaurus site.
relations:
- decomposes: epic:public-developer-preview
- serves: vision:O2
- serves: vision:O6
revision: 1
---
## Context

Repository engineering records do not form a product guide, and duplicating them verbatim would turn internal status prose into an accidental public promise.

## Acceptance

A separately curated Docusaurus site builds with broken links refused, covers concepts, quickstart, protocol use, API reference, configuration, preview deployment, security limits and roadmap, and deploys to GitHub Pages from public material only.
