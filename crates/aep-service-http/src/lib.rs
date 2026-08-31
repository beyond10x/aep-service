//! HTTP realization of the EP-owned command/query service contract.
//!
//! This adapter validates wire versions and paths, verifies credentials, authorizes the requested
//! realm/workspace and only then decodes or dispatches semantic intent. It owns no AEP decision
//! semantics and exposes no Entity Runtime storage operation.

use std::collections::BTreeMap;

use aep_client::wire::{
    self, AuditQueryV1, CommandRequestV1, EntityQueryV1, HistoryQueryV2, Method, PageV2,
    ProblemDocumentV1, ProblemMappingV1, RelationQueryV1, Request, ResolveRequestV1, Response,
    SuccessV1, CONSISTENCY_HEADER, MEDIA_TYPE_V1, MEDIA_TYPE_V2, SUPPORTED_VERSIONS_HEADER,
};
use aep_contract::command::{CommandEnvelope, CommandService};
use aep_contract::error::{CommandError, QueryError};
use aep_contract::query::{AuditQuery, EntityQuery, HistoryQuery, QueryService, RelationQuery};
use aep_contract::{ConsistencyToken, QueryConsistency};
use aep_domain::audit::AuditRecord;
use aep_domain::command::Command;
use aep_domain::entity::{EntityId, EntityRef, EntityType};
use aep_domain::error::{ValidationCode, ValidationError, ValidationErrors};
use aep_domain::ids::RequestId;
use aep_domain::time::Timestamp;
use aep_service_app::{ServiceProvider, ServiceScope, TrustedRequestContext};
use aep_service_auth::CredentialVerifier;

/// Server-owned metadata supplied by the concrete HTTP listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestMetadata {
    /// The identity of this transport attempt.
    pub request_id: RequestId,
    /// The receiving server's clock value.
    pub received_at: Timestamp,
}

/// A runtime- and framework-neutral HTTP adapter for the supported AEP service wires.
pub struct AepHttpService<V, P> {
    verifier: V,
    services: P,
}

impl<V, P> AepHttpService<V, P> {
    /// Composes credential verification and trusted semantic service selection.
    pub const fn new(verifier: V, services: P) -> Self {
        Self { verifier, services }
    }

    /// Recovers the injected verifier and service provider.
    pub fn into_parts(self) -> (V, P) {
        (self.verifier, self.services)
    }
}

impl<V: CredentialVerifier, P: ServiceProvider> AepHttpService<V, P> {
    /// Handles one already-received HTTP exchange.
    ///
    /// Credential bytes are borrowed only while calling the verifier and are never retained.
    pub async fn handle(&self, request: Request, metadata: RequestMetadata) -> Response {
        let Some(version) = negotiated_version(&request) else {
            return unsupported_version();
        };
        let media_type = version.media_type();

        let (scope, route) = match Route::parse(request.method, &request.path) {
            Ok(route) => route,
            Err(reason) => {
                return problem(
                    metadata.request_id,
                    ProblemMappingV1::query(&QueryError::Invalid { reason }),
                    media_type,
                );
            }
        };

        let principal = match self
            .verifier
            .verify(header(&request, "Authorization"))
            .await
        {
            Ok(principal) => principal,
            Err(error) => {
                return problem(
                    metadata.request_id,
                    ProblemMappingV1::unauthenticated(error.reason()),
                    media_type,
                );
            }
        };
        if !principal.authorizes(scope.realm(), scope.workspace()) {
            return problem(
                metadata.request_id,
                ProblemMappingV1::unauthorized("workspace is not granted"),
                media_type,
            );
        }

        let trusted = TrustedRequestContext::new(
            principal,
            scope,
            metadata.request_id.clone(),
            metadata.received_at,
        );
        if !version.permits(&route) {
            return unsupported_version();
        }
        let intent = match Intent::decode(route, &request, &trusted) {
            Ok(intent) => intent,
            Err(mapping) => return problem(metadata.request_id, mapping, media_type),
        };
        let command_intent = matches!(&intent, Intent::Command(_));

        let service = match self.services.bind(&trusted).await {
            Ok(service) => service,
            Err(error) if command_intent => {
                return problem(
                    metadata.request_id,
                    ProblemMappingV1::command(&CommandError::Unavailable {
                        reason: error.reason().to_owned(),
                    }),
                    media_type,
                );
            }
            Err(error) => {
                return problem(
                    metadata.request_id,
                    ProblemMappingV1::query(&QueryError::Unavailable {
                        reason: error.reason().to_owned(),
                    }),
                    media_type,
                );
            }
        };

        dispatch_intent(service, intent, metadata.request_id, media_type).await
    }
}

