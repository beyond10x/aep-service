//! Deterministic `OpenAPI` 3.1 projection of the EP-owned service wire.
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
    let mut operation = json!({
        "operationId": route.operation.id(),
        "summary": summary(route.operation),
        "parameters": parameters,
        "responses": responses(route.media_type, success),
        "security": [{ "bearerAuth": [] }]
    });
    if let Some(schema) = request {
        operation
            .as_object_mut()
            .expect("operation is an object")
            .insert(
                "requestBody".to_owned(),
                json!({
                    "required": true,
                    "content": { route.media_type: { "schema": schema_ref(schema) } }
                }),
            );
    }
    json!({ route.method.as_str().to_ascii_lowercase(): operation })
}

fn responses(media_type: &str, success: &str) -> Value {
    let problem = json!({
        "description": "A typed AEP refusal or service failure.",
        "content": { media_type: { "schema": schema_ref("ProblemDocumentV1") } }
    });
    json!({
        "200": {
            "description": "The request was answered.",
            "content": { media_type: { "schema": schema_ref(success) } }
        },
        "400": problem,
        "401": problem,
        "403": problem,
        "404": problem,
        "409": problem,
        "422": problem,
        "503": problem,
        "504": problem
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

fn summary(operation: Operation) -> &'static str {
    match operation {
        Operation::Command => "Execute one semantic command",
        Operation::GetEntity => "Read one entity",
        Operation::ResolveEntity => "Resolve a logical entity address",
        Operation::QueryEntities => "Query entities",
        Operation::QueryRelations => "Query relations",
        Operation::GetHistory => "Read complete entity history",
        Operation::QueryHistory => "Query bounded entity history",
        Operation::QueryAudit => "Query attributable audit records",
        Operation::DescribeType => "Describe one registered entity type",
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
                "required": true,
                "schema": { "type": "string", "minLength": 1 }
            })
        })
        .collect()
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
}
