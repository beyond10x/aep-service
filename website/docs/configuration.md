---
title: Configuration reference
description: Command-line and environment configuration for the AEP Service preview process.
---

# Configure one preview authority

The `serve` command receives non-secret policy as arguments and reads secrets through named
environment variables.

## Required deployment values

| Argument | Meaning |
|---|---|
| `--realm` | Globally scoped realm served by this process |
| `--workspace` | One workspace admitted by this process |
| `--schema` | PostgreSQL schema dedicated to the configured authority |
| `--definitions` | Root of the immutable AEP definition tree |
| `--definition-digest` | Expected lowercase SHA-256 for the validated tree |

The database URL defaults to environment variable `AEP_DATABASE_URL`; change the variable name with
`--database-url-env`. The development token defaults to `AEP_DEV_BEARER_TOKEN`; change it with
`--dev-token-env`.

## Hosted identity

| Argument | Environment | Meaning |
|---|---|---|
| `--identity-origin` | `AEP_IDENTITY_ORIGIN` | Exact hosted Identity origin |
| `--identity-audience` | `AEP_IDENTITY_AUDIENCE` | Relying-party audience; defaults to `urn:b10x:aep-service` |
| `--identity-tenant` | `AEP_IDENTITY_TENANT` | Exact tenant admitted to this authority |

Origin and tenant are configured together. In hosted mode each bearer is resolved through Identity;
the development token and its non-loopback override are not used.

## Listener and development identity

| Argument | Default | Meaning |
|---|---:|---|
| `--bind` | `127.0.0.1:8080` | Listener address |
| `--dev-authority` | `human:developer` | Authority attributed to accepted development requests |
| `--allow-insecure-dev-listener` | off | Explicitly permit the development verifier on non-loopback |

The override prints a warning and does not make the development token production-safe.

## Resource bounds

| Argument | Default | Meaning |
|---|---:|---|
| `--database-concurrency` | `32` | Maximum simultaneous blocking database exchanges |
| `--queue-timeout-ms` | `2000` | Maximum wait for a database execution slot |
| `--request-timeout-ms` | `30000` | Maximum duration of one database-backed exchange |
| `--shutdown-timeout-ms` | `15000` | Maximum graceful drain after SIGINT or SIGTERM |

Zero concurrency or zero timeout values are refused at startup.

## Utility commands

```console
aep-service definitions digest --path /definitions
aep-service openapi
aep-service probe --address 127.0.0.1:8080
aep-service probe --address 127.0.0.1:8080 --readiness
```

`definitions digest` validates before printing the identity. `openapi` writes exactly the bytes
served at `/openapi.json`. `probe` bounds its own connection and response time; override the
two-second default with `--timeout-ms`.
