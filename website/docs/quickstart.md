---
sidebar_position: 2
title: Quickstart
---

# Run the developer preview

You need Rust 1.85+, PostgreSQL, and an Engineering Protocols definition tree. Clone the repository
and inspect the generated contract first:

```console
cargo run --locked -p aep-service -- openapi > openapi.json
```

The service release pins Engineering Protocols 0.38.1; use that tag for the sibling definition tree.

Then configure a disposable database and local token:

```console
export AEP_DATABASE_URL='postgresql://postgres:postgres@127.0.0.1:5432/aep'
export AEP_DEV_BEARER_TOKEN='replace-this-local-token'
export AEP_DEFINITION_DIGEST="$(cargo run --quiet --locked -p aep-service -- \
  definitions digest --path ../engineering-protocols)"

cargo run --locked -p aep-service -- serve \
  --realm company-planning \
  --workspace example-repository \
  --schema company_planning \
  --definitions ../engineering-protocols \
  --definition-digest "$AEP_DEFINITION_DIGEST"
```

Check the process from another terminal:

```console
cargo run --locked -p aep-service -- probe --readiness
curl http://127.0.0.1:8080/openapi.json
```

The exact bearer belongs in the `Authorization` header for semantic routes. Do not place the token
in a repository, command transcript, issue, or generated projection.
