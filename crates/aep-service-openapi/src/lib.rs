//! Deterministic `OpenAPI` 3.1 projection of the AEP-owned service wire.
//!
//! Paths come from [`aep_client::wire::ROUTES`] and payload schemas come from the strict wire
//! structs' [`schemars::JsonSchema`] implementations. This crate owns presentation metadata only.

use std::collections::BTreeMap;

use aep_client::wire::{
    AuditPageV1, AuditQueryV1, CommandRequestV1, CommandResultV1, EntityPageV1, EntityQueryV1,
    HistoryQueryV2, Operation, PageV2, ProblemDocumentV1, RelationPageV1, RelationQueryV1,
    ResolveRequestV1, ResolvedEntityV1, RouteSpec, SuccessV1, TypeDescriptionV1, ROUTES,
};
use aep_contract::query::{EntityEnvelope, RevisionRecord};
use schemars::JsonSchema;
use serde_json::{json, Map, Value};

/// Builds the complete `OpenAPI` document as a deterministic JSON value.
pub fn document() -> Value {
    let mut schemas = BTreeMap::new();
    register::<CommandRequestV1>("CommandRequestV1", &mut schemas);
    register::<SuccessV1<CommandResultV1>>("CommandResponseV1", &mut schemas);
    register::<SuccessV1<EntityEnvelope>>("EntityResponseV1", &mut schemas);
    register::<ResolveRequestV1>("ResolveRequestV1", &mut schemas);
    register::<SuccessV1<ResolvedEntityV1>>("ResolveResponseV1", &mut schemas);
    register::<EntityQueryV1>("EntityQueryV1", &mut schemas);
    register::<SuccessV1<EntityPageV1>>("EntityQueryResponseV1", &mut schemas);
    register::<RelationQueryV1>("RelationQueryV1", &mut schemas);
    register::<SuccessV1<RelationPageV1>>("RelationQueryResponseV1", &mut schemas);
    register::<SuccessV1<Vec<RevisionRecord>>>("HistoryResponseV1", &mut schemas);
    register::<HistoryQueryV2>("HistoryQueryV2", &mut schemas);
    register::<SuccessV1<PageV2<RevisionRecord>>>("HistoryQueryResponseV2", &mut schemas);
    register::<AuditQueryV1>("AuditQueryV1", &mut schemas);
    register::<SuccessV1<AuditPageV1>>("AuditQueryResponseV1", &mut schemas);
    register::<SuccessV1<TypeDescriptionV1>>("TypeDescriptionResponseV1", &mut schemas);
    register::<ProblemDocumentV1>("ProblemDocumentV1", &mut schemas);

    let paths = ROUTES
        .iter()
        .map(|route| (route.path(), path_item(route)))
        .collect::<Map<_, _>>();
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "AEP Service API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Authenticated semantic commands and queries for Agentic Engineering Protocol entities. PostgreSQL and Entity Runtime provider operations are not public APIs."
        },
        "servers": [{ "url": "/" }],
        "externalDocs": {
            "description": "AEP Service documentation",
            "url": "https://beyond10x.github.io/aep-service/docs/intro"
        },
        "tags": [
            { "name": "Commands", "description": "Submit one attributable semantic state change." },
            { "name": "Entities", "description": "Resolve, read and query current entity state." },
            { "name": "Relations", "description": "Query typed edges between entities." },
            { "name": "History", "description": "Read immutable entity revisions." },
            { "name": "Audit", "description": "Query attributable accepted and refused activity." },
            { "name": "Types", "description": "Inspect registered entity capabilities and lifecycles." }
        ],
        "paths": paths,
        "components": {
            "securitySchemes": {
                "bearerAuth": { "type": "http", "scheme": "bearer" }
            },
            "schemas": schemas
        },
        "security": [{ "bearerAuth": [] }]
    })
}

/// Serializes [`document`] as canonical pretty JSON with one trailing line feed.
pub fn document_bytes() -> Vec<u8> {
    let mut bytes =
        serde_json::to_vec_pretty(&document()).expect("the OpenAPI projection serializes");
    bytes.push(b'\n');
    bytes
}

fn path_item(route: &RouteSpec) -> Value {
    let request = request_schema(route.operation);
    let success = success_schema(route.operation);
    let parameters = path_parameters(&route.path());
    let (tag, summary, description) = operation_details(route.operation);
    let mut operation = json!({
        "operationId": route.operation.id(),
        "tags": [tag],
        "summary": summary,
        "description": description,
        "parameters": parameters,
        "responses": responses(route.operation, route.media_type, success),
        "security": [{ "bearerAuth": [] }]
    });
    if let Some(schema) = request {
        let example = request_example(route.operation);
        operation
            .as_object_mut()
            .expect("operation is an object")
            .insert(
                "requestBody".to_owned(),
                json!({
                    "required": true,
                    "content": {
                        route.media_type: {
                            "schema": schema_ref(schema),
                            "examples": {
                                "request": {
                                    "summary": request_example_summary(route.operation),
                                    "value": example
                                }
                            }
                        }
                    }
                }),
            );
    }
    json!({ route.method.as_str().to_ascii_lowercase(): operation })
}

