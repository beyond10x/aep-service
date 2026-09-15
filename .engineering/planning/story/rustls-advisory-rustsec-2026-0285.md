---
format: aep.planning-md/1
id: story:rustls-advisory-rustsec-2026-0285
kind: story
status: implemented
title: Update rustls for RUSTSEC-2026-0285
summary: cargo audit reports RUSTSEC-2026-0285 against the locked rustls; cargo update -p rustls and re-run the gate. Observed 2026-09-15 at a47f8ca by the fix pass.
relations:
- serves: vision:O6
- decomposes: epic:public-developer-preview
scope:
- confidence: cited
  path: Cargo.lock
revision: 7
---
# Update rustls for RUSTSEC-2026-0285

## Acceptance

`cargo deny advisories` inside `task check` no longer refuses the locked `rustls`, and the Gate
workflow reports `gate` and `site` at `success` on the commit that carries the updated `Cargo.lock`.

## Implementation record

Commit `83d447d` ("fix: update rustls for RUSTSEC-2026-0285") moves `rustls` to the patched release
in `Cargo.lock` with no source change. Gate run 34914495607 reports `gate` and `site` both `success`
on `83d447d`; commit `af9bd21` recorded that run as this story's `test_result` evidence
(`github-actions`, 2026-09-15T00:55:20Z).
