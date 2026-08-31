# aep-service

The authenticated central authority for Agentic Engineering Protocol entities. It accepts semantic
AEP commands and queries, derives trusted actor/executor attribution from authenticated requests,
evaluates authorization and protocol rules, and persists the resulting state, history and refusals
transactionally through Entity Runtime providers.

The service is deliberately headless. `engineering-protocols` owns the AEP vocabulary, wire
contract, official client and `protocol` CLI; `entity-runtime` owns deterministic entity execution
and generic stores. This repository owns the deployed application between those contracts and
PostgreSQL.

```text
protocol CLI
    |
    | authenticated AEP commands and queries
    v
aep-service
    |-- trusted identity and delegated-agent authorization
    |-- EP command/query evaluation
    |-- definition bundle selection
    |-- authorized snapshot export
    `-- Entity Runtime providers --> PostgreSQL
```

## Status

The first central-authority slice is runnable. The service loads and verifies one immutable EP
definition bundle before becoming reachable, opens a fresh transactional PostgreSQL command session
for each request, and answers bounded queries through Entity Runtime's durable indexes rather than a
process-wide realm copy. HTTP wire v1 remains compatible with EP's constructed corpus; wire v2 adds
bounded cursor-based history. Authentication is deliberately limited to a loopback-only development
bearer until the identity wave lands. Work is governed in `.engineering/planning/` and ordered in
[`docs/roadmap.md`](docs/roadmap.md).

## Workspace

| crate | responsibility |
|---|---|
| `aep-service` | runnable HTTP process, startup configuration and graceful shutdown |
| `aep-service-app` | authenticated command/query orchestration and application policy |
| `aep-service-auth` | verification of human and delegated-agent identity claims |
| `aep-service-postgres` | transactional AEP persistence, indexes and definition bundles |
| `aep-service-http` | versioned HTTP realization of the EP-owned service contract |

These are library boundaries inside one deployable service, not a microservice decomposition. The
binary currently serves one configured realm/workspace authority per process; tenant and realm
provisioning remain a later control-plane concern.

## Run locally

The development verifier refuses non-loopback listeners. Put the database URL and an exact bearer
token in environment variables, then name the EP definitions and their pinned digest explicitly:

```console
export AEP_DATABASE_URL='postgresql://...'
export AEP_DEV_BEARER_TOKEN='local-secret'
aep-service serve \
  --realm company-planning \
  --workspace aep-service \
  --schema company_planning \
  --definitions ../engineering-protocols \
  --definition-digest <sha256>
```

`/livez` reports that the process is serving, and `/readyz` exists only after the definition bundle
has verified and PostgreSQL preparation has succeeded. The development token is an explicit local
bootstrap seam, not a production authentication mode.

## Local checks

```console
task check
```

Set `ENTITY_POSTGRES_URL` to make the gate run the injected-failure and competing-writer cases
against PostgreSQL. When it is unset, the gate prints that those cases were skipped.

No credential, token, company data or production configuration belongs in this repository.
