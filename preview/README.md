# Local preview bundle

The root `compose.yaml` builds the service, starts PostgreSQL, mounts a sibling
`../aep` checkout read-only as the immutable definition tree, and exposes the
explicitly insecure development listener only on host loopback.

Compute the exact digest expected by `aep_project::load_pinned_bundle` and start the bundle:

```console
git -C ../aep checkout 0.40.0
export AEP_DEFINITION_DIGEST="$(cargo run --quiet --locked -p aep-service -- \
  definitions digest --path ../aep)"
export AEP_DEV_BEARER_TOKEN=<local-only-token>
docker compose up --build
```

The digest command validates the complete tree before printing its 64-character identity. This is
intentionally not a production Helm chart. The fixed development bearer, default database
password, local build, and bind override make the trust assumptions visible rather than plausible.
