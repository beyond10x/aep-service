---
title: Run the OCI preview
description: Start PostgreSQL and AEP Service, create a governed story, replay it safely and inspect its history without building Rust.
---

# From an empty database to a governed story

This path uses the released multiarch image. You need Docker with Compose, Git, curl and jq; you do
not need a Rust toolchain. Everything binds to host loopback and uses disposable local data.

:::warning Development authentication
The preview token is not production authentication. Use these commands only on a development
machine, never with production data or a publicly reachable Docker host.
:::

## 1. Pin the service and definitions

Create a clean directory and clone the exact source generations used by this guide:

```console
mkdir aep-evaluation && cd aep-evaluation
git clone --branch 0.1.1 --depth 1 https://github.com/beyond10x/aep-service.git
git clone --branch 0.38.1 --depth 1 https://github.com/beyond10x/aep.git
cd aep-service
```

Choose a local token and ask the released image to validate and digest the definition tree:

```console
export AEP_DEV_BEARER_TOKEN='local-preview-change-me'
export AEP_DEFINITIONS_PATH='../aep'
export AEP_DEFINITION_DIGEST="$(docker run --rm \
  --volume "$(realpath "$AEP_DEFINITIONS_PATH"):/definitions:ro" \
  ghcr.io/beyond10x/aep-service:0.1.1 \
  definitions digest --path /definitions)"
```

The command refuses an invalid definition tree rather than hashing bytes it cannot load.

## 2. Start one local authority

```console
docker compose -f compose.preview.yaml up --detach
docker compose -f compose.preview.yaml ps
curl --fail http://127.0.0.1:8080/readyz
```

Compose starts PostgreSQL 17, then the non-root service image. The container listener uses the
explicit insecure-development override because container loopback is not host loopback; the port
mapping remains `127.0.0.1:8080` on the host.

Define the strict version-1 media type once:

```console
export AEP_MEDIA_TYPE='application/vnd.aep.service+json;version=1'
```

## 3. Submit a semantic create command

Write the command once so the replay below is byte-for-byte obvious:

```console
cat > create-story.json <<'JSON'
{
  "command_id": "docs-create-story",
  "idempotency_key": "docs-create-story-v1",
  "command_type": "aep.entity.create/v1",
  "target": null,
  "expected_revision": null,
  "correlation_id": "docs-evaluation",
  "causation": null,
  "execution_id": null,
  "task": null,
  "payload": {
    "command": "create-entity",
    "entity_type": "aep.story/v1",
    "locator": "ep://demo/plan/story/first-governed-story",
    "data": {"status": "draft", "title": "First governed story"}
  }
}
JSON

response="$(curl --fail-with-body \
  --request POST \
  --header "Authorization: Bearer $AEP_DEV_BEARER_TOKEN" \
  --header "Accept: $AEP_MEDIA_TYPE" \
  --header "Content-Type: $AEP_MEDIA_TYPE" \
  --data @create-story.json \
  http://127.0.0.1:8080/aep/v1/realms/demo/workspaces/plan/commands)"

printf '%s\n' "$response" | jq .
export AEP_ENTITY_ID="$(printf '%s' "$response" | jq -r '.result.affected[0] | sub("@.*$"; "")')"
```

The response carries the server request id, command outcome, revision-qualified affected entity
reference, emitted records and a consistency token. Actor and received time do not appear in the
request because the server derives them.

## 4. Query and replay

Read the entity the command created:

```console
curl --fail-with-body \
  --header "Authorization: Bearer $AEP_DEV_BEARER_TOKEN" \
  --header "Accept: $AEP_MEDIA_TYPE" \
  "http://127.0.0.1:8080/aep/v1/realms/demo/workspaces/plan/entities/$AEP_ENTITY_ID" | jq .
```

Replay the exact command document:

```console
curl --fail-with-body \
  --request POST \
  --header "Authorization: Bearer $AEP_DEV_BEARER_TOKEN" \
  --header "Accept: $AEP_MEDIA_TYPE" \
  --header "Content-Type: $AEP_MEDIA_TYPE" \
  --data @create-story.json \
  http://127.0.0.1:8080/aep/v1/realms/demo/workspaces/plan/commands | jq .
```

The result reports `replayed`; it does not create a second story. Reusing the same idempotency key
for different intent produces a typed conflict instead.

Inspect the immutable revision sequence:

```console
curl --fail-with-body \
  --header "Authorization: Bearer $AEP_DEV_BEARER_TOKEN" \
  --header "Accept: $AEP_MEDIA_TYPE" \
  "http://127.0.0.1:8080/aep/v1/realms/demo/workspaces/plan/entities/$AEP_ENTITY_ID/history" | jq .
```

## 5. Clean up

```console
docker compose -f compose.preview.yaml down --volumes
cd ../..
```

Continue with [commands and queries](./commands-and-queries), or inspect every operation in the
[derived API reference](/aep-service/api).