fn responses(operation: Operation, media_type: &str, success: &str) -> Value {
    let problem = |status| {
        json!({
            "description": problem_description(status),
            "content": {
                media_type: {
                    "schema": schema_ref("ProblemDocumentV1"),
                    "examples": {
                        "problem": {
                            "summary": problem_example_summary(status),
                            "value": problem_example(status)
                        }
                    }
                }
            }
        })
    };
    let mut success_content = json!({ "schema": schema_ref(success) });
    if let Some(example) = success_example(operation) {
        success_content
            .as_object_mut()
            .expect("success content is an object")
            .insert(
                "examples".to_owned(),
                json!({ "answer": { "summary": "An answered request", "value": example } }),
            );
    }
    json!({
        "200": {
            "description": "The request was answered.",
            "content": { media_type: success_content }
        },
        "400": problem(400),
        "401": problem(401),
        "403": problem(403),
        "404": problem(404),
        "409": problem(409),
        "422": problem(422),
        "503": problem(503),
        "504": problem(504)
    })
}

fn request_schema(operation: Operation) -> Option<&'static str> {
    match operation {
        Operation::Command => Some("CommandRequestV1"),
        Operation::ResolveEntity => Some("ResolveRequestV1"),
        Operation::QueryEntities => Some("EntityQueryV1"),
        Operation::QueryRelations => Some("RelationQueryV1"),
        Operation::QueryHistory => Some("HistoryQueryV2"),
        Operation::QueryAudit => Some("AuditQueryV1"),
        Operation::GetEntity | Operation::GetHistory | Operation::DescribeType => None,
    }
}

fn success_schema(operation: Operation) -> &'static str {
    match operation {
        Operation::Command => "CommandResponseV1",
        Operation::GetEntity => "EntityResponseV1",
        Operation::ResolveEntity => "ResolveResponseV1",
        Operation::QueryEntities => "EntityQueryResponseV1",
        Operation::QueryRelations => "RelationQueryResponseV1",
        Operation::GetHistory => "HistoryResponseV1",
        Operation::QueryHistory => "HistoryQueryResponseV2",
        Operation::QueryAudit => "AuditQueryResponseV1",
        Operation::DescribeType => "TypeDescriptionResponseV1",
    }
}

fn operation_details(operation: Operation) -> (&'static str, &'static str, &'static str) {
    match operation {
        Operation::Command => (
            "Commands",
            "Execute one semantic command",
            "Submit one versioned AEP intention. The service derives actor, executor, request identity and received time from verified server context, then commits the complete outcome or no mutation.",
        ),
        Operation::GetEntity => (
            "Entities",
            "Read one entity",
            "Read current entity state by immutable entity identity inside the authorized realm and workspace.",
        ),
        Operation::ResolveEntity => (
            "Entities",
            "Resolve a logical entity address",
            "Resolve a stable `ep://` locator to its current immutable entity identity and revision.",
        ),
        Operation::QueryEntities => (
            "Entities",
            "Query entities",
            "Query current entities with exact typed filters, bounded pagination and an explicit consistency demand.",
        ),
        Operation::QueryRelations => (
            "Relations",
            "Query relations",
            "Query typed incoming or outgoing entity edges without exposing the underlying provider representation.",
        ),
        Operation::GetHistory => (
            "History",
            "Read complete entity history",
            "Read the immutable revision sequence for one authorized entity.",
        ),
        Operation::QueryHistory => (
            "History",
            "Query bounded entity history",
            "Page through immutable revisions with an opaque continuation cursor and consistency demand.",
        ),
        Operation::QueryAudit => (
            "Audit",
            "Query attributable audit records",
            "Query accepted and refused activity by entity, correlation, command, authority, kind or time range.",
        ),
        Operation::DescribeType => (
            "Types",
            "Describe one registered entity type",
            "Inspect the commands, lifecycle and relations exposed by the active immutable definition bundle.",
        ),
    }
}

fn path_parameters(path: &str) -> Vec<Value> {
    ["realm", "workspace", "entity", "entity_type"]
        .into_iter()
        .filter(|name| path.contains(&format!("{{{name}}}")))
        .map(|name| {
            json!({
                "name": name,
                "in": "path",
                "description": path_parameter_description(name),
                "required": true,
                "schema": { "type": "string", "minLength": 1 }
            })
        })
        .collect()
}