async fn dispatch_intent<S>(
    service: S,
    intent: Intent,
    request_id: RequestId,
    media_type: &str,
) -> Response
where
    S: CommandService<Command = Command> + QueryService<AuditRecord = AuditRecord>,
{
    match intent {
        Intent::Command(envelope) => match service.execute(*envelope).await {
            Ok(result) => success(request_id, wire::CommandResultV1::from(result), media_type),
            Err(error) => problem(request_id, ProblemMappingV1::command(&error), media_type),
        },
        Intent::Get(reference, consistency) => match service.get(&reference, consistency).await {
            Ok(result) => success(request_id, result, media_type),
            Err(error) => problem(request_id, ProblemMappingV1::query(&error), media_type),
        },
        Intent::Resolve(locator) => match service.resolve(&locator).await {
            Ok(result) => success(request_id, result, media_type),
            Err(error) => problem(request_id, ProblemMappingV1::query(&error), media_type),
        },
        Intent::EntityQuery(query) => match service.query(&query).await {
            Ok(result) => success(request_id, wire::PageV1::from(result), media_type),
            Err(error) => problem(request_id, ProblemMappingV1::query(&error), media_type),
        },
        Intent::RelationQuery(query) => match service.relations(&query).await {
            Ok(result) => success(request_id, wire::PageV1::from(result), media_type),
            Err(error) => problem(request_id, ProblemMappingV1::query(&error), media_type),
        },
        Intent::History(reference) => match service.history(&reference).await {
            Ok(result) => success(request_id, result, media_type),
            Err(error) => problem(request_id, ProblemMappingV1::query(&error), media_type),
        },
        Intent::HistoryPage(mut query) => {
            query.limit.get_or_insert(100);
            match service.history_page(&query).await {
                Ok(result) => success(request_id, PageV2::from(result), media_type),
                Err(error) => problem(request_id, ProblemMappingV1::query(&error), media_type),
            }
        }
        Intent::Audit(query) => match service.audit(&query).await {
            Ok(result) => success(request_id, wire::PageV1::from(result), media_type),
            Err(error) => problem(request_id, ProblemMappingV1::query(&error), media_type),
        },
        Intent::DescribeType(entity_type) => match service.describe_type(&entity_type).await {
            Ok(result) => success(request_id, result, media_type),
            Err(error) => problem(request_id, ProblemMappingV1::query(&error), media_type),
        },
    }
}

#[derive(Debug)]
enum Route {
    Command,
    GetEntity(String),
    Resolve,
    EntityQuery,
    RelationQuery,
    History(String),
    HistoryQuery,
    Audit,
    DescribeType(String),
}

impl Route {
    fn parse(method: Method, path: &str) -> Result<(ServiceScope, Self), String> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() < 8 || parts[..4] != ["", "aep", "v1", "realms"] || parts[5] != "workspaces"
        {
            return Err("the request path is not a version-1 AEP route".to_owned());
        }
        let realm = decode_coordinate("realm", parts[4])?;
        let workspace = decode_coordinate("workspace", parts[6])?;
        let tail = &parts[7..];
        let route = match (method, tail) {
            (Method::Post, ["commands"]) => Self::Command,
            (Method::Get, ["entities", entity]) => Self::GetEntity(decode_segment(entity)?),
            (Method::Post, ["entities", "resolve"]) => Self::Resolve,
            (Method::Post, ["entities", "query"]) => Self::EntityQuery,
            (Method::Post, ["relations", "query"]) => Self::RelationQuery,
            (Method::Get, ["entities", entity, "history"]) => {
                Self::History(decode_segment(entity)?)
            }
            (Method::Post, ["history", "query"]) => Self::HistoryQuery,
            (Method::Post, ["audit", "query"]) => Self::Audit,
            (Method::Get, ["types", entity_type]) => {
                Self::DescribeType(decode_segment(entity_type)?)
            }
            _ => return Err("the method and path do not name a version-1 AEP operation".to_owned()),
        };
        Ok((ServiceScope::new(realm, workspace), route))
    }
}

enum Intent {
    Command(Box<CommandEnvelope<Command>>),
    Get(EntityRef, QueryConsistency),
    Resolve(aep_domain::entity::EntityLocator),
    EntityQuery(EntityQuery),
    RelationQuery(RelationQuery),
    History(EntityRef),
    HistoryPage(HistoryQuery),
    Audit(AuditQuery),
    DescribeType(EntityType),
}

