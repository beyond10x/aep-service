---
format: aep.planning-md/1
id: story:refresh-compatible-dependencies
kind: story
status: implemented
title: Refresh compatible service and website dependencies
summary: Upgrade the supported build, release, schema, and website dependency set together.
relations:
- derived_from: epic:public-developer-preview
- serves: vision:O4
scope:
- confidence: cited
  path: .github/workflows/audit.yml
- confidence: cited
  path: .github/workflows/b10x-docs-bundle.yml
- confidence: cited
  path: .github/workflows/gate.yml
- confidence: cited
  path: .github/workflows/pages.yml
- confidence: cited
  path: .github/workflows/release.yml
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Dockerfile
- confidence: cited
  path: website/package-lock.json
- confidence: cited
  path: website/package.json
- confidence: cited
  path: website/tsconfig.json
revision: 7
---
## Goal

Refresh AEP Service's supported build image, release actions, Rust schema dependency, and website toolchain from the current dependency queue while preserving the service and public documentation contracts.

## Acceptance

Every compatible AEP Service upgrade currently proposed by Dependabot is applied together, superseded internal-package downgrades are rejected, and `task check` plus `task site-build` pass from a clean install.

## Scope

- `Dockerfile`
- `.github/workflows/release.yml`
- `Cargo.toml` and `Cargo.lock`
- `website/package.json` and `website/package-lock.json`