fn path_parameter_description(name: &str) -> &'static str {
    match name {
        "realm" => "The isolated policy, definition and data boundary.",
        "workspace" => "The repository or collaboration surface inside the realm.",
        "entity" => "The immutable entity identity.",
        "entity_type" => "The versioned entity type, URL-encoded where necessary.",
        _ => "A required path parameter.",
    }
}

macro_rules! typed_example {
    ($type:ty, $value:expr) => {{
        let typed: $type =
            serde_json::from_value($value).expect("the documentation example is valid");
        serde_json::to_value(typed).expect("the documentation example serializes")
    }};
}

fn request_example(operation: Operation) -> Value {
    match operation {
        Operation::Command => typed_example!(
            CommandRequestV1,
            json!({
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
                    "data": { "status": "draft", "title": "First governed story" }
                }
            })
        ),
        Operation::ResolveEntity => typed_example!(
            ResolveRequestV1,
            json!({
                "locator": "ep://demo/plan/story/first-governed-story"
            })
        ),
        Operation::QueryEntities => typed_example!(
            EntityQueryV1,
            json!({
                "entity_type": "aep.story/v1",
                "organisation": "demo",
                "space": "plan",
                "matching": { "status": "draft" },
                "related_to": null,
                "relation": null,
                "limit": 25,
                "after": null,
                "consistency": { "consistency": "current" }
            })
        ),
        Operation::QueryRelations => typed_example!(
            RelationQueryV1,
            json!({
                "source": "01K2R8JD3ZJME72AJGQY67E5F8",
                "target": null,
                "kind": "decomposes",
                "limit": 25,
                "after": null,
                "consistency": { "consistency": "current" }
            })
        ),
        Operation::QueryHistory => typed_example!(
            HistoryQueryV2,
            json!({
                "entity": "01K2R8JD3ZJME72AJGQY67E5F8",
                "limit": 25,
                "after": null,
                "consistency": { "consistency": "current" }
            })
        ),
        Operation::QueryAudit => typed_example!(
            AuditQueryV1,
            json!({
                "entity": null,
                "correlation_id": "docs-evaluation",
                "command_id": null,
                "actor": "human:developer",
                "kind": null,
                "since": null,
                "until": null,
                "rejected_only": false,
                "limit": 25,
                "after": null
            })
        ),
        Operation::GetEntity | Operation::GetHistory | Operation::DescribeType => {
            unreachable!("bodyless operations do not request examples")
        }
    }
}

fn request_example_summary(operation: Operation) -> &'static str {
    match operation {
        Operation::Command => "Create a draft story",
        Operation::ResolveEntity => "Resolve a story locator",
        Operation::QueryEntities => "Find draft stories",
        Operation::QueryRelations => "Find outgoing decomposition relations",
        Operation::QueryHistory => "Read the first page of revisions",
        Operation::QueryAudit => "Find activity in one correlation",
        Operation::GetEntity | Operation::GetHistory | Operation::DescribeType => "Request",
    }
}

fn success_example(operation: Operation) -> Option<Value> {
    let case_name = match operation {
        Operation::Command => "accepted-human-command",
        Operation::QueryEntities => "authorized-entity-query",
        _ => return None,
    };
    let case = aep_client::conformance::CASES
        .iter()
        .find(|case| case.name == case_name)
        .expect("the released conformance case exists");
    Some(serde_json::from_slice(case.response.body).expect("the conformance response is JSON"))
}

fn problem_description(status: u16) -> &'static str {
    match status {
        400 => "The request document or media negotiation is malformed.",
        401 => "No credential was accepted.",
        403 => "The verified principal is outside the requested realm or workspace.",
        404 => "The requested entity, locator or type does not exist.",
        409 => "The semantic command, revision guard or idempotency identity conflicts with current state.",
        422 => "The strict document is structurally valid but semantically invalid.",
        503 => "The database authority or bounded execution capacity is temporarily unavailable.",
        504 => "The database-backed exchange exceeded its configured deadline.",
        _ => "A typed AEP refusal or service failure.",
    }
}

fn problem_example_summary(status: u16) -> &'static str {
    match status {
        400 => "Malformed strict request",
        401 => "Credential required",
        403 => "Workspace outside the grant",
        404 => "Entity not found",
        409 => "Semantic or revision conflict",
        422 => "Invalid semantic input",
        503 => "Authority temporarily unavailable",
        504 => "Exchange deadline exceeded",
        _ => "Typed problem",
    }
}

