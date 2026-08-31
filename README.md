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

The versioned HTTP boundary, trusted request context and realm-scoped transactional PostgreSQL
authority are implemented as libraries. Each request opens a fresh EP backend; Entity Runtime's
PostgreSQL provider locks current revisions and atomically commits the complete command batch. No
server binary is runnable yet, and durable queries still hydrate the realm until the indexed-query
story replaces that temporary read path. Work is governed in `.engineering/planning/` and ordered
in [`docs/roadmap.md`](docs/roadmap.md).

## Workspace

| crate | responsibility |
|---|---|
| `aep-service-app` | authenticated command/query orchestration and application policy |
| `aep-service-auth` | verification of human and delegated-agent identity claims |
| `aep-service-postgres` | transactional AEP persistence, indexes and definition bundles |
| `aep-service-http` | versioned HTTP realization of the EP-owned service contract |

These are library boundaries inside one deployable service, not a microservice decomposition. The
service binary arrives with the HTTP-runtime story once there is an application worth starting.

## Local checks

```console
task check
```

Set `ENTITY_POSTGRES_URL` to make the gate run the injected-failure and competing-writer cases
against PostgreSQL. When it is unset, the gate prints that those cases were skipped.

No credential, token, company data or production configuration belongs in this repository.
