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
revision: 2
---
## Context

The first public site proves the deployment pipeline and states the correct boundaries, but its
single-screen landing page, terse guide set and source-build-first quickstart still read like a
repository placeholder. A technical evaluator cannot yet move cleanly from the reason the service
exists, through its architecture, to one successful command against the released image.

## Delivery

Redesign the site as a technical product journey: evaluate, understand, run, then integrate. Use
the beyond10x family typography and layout language with a distinct violet/cyan service identity,
accessible code-native diagrams, responsive light/dark presentation and explicit preview status.

The primary quickstart uses the published OCI image and a pinned Engineering Protocols definition
tree. It must start PostgreSQL and the service without a local Rust build, create and query an
entity, demonstrate idempotent replay and history, and clean up. Source-build instructions remain
for contributors rather than being the evaluation path.

## Acceptance

- The landing page explains the problem, system boundary, request/decision flow, evidence trail,
  non-goals and current-versus-next status before asking the reader to install anything.
- Grouped guides cover overview, quickstart, architecture, concepts, commands and queries, HTTP
  behavior, reliability semantics, operations, security and release status without duplicating EP
  as a second wire authority.
- A published-image Compose path is pinned to the release and binds the development verifier to
  host loopback; the documented flow is exercised from a clean directory without a Rust toolchain.
- Navigation, social metadata, code examples, light/dark modes, keyboard focus, reduced motion and
  desktop/tablet/mobile layouts form one coherent public experience.
- `task site-build` refuses type errors, critical production dependency advisories, broken links or
  broken anchors and emits the complete static site for GitHub Pages.

## Non-goals

No hosted backend, browser-side token storage, production identity, MCP surface, repository
projection or Jira-style product UI is introduced by this story.