impl Intent {
    fn decode(
        route: Route,
        request: &Request,
        trusted: &TrustedRequestContext,
    ) -> Result<Self, ProblemMappingV1> {
        match route {
            Route::Command => decode_command(&request.body, trusted)
                .map(Box::new)
                .map(Self::Command),
            Route::GetEntity(raw) => {
                let reference = entity_reference(&raw)?;
                let consistency = consistency(request)?;
                Ok(Self::Get(reference, consistency))
            }
            Route::Resolve => decode_query_body::<ResolveRequestV1>(&request.body)
                .map(|request| Self::Resolve(request.locator)),
            Route::EntityQuery => decode_query_body::<EntityQueryV1>(&request.body)
                .map(EntityQuery::from)
                .map(Self::EntityQuery),
            Route::RelationQuery => decode_query_body::<RelationQueryV1>(&request.body)
                .map(Into::into)
                .map(Self::RelationQuery),
            Route::History(raw) => entity_reference(&raw).map(Self::History),
            Route::HistoryQuery => decode_query_body::<HistoryQueryV2>(&request.body)
                .map(HistoryQuery::from)
                .map(Self::HistoryPage),
            Route::Audit => decode_query_body::<AuditQueryV1>(&request.body)
                .map(AuditQuery::from)
                .map(Self::Audit),
            Route::DescribeType(raw) => raw.parse().map(Self::DescribeType).map_err(|error| {
                ProblemMappingV1::query(&QueryError::Invalid {
                    reason: format!("invalid entity type in path: {error}"),
                })
            }),
        }
    }
}

fn decode_command(
    body: &[u8],
    trusted: &TrustedRequestContext,
) -> Result<CommandEnvelope<Command>, ProblemMappingV1> {
    let request: CommandRequestV1 =
        wire::decode(body).map_err(|error| malformed_command(&error))?;
    let payload = request
        .decode_command()
        .map_err(|error| malformed_command(&error))?;
    let context = trusted.command_context(
        request.idempotency_key,
        request.correlation_id,
        request.causation.into_option(),
        request.execution_id.into_option(),
        request.task.into_option(),
    );
    let mut envelope =
        CommandEnvelope::new(request.command_id, request.command_type, payload, context);
    envelope.target = request.target.into_option();
    envelope.expected_revision = request.expected_revision.into_option();
    Ok(envelope)
}

fn malformed_command(error: &wire::DocumentError) -> ProblemMappingV1 {
    let rendered = error.to_string();
    let location = missing_member(&rendered).unwrap_or_else(|| "$".to_owned());
    let message = if location == "$" {
        "the document does not match the version-1 command shape"
    } else {
        "required member is missing"
    };
    let errors = ValidationErrors::from(ValidationError::new(
        ValidationCode::TypeMismatch,
        location,
        message,
    ));
    let mut mapping = ProblemMappingV1::command(&CommandError::Invalid { errors });
    "the request is malformed".clone_into(&mut mapping.problem.message);
    mapping
}

fn missing_member(message: &str) -> Option<String> {
    let marker = "missing field `";
    let start = message.find(marker)? + marker.len();
    let remainder = &message[start..];
    let end = remainder.find('`')?;
    Some(remainder[..end].to_owned())
}

fn decode_query_body<T: for<'de> serde::Deserialize<'de>>(
    body: &[u8],
) -> Result<T, ProblemMappingV1> {
    wire::decode(body).map_err(|error| {
        ProblemMappingV1::query(&QueryError::Invalid {
            reason: error.to_string(),
        })
    })
}

fn entity_reference(raw: &str) -> Result<EntityRef, ProblemMappingV1> {
    EntityId::new(raw).map(EntityRef::new).map_err(|error| {
        ProblemMappingV1::query(&QueryError::Invalid {
            reason: format!("invalid entity id in path: {error}"),
        })
    })
}

fn consistency(request: &Request) -> Result<QueryConsistency, ProblemMappingV1> {
    header(request, CONSISTENCY_HEADER).map_or_else(
        || Ok(QueryConsistency::default()),
        |raw| {
            ConsistencyToken::new(raw)
                .map(QueryConsistency::at_least)
                .map_err(|error| {
                    ProblemMappingV1::query(&QueryError::Invalid {
                        reason: format!("invalid consistency token: {error}"),
                    })
                })
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WireVersion {
    V1,
    V2,
}

impl WireVersion {
    const fn media_type(self) -> &'static str {
        match self {
            Self::V1 => MEDIA_TYPE_V1,
            Self::V2 => MEDIA_TYPE_V2,
        }
    }

    const fn permits(self, route: &Route) -> bool {
        matches!((self, route), (Self::V2, Route::HistoryQuery))
            || matches!(self, Self::V1) && !matches!(route, Route::HistoryQuery)
    }
}

fn negotiated_version(request: &Request) -> Option<WireVersion> {
    let version = header(request, "Accept").and_then(|value| {
        value
            .split(',')
            .find_map(|candidate| match candidate.trim() {
                MEDIA_TYPE_V2 => Some(WireVersion::V2),
                MEDIA_TYPE_V1 => Some(WireVersion::V1),
                _ => None,
            })
    })?;
    let content_type = if request.method == Method::Post {
        header(request, "Content-Type") == Some(version.media_type())
    } else {
        request.body.is_empty()
    };
    content_type.then_some(version)
}

fn header<'a>(request: &'a Request, name: &str) -> Option<&'a str> {
    request
        .headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn decode_coordinate(kind: &'static str, raw: &str) -> Result<String, String> {
    let value = decode_segment(raw)?;
    if value.is_empty() {
        return Err(format!("the {kind} must not be empty"));
    }
    if value.len() > 200 {
        return Err(format!("the {kind} must be at most 200 bytes"));
    }
    if value.chars().any(char::is_control) {
        return Err(format!("the {kind} must not contain control characters"));
    }
    Ok(value)
}

fn decode_segment(raw: &str) -> Result<String, String> {
    let bytes = raw.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err("a path segment contains an incomplete percent escape".to_owned());
            }
            let high = hex(bytes[index + 1])?;
            let low = hex(bytes[index + 2])?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).map_err(|_| "a path segment is not valid UTF-8".to_owned())
}

