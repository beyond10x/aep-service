# aep-service

`aep-service` is the central, PostgreSQL-backed authority for Agentic Engineering Protocol (AEP)
entities. Humans and agents submit semantic commands and queries; the service derives trusted
attribution, applies authorization and protocol rules independently, and commits state, history,
events, relations, idempotency records, and refusals atomically.

This is a developer preview. The data and service boundaries are implemented, but production SSO
and delegated-agent token verification are deliberately not: the current development verifier uses
one exact bearer token and is loopback-only unless an operator opts into an insecure listener.

## Where it fits

- [AEP](https://github.com/beyond10x/aep) owns the entity
  vocabulary, semantic command/query traits, strict wire documents, the official client, and the
  canonical `aep` CLI (`protocol` remains an exact compatibility alias).
- [Entity Runtime](https://github.com/beyond10x/entity-runtime) owns deterministic entity execution
  and generic persistence/query providers.
- This repository owns the deployable application: trust-boundary orchestration, the HTTP adapter,
  PostgreSQL authority, operational limits, and deployment artifacts.

It is not a generic Entity Runtime store API and not a Jira-style product UI. Markdown remains a
deterministic projection for review or local use, never a second authoritative write path.

## Try the public surface

The shortest evaluation path uses the released OCI image and requires no Rust toolchain. Follow
the [published-image quickstart](https://beyond10x.github.io/aep-service/docs/quickstart) to start
PostgreSQL and AEP Service, submit a governed entity command, replay it, and inspect its history.

For source development, generate the same OpenAPI document served at `/openapi.json`:

```console
cargo run --locked -p aep-service -- openapi > openapi.json
```

Validate the immutable AEP definitions and capture the digest the service will pin:

```console
export AEP_DEFINITION_DIGEST="$(cargo run --quiet --locked -p aep-service -- \
  definitions digest --path ../aep)"
```

Then provide PostgreSQL and start the authority:

```console
export AEP_DATABASE_URL='postgresql://postgres:postgres@127.0.0.1:5432/aep'
export AEP_DEV_BEARER_TOKEN='replace-this-local-token'

cargo run --locked -p aep-service -- serve \
  --realm company-planning \
  --workspace example-repository \
  --schema company_planning \
  --definitions ../aep \
  --definition-digest "$AEP_DEFINITION_DIGEST"
```

The process binds `127.0.0.1:8080` by default. `/livez` reports process liveness, `/readyz` is
available only after definitions and PostgreSQL have been prepared, and the binary can probe either:

```console
cargo run --locked -p aep-service -- probe
cargo run --locked -p aep-service -- probe --readiness
```

`--allow-insecure-dev-listener` is required to expose the development verifier beyond loopback and
prints a warning. It is suitable only for isolated preview environments behind another trusted
boundary.

## Workspace

| crate | responsibility |
|---|---|
| `aep-service` | process configuration, listener, limits, probes, shutdown |
| `aep-service-app` | authenticated command/query orchestration |
| `aep-service-auth` | verified human and delegated-agent principal model |
| `aep-service-http` | HTTP realization of the AEP-owned contract |
| `aep-service-openapi` | deterministic OpenAPI projection from AEP routes and DTO schemas |
| `aep-service-postgres` | fresh transactional command sessions and indexed queries |

These are boundaries inside one deployable service, not a microservice decomposition.

## Development

Rust 1.85 or newer, `go-task`, `protocol`, and Node.js 20 are required for the complete gate.

```console
task check
task site-build
```

Set `ENTITY_POSTGRES_URL` to exercise the real-PostgreSQL injected-failure and competing-writer
tests locally. CI always supplies PostgreSQL. See [CONTRIBUTING.md](CONTRIBUTING.md),
[SECURITY.md](SECURITY.md), and the [public documentation](https://beyond10x.github.io/aep-service/).

Apache-2.0 licensed. No credential, company data, private transcript, or production configuration
belongs in this repository.
