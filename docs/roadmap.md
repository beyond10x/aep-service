# Roadmap — from scaffold to central AEP authority

The planning artifacts under `.engineering/planning/` are the work record. This page orders that
work and names the acceptance boundary of each wave; it does not duplicate artifact statuses.

## Wave 0 — publish the boundary

Agree and version the EP-owned command/query service wire, authentication-derived context and
compatibility rules. `aep-service` serves the contract; `engineering-protocols` keeps the official
client and `protocol` integration. Any shared verified bytes are introduced through an atlas ADR.

Work: `epic:application-service-boundary`.

## Wave 1 — one transactional authority

Run one company tenant with one globally unique planning realm and one repository workspace against
PostgreSQL. The schema and authorization boundary are realm-scoped from the first deployment, so
the service is multi-tenant even while the first tenant owns only one realm. A command is evaluated from
fresh state and commits state, relations, events, audit and idempotency memory atomically. Queries
come from indexed durable state rather than a process-wide hydrated copy.

Work: `epic:transactional-authority`, with the minimum runtime slice of
`epic:operable-service`.

## Wave 2 — trusted people and agents

Verify the assumed human and delegated-agent tokens, derive actor and executor server-side, enforce
workspace roles and delegation scopes, and authorize queries before traversal or materialization.
Once that boundary holds, expose an MCP adapter for agents. Its tools and resources project the same
EP-owned command/query service; MCP does not gain a second mutation path or a raw Entity Runtime
store surface.

Work: `epic:identity-and-access`, plus `story:mcp-agent-access` under
`epic:application-service-boundary`.

## Wave 3 — adopt repositories without losing Git review

Export consistent authorized snapshots, materialize deterministic Markdown through EP, and make one
repository use the service as authority while retaining committed generated documents and a drift
gate. Add cross-repository graph and blocker views after one repository is stable.

Work: `epic:projections-and-adoption` except Jira intake.

## Wave 4 — evolve the model deliberately

Register immutable EP bundles, preflight them against stored instances, activate a new default and
migrate instances with recorded events. Historical decisions retain the exact definition bytes
that produced them.

Work: `epic:definition-bundle-lifecycle`.

## Wave 5 — connect existing intake and operate continuously

Ingest Jira reports as externally sourced entities related to internal work, finish production
backup/restore and observability, and prove that security-relevant refusals and service health can be
audited without exposing restricted entity data.

Work: the remaining stories in `epic:projections-and-adoption` and `epic:operable-service`.

## Deliberately later

Company-brain entities may use the same hosting platform only after access control and projection
isolation are proven. They get a separate definition bundle and storage realm; they are not folded
into the planning model for graph convenience. A tenant may own both realms, but common ownership
grants no cross-realm read, relation or command path; tenant billing and realm provisioning remain
control-plane concerns outside the AEP wire.