fn hex(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("a path segment contains an invalid percent escape".to_owned()),
    }
}

fn response_headers(media_type: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("Content-Type".to_owned(), media_type.to_owned()),
        ("Vary".to_owned(), "Accept".to_owned()),
    ])
}

fn success<T: serde::Serialize>(request_id: RequestId, result: T, media_type: &str) -> Response {
    encoded_response(200, &SuccessV1 { request_id, result }, media_type)
}

fn problem(request_id: RequestId, mapping: ProblemMappingV1, media_type: &str) -> Response {
    encoded_response(
        mapping.status,
        &ProblemDocumentV1 {
            request_id,
            error: mapping.problem,
        },
        media_type,
    )
}

fn encoded_response<T: serde::Serialize>(status: u16, document: &T, media_type: &str) -> Response {
    match wire::encode(document) {
        Ok(body) => Response {
            status,
            headers: response_headers(media_type),
            body,
        },
        Err(_) => Response {
            status: 500,
            headers: BTreeMap::from([("Vary".to_owned(), "Accept".to_owned())]),
            body: Vec::new(),
        },
    }
}

fn unsupported_version() -> Response {
    Response {
        status: 406,
        headers: BTreeMap::from([
            (SUPPORTED_VERSIONS_HEADER.to_owned(), "1, 2".to_owned()),
            ("Vary".to_owned(), "Accept".to_owned()),
        ]),
        body: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_decoding_is_strict_and_preserves_opaque_utf8_coordinates() {
        assert_eq!(
            decode_segment("company%20brain"),
            Ok("company brain".to_owned())
        );
        assert_eq!(
            decode_segment("aep.design%2Fv1"),
            Ok("aep.design/v1".to_owned())
        );
        assert!(decode_segment("bad%2").is_err());
        assert!(decode_segment("bad%XZ").is_err());
        assert!(decode_segment("%FF").is_err());
    }

    #[test]
    fn only_the_declared_methods_and_routes_are_recognized() {
        let base = "/aep/v1/realms/company/workspaces/repo";
        let entity = "01K2R8JD3ZJME72AJGQY67E5F8";
        assert!(matches!(
            Route::parse(Method::Post, &format!("{base}/commands")),
            Ok((_, Route::Command))
        ));
        assert!(matches!(
            Route::parse(Method::Get, &format!("{base}/entities/{entity}")),
            Ok((_, Route::GetEntity(value))) if value == entity
        ));
        assert!(matches!(
            Route::parse(Method::Post, &format!("{base}/entities/resolve")),
            Ok((_, Route::Resolve))
        ));
        assert!(matches!(
            Route::parse(Method::Post, &format!("{base}/entities/query")),
            Ok((_, Route::EntityQuery))
        ));
        assert!(matches!(
            Route::parse(Method::Post, &format!("{base}/relations/query")),
            Ok((_, Route::RelationQuery))
        ));
        assert!(matches!(
            Route::parse(
                Method::Get,
                &format!("{base}/entities/{entity}/history")
            ),
            Ok((_, Route::History(value))) if value == entity
        ));
        assert!(matches!(
            Route::parse(Method::Post, &format!("{base}/audit/query")),
            Ok((_, Route::Audit))
        ));
        assert!(matches!(
            Route::parse(Method::Get, &format!("{base}/types/aep.design%2Fv1")),
            Ok((_, Route::DescribeType(value))) if value == "aep.design/v1"
        ));
        assert!(Route::parse(Method::Get, &format!("{base}/commands")).is_err());
        assert!(Route::parse(Method::Post, &format!("{base}/raw-store")).is_err());
    }
}
