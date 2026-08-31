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

Planning scaffold. The workspace establishes the intended component boundaries and compiles, but no
server is runnable yet. The work is governed in `.engineering/planning/` and ordered in
[`docs/roadmap.md`](docs/roadmap.md).

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

No credential, token, company data or production configuration belongs in this repository.

