---
format: aep.planning-md/1
id: story:public-product-documentation
kind: story
status: implemented
title: Publish a curated AEP service documentation site
summary: Explain the service, its trust boundary, local use and limits through a public Docusaurus site.
relations:
- decomposes: epic:public-developer-preview
- serves: vision:O2
- serves: vision:O6
revision: 6
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

## Implementation record

Read at `ed4ef5b`, one clause at a time:

- Landing page before any install instruction: `website/src/pages/index.tsx` sections `premise`
  (`:124`), `flow` (`:144`), `trace` (`:173`), `capability` (`:209`, the non-goals), `quickstart`
  (`:230`) and `status` (`:259`, current versus next).
- Grouped guides: `website/docs/` holds `intro`, `quickstart`, `architecture`, `concepts`,
  `commands-and-queries`, `http-contract`, `reliability`, `operations`, `configuration`, `security`
  and `release-status`.
- Published-image Compose path pinned to the release and bound to host loopback:
  `compose.preview.yaml:17` (`ghcr.io/beyond10x/aep-service:${AEP_SERVICE_VERSION:-0.1.8}`) and
  `:44-45` (`127.0.0.1:8080:8080`).
- Presentation: `website/docusaurus.config.ts:30-31` (social image and metadata),
  `website/src/css/custom.css:39` (dark theme), `:182` (`:focus-visible`), `:193` and
  `website/src/pages/index.module.css:507` (`prefers-reduced-motion`).
- The gate clause: `Taskfile.yml:41-49` is `task site-build` — `npm ci`, `npm audit --omit=dev
  --audit-level=critical`, `npm run typecheck`, `npm run build` — and
  `website/docusaurus.config.ts:15-18` sets `onBrokenLinks`, `onBrokenAnchors` and
  `onBrokenMarkdownLinks` to `throw`.

CHANGELOG `0.1.1` records the rebuild; release 0.1.8 ships it and Gate run 34914495607's `site` job
(`.github/workflows/gate.yml:66-80`, `task site-build`) reports success at `83d447d`.
