---
sidebar_position: 1
title: What AEP Service is
---

# One authority, many useful projections

AEP Service makes engineering entities centrally addressable without turning repository Markdown
or PostgreSQL into competing write paths. Humans, agents, and integrations submit semantic AEP
commands and queries. The service owns trusted attribution, authorization, operational policy, and
transaction boundaries.

Engineering Protocols owns the entity vocabulary and service contract. Entity Runtime owns the
deterministic kernel and generic providers. AEP Service composes them into an operable authority.

The result is deliberately narrower than an issue tracker product: no sprint UI, comment system,
notification marketplace, customer queue, or arbitrary workflow editor. Repository Markdown,
dashboards, and integrations are projections over the same entity record.

## Preview status

The transactional and wire boundaries are available for evaluation. Production identity is not.
Use the development verifier only on loopback, or behind an explicit isolated preview boundary.