fn problem_example(status: u16) -> Value {
    let corpus_case = match status {
        400 => Some("malformed-command"),
        401 => Some("unauthenticated-command"),
        403 => Some("workspace-unauthorized-command"),
        409 => Some("revision-conflict"),
        503 => Some("service-unavailable"),
        _ => None,
    };
    if let Some(name) = corpus_case {
        let case = aep_client::conformance::CASES
            .iter()
            .find(|case| case.name == name)
            .expect("the released conformance case exists");
        return serde_json::from_slice(case.response.body)
            .expect("the conformance problem is JSON");
    }
    let (code, message, retryable, details) = match status {
        404 => (
            "not_found",
            "the requested entity does not exist",
            false,
            json!({
                "entity": "01K2R8JD3ZJME72AJGQY67E5F8"
            }),
        ),
        422 => (
            "invalid",
            "the semantic request is invalid",
            false,
            json!({
                "errors": [{ "path": "payload.data.title", "reason": "must not be empty" }]
            }),
        ),
        504 => (
            "unavailable",
            "database exchange exceeded its deadline",
            true,
            json!({
                "reason": "database exchange exceeded its deadline"
            }),
        ),
        _ => (
            "unavailable",
            "the service is unavailable",
            true,
            json!({
                "reason": "temporarily unavailable"
            }),
        ),
    };
    typed_example!(
        ProblemDocumentV1,
        json!({
            "request_id": "0198f03a-7b62-7000-8000-000000000001",
            "error": {
                "code": code,
                "message": message,
                "retryable": retryable,
                "details": details
            }
        })
    )
}

fn schema_ref(name: &str) -> Value {
    json!({ "$ref": format!("#/components/schemas/{name}") })
}

fn register<T: JsonSchema>(name: &str, schemas: &mut BTreeMap<String, Value>) {
    let root = schemars::schema_for!(T);
    for (definition_name, definition) in root.definitions {
        insert_schema(
            definition_name,
            rewrite_references(serde_json::to_value(definition).expect("schema serializes")),
            schemas,
        );
    }
    let mut value = serde_json::to_value(root.schema).expect("root schema serializes");
    if let Some(object) = value.as_object_mut() {
        object.remove("$schema");
    }
    insert_schema(name.to_owned(), rewrite_references(value), schemas);
}

fn insert_schema(name: String, value: Value, schemas: &mut BTreeMap<String, Value>) {
    if let Some(existing) = schemas.get(&name) {
        assert_eq!(existing, &value, "schema name {name} has two meanings");
    } else {
        schemas.insert(name, value);
    }
}

fn rewrite_references(mut value: Value) -> Value {
    match &mut value {
        Value::Object(object) => {
            for member in object.values_mut() {
                *member = rewrite_references(member.take());
            }
        }
        Value::Array(items) => {
            for item in items {
                *item = rewrite_references(item.take());
            }
        }
        Value::String(reference) if reference.starts_with("#/definitions/") => {
            *reference = reference.replacen("#/definitions/", "#/components/schemas/", 1);
        }
        _ => {}
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_ep_route_is_present_once_with_its_stable_operation_id() {
        let document = document();
        let paths = document["paths"].as_object().expect("paths object");
        assert_eq!(paths.len(), ROUTES.len());
        for route in ROUTES {
            assert_eq!(
                paths[&route.path()][route.method.as_str().to_ascii_lowercase()]["operationId"],
                route.operation.id()
            );
        }
    }

    #[test]
    fn two_projections_are_byte_identical_and_hold_no_legacy_definition_references() {
        let first = document_bytes();
        assert_eq!(first, document_bytes());
        assert!(!String::from_utf8(first).unwrap().contains("#/definitions/"));
    }

    #[test]
    fn every_operation_is_documented_and_every_example_is_a_strict_wire_value() {
        let document = document();
        for route in ROUTES {
            let operation =
                &document["paths"][&route.path()][route.method.as_str().to_ascii_lowercase()];
            assert!(
                operation["description"]
                    .as_str()
                    .is_some_and(|text| !text.is_empty()),
                "{} has a description",
                route.operation.id()
            );
            assert_eq!(
                operation["tags"].as_array().map(Vec::len),
                Some(1),
                "{} has one stable group",
                route.operation.id()
            );
            if request_schema(route.operation).is_some() {
                assert_ne!(
                    operation["requestBody"]["content"][route.media_type]["examples"]["request"]
                        ["value"],
                    Value::Null,
                    "{} has a typed request example",
                    route.operation.id()
                );
            }
        }

        let request: CommandRequestV1 = serde_json::from_value(request_example(Operation::Command))
            .expect("the rendered command remains a strict request");
        assert_eq!(
            request.command_type,
            request
                .decode_command()
                .expect("the example payload is a semantic command")
                .kind()
                .as_str()
        );
        for status in [400, 401, 403, 404, 409, 422, 503, 504] {
            serde_json::from_value::<ProblemDocumentV1>(problem_example(status))
                .unwrap_or_else(|error| panic!("status {status} has a strict problem: {error}"));
        }
    }
}
