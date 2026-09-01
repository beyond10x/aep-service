//! Service-side verification of the AEP-owned constructed wire corpus.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::future::{ready, Future};
use std::rc::Rc;

use aep_client::conformance::{Dispatch, Principal, VerifierOutcome, CASES};
use aep_client::wire::{self, Request};
use aep_contract::command::{CommandEnvelope, CommandOutcome, CommandResult, CommandService};
use aep_contract::error::{CommandError, QueryError};
use aep_contract::query::{
    AuditQuery, EntityEnvelope, EntityQuery, HistoryQuery, Page, QueryService, Relation,
    RelationQuery, RevisionRecord,
};
use aep_contract::registry::TypeDescriptor;
use aep_contract::{ConsistencyToken, QueryConsistency};
use aep_domain::audit::AuditRecord;
use aep_domain::command::Command;
use aep_domain::entity::{EntityId, EntityLocator, EntityRef, EntityType};
use aep_service_app::{ServiceBindingError, ServiceProvider, TrustedRequestContext};
use aep_service_auth::{AuthenticationError, CredentialVerifier, VerifiedPrincipal};
use aep_service_http::{AepHttpService, RequestMetadata};

#[derive(Clone, Copy)]
struct CorpusVerifier {
    outcome: VerifierOutcome,
}

impl CredentialVerifier for CorpusVerifier {
    fn verify(
        &self,
        authorization: Option<&str>,
    ) -> impl Future<Output = Result<VerifiedPrincipal, AuthenticationError>> {
        let result = match (authorization, self.outcome) {
            (None, _) | (_, VerifierOutcome::Unauthenticated) => {
                Err(AuthenticationError::new("credential missing"))
            }
            (Some(_), VerifierOutcome::Verified(principal)) => Ok(verified(principal)),
        };
        ready(result)
    }
}

fn verified(principal: Principal) -> VerifiedPrincipal {
    VerifiedPrincipal::new(
        principal.authority.parse().expect("corpus authority"),
        principal
            .executor
            .map(|executor| executor.parse().expect("corpus executor")),
        principal.realm,
        principal
            .workspace_grants
            .iter()
            .map(|grant| (*grant).to_owned()),
        principal.roles.iter().map(|role| (*role).to_owned()),
        principal.delegation_id.map(str::to_owned),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Accepted,
    Replayed,
    SemanticRefusal,
    RevisionConflict,
    Unavailable,
    Query,
    NoDispatch,
}

impl Mode {
    fn for_case(name: &str) -> Self {
        match name {
            "accepted-human-command" => Self::Accepted,
            "replayed-delegated-command" => Self::Replayed,
            "semantic-command-refusal" => Self::SemanticRefusal,
            "revision-conflict" => Self::RevisionConflict,
            "service-unavailable" => Self::Unavailable,
            "authorized-entity-query" => Self::Query,
            _ => Self::NoDispatch,
        }
    }
}

#[derive(Default)]
struct Observed {
    bindings: Vec<TrustedRequestContext>,
    command_calls: usize,
    entity_query_calls: usize,
    history_page_calls: usize,
    commands: Vec<CommandEnvelope<Command>>,
}

#[derive(Clone)]
struct FakeService {
    mode: Mode,
    observed: Rc<RefCell<Observed>>,
}

struct FakeProvider {
    service: FakeService,
}

impl ServiceProvider for FakeProvider {
    type Service = FakeService;

    fn bind(
        &self,
        context: &TrustedRequestContext,
    ) -> impl Future<Output = Result<Self::Service, ServiceBindingError>> {
        self.service
            .observed
            .borrow_mut()
            .bindings
            .push(context.clone());
        ready(Ok(self.service.clone()))
    }
}

impl CommandService for FakeService {
    type Command = Command;

    fn execute(
        &self,
        command: CommandEnvelope<Self::Command>,
    ) -> impl Future<Output = Result<CommandResult, CommandError>> {
        let mut observed = self.observed.borrow_mut();
        observed.command_calls += 1;
        observed.commands.push(command.clone());
        let result = match self.mode {
            Mode::Accepted | Mode::Replayed => Ok(CommandResult {
                command_id: command.command_id,
                outcome: if self.mode == Mode::Accepted {
                    CommandOutcome::Accepted
                } else {
                    CommandOutcome::Replayed
                },
                affected: Vec::new(),
                events: Vec::new(),
                audit: Vec::new(),
                consistency: ConsistencyToken::new("seq:1").expect("corpus consistency"),
            }),
            Mode::SemanticRefusal => Err(CommandError::Conflict {
                reason: "story is already implemented".to_owned(),
            }),
            Mode::RevisionConflict => Err(CommandError::RevisionConflict {
                entity: command.target.expect("corpus target"),
                expected: command.expected_revision.expect("corpus expected revision"),
                actual: aep_domain::entity::EntityRevision::new(8).expect("actual revision"),
            }),
            Mode::Unavailable => Err(CommandError::Unavailable {
                reason: "database unavailable".to_owned(),
            }),
            Mode::Query | Mode::NoDispatch => Err(CommandError::Unavailable {
                reason: "unexpected corpus command dispatch".to_owned(),
            }),
        };
        ready(result)
    }
}

impl QueryService for FakeService {
    type AuditRecord = AuditRecord;

    fn get(
        &self,
        _reference: &EntityRef,
        _consistency: QueryConsistency,
    ) -> impl Future<Output = Result<EntityEnvelope, QueryError>> {
        ready(unexpected_query("get"))
    }

    fn resolve(
        &self,
        _locator: &EntityLocator,
    ) -> impl Future<Output = Result<EntityId, QueryError>> {
        ready(unexpected_query("resolve"))
    }

    fn query(
        &self,
        _query: &EntityQuery,
    ) -> impl Future<Output = Result<Page<EntityEnvelope>, QueryError>> {
        self.observed.borrow_mut().entity_query_calls += 1;
        if self.mode == Mode::Query {
            ready(Ok(Page::complete(Vec::new())))
        } else {
            ready(unexpected_query("entity query"))
        }
    }

    fn relations(
        &self,
        _query: &RelationQuery,
    ) -> impl Future<Output = Result<Page<Relation>, QueryError>> {
        ready(unexpected_query("relation query"))
    }

    fn history(
        &self,
        _reference: &EntityRef,
    ) -> impl Future<Output = Result<Vec<RevisionRecord>, QueryError>> {
        ready(unexpected_query("history"))
    }

    fn history_page(
        &self,
        _query: &HistoryQuery,
    ) -> impl Future<Output = Result<Page<RevisionRecord>, QueryError>> {
        self.observed.borrow_mut().history_page_calls += 1;
        ready(Ok(Page::complete(Vec::new())))
    }

    fn audit(
        &self,
        _query: &AuditQuery,
    ) -> impl Future<Output = Result<Page<Self::AuditRecord>, QueryError>> {
        ready(unexpected_query("audit"))
    }

    fn describe_type(
        &self,
        _entity_type: &EntityType,
    ) -> impl Future<Output = Result<TypeDescriptor, QueryError>> {
        ready(unexpected_query("type description"))
    }
}

fn unexpected_query<T>(operation: &str) -> Result<T, QueryError> {
    Err(QueryError::Unavailable {
        reason: format!("unexpected corpus {operation} dispatch"),
    })
}

fn request_id(name: &str) -> &'static str {
    match name {
        "accepted-human-command" => "server-request-accepted",
        "replayed-delegated-command" => "server-request-replayed",
        "semantic-command-refusal" => "server-request-refused",
        "revision-conflict" => "server-request-conflict",
        "malformed-command" => "server-request-malformed",
        "service-unavailable" => "server-request-unavailable",
        "unauthenticated-command" => "server-request-unauthenticated",
        "workspace-unauthorized-command" => "server-request-unauthorized",
        "authorized-entity-query" => "server-request-query",
        "workspace-unauthorized-query" => "server-request-query-denied",
        "unsupported-wire-version" => "server-request-version",
        _ => panic!("unknown corpus case {name}"),
    }
}

fn supported_versions<'a>(name: &str, corpus: Option<&'a str>) -> Option<&'a str> {
    if name == "unsupported-wire-version" {
        Some("1, 2")
    } else {
        corpus
    }
}

#[test]
fn the_service_answers_every_ep_owned_exchange_byte_for_byte_and_dispatches_only_when_declared() {
    for case in CASES {
        let observed = Rc::new(RefCell::new(Observed::default()));
        let service = FakeService {
            mode: Mode::for_case(case.name),
            observed: Rc::clone(&observed),
        };
        let adapter = AepHttpService::new(
            CorpusVerifier {
                outcome: case.verifier,
            },
            FakeProvider { service },
        );
        let mut headers = BTreeMap::from([("Accept".to_owned(), case.request.accept.to_owned())]);
        if let Some(content_type) = case.request.content_type {
            headers.insert("Content-Type".to_owned(), content_type.to_owned());
        }
        if case.request.credential_present {
            headers.insert("Authorization".to_owned(), "Bearer synthetic".to_owned());
        }
        let response = aep_contract::testing::block_on(adapter.handle(
            Request {
                method: case.request.method,
                path: case.request.path.to_owned(),
                headers,
                body: case.request.body.to_vec(),
            },
            RequestMetadata {
                request_id: request_id(case.name).parse().expect("request id"),
                received_at: aep_domain::time::Timestamp::from_epoch_millis(1_800_000_000_000),
            },
        ));

        assert_eq!(
            response.status, case.response.status,
            "{} status",
            case.name
        );
        assert_eq!(
            response.header("Content-Type"),
            case.response.content_type,
            "{} content type",
            case.name
        );
        assert_eq!(
            response.header(wire::SUPPORTED_VERSIONS_HEADER),
            supported_versions(case.name, case.response.supported_versions),
            "{} supported versions",
            case.name
        );
        assert_eq!(
            response.header("Vary"),
            Some("Accept"),
            "{} negotiation cache key",
            case.name
        );
        assert_eq!(response.body, case.response.body, "{} body", case.name);

        let observed = observed.borrow();
        let expected_commands = usize::from(case.dispatch == Dispatch::Command);
        let expected_queries = usize::from(case.dispatch == Dispatch::EntityQuery);
        assert_eq!(
            observed.command_calls, expected_commands,
            "{} command dispatch",
            case.name
        );
        assert_eq!(
            observed.entity_query_calls, expected_queries,
            "{} query dispatch",
            case.name
        );
        assert_eq!(
            observed.bindings.len(),
            expected_commands + expected_queries,
            "{} trusted service binding",
            case.name
        );

        if let Some(command) = observed.commands.first() {
            assert_eq!(
                command.context.actor.to_string(),
                "human:alice",
                "{}",
                case.name
            );
            let executor = command.context.executor.as_ref().map(ToString::to_string);
            if case.name == "replayed-delegated-command" {
                assert_eq!(executor.as_deref(), Some("agent:planner"));
            } else {
                assert_eq!(executor, None);
            }
            assert_eq!(
                command.context.request_id.to_string(),
                request_id(case.name)
            );
            assert_eq!(
                command.context.issued_at,
                aep_domain::time::Timestamp::from_epoch_millis(1_800_000_000_000)
            );
        }
    }
}

#[test]
fn bounded_history_version_two_selects_its_strict_documents_and_dispatches_one_page() {
    let observed = Rc::new(RefCell::new(Observed::default()));
    let adapter = AepHttpService::new(
        CorpusVerifier {
            outcome: VerifierOutcome::Verified(Principal {
                authority: "human:alice",
                executor: None,
                realm: "company",
                workspace_grants: &["repo"],
                roles: &["engineer"],
                delegation_id: None,
            }),
        },
        FakeProvider {
            service: FakeService {
                mode: Mode::Query,
                observed: Rc::clone(&observed),
            },
        },
    );
    let query = wire::HistoryQueryV2 {
        entity: EntityRef::new("01HISTORY00000000000000001".parse().expect("entity")),
        limit: wire::Nullable::new(Some(25)),
        after: wire::Nullable::new(None),
        consistency: QueryConsistency::Current,
    };
    let response = aep_contract::testing::block_on(adapter.handle(
        Request {
            method: wire::Method::Post,
            path: "/aep/v1/realms/company/workspaces/repo/history/query".to_owned(),
            headers: BTreeMap::from([
                ("Accept".to_owned(), wire::MEDIA_TYPE_V2.to_owned()),
                ("Content-Type".to_owned(), wire::MEDIA_TYPE_V2.to_owned()),
                ("Authorization".to_owned(), "Bearer synthetic".to_owned()),
            ]),
            body: wire::encode(&query).expect("request bytes"),
        },
        RequestMetadata {
            request_id: "server-request-history".parse().expect("request id"),
            received_at: aep_domain::time::Timestamp::from_epoch_millis(1_800_000_000_000),
        },
    ));

    assert_eq!(response.status, 200);
    assert_eq!(response.header("Content-Type"), Some(wire::MEDIA_TYPE_V2));
    let expected = wire::SuccessV1 {
        request_id: "server-request-history".parse().expect("request id"),
        result: wire::PageV2::<RevisionRecord> {
            items: Vec::new(),
            next: wire::Nullable::new(None),
        },
    };
    assert_eq!(
        response.body,
        wire::encode(&expected).expect("response bytes")
    );
    assert_eq!(observed.borrow().history_page_calls, 1);
}

#[test]
fn workspace_authorization_precedes_body_decoding_and_service_binding() {
    let case = CASES
        .iter()
        .find(|case| case.name == "workspace-unauthorized-command")
        .expect("unauthorized corpus principal");
    let observed = Rc::new(RefCell::new(Observed::default()));
    let adapter = AepHttpService::new(
        CorpusVerifier {
            outcome: case.verifier,
        },
        FakeProvider {
            service: FakeService {
                mode: Mode::NoDispatch,
                observed: Rc::clone(&observed),
            },
        },
    );
    let response = aep_contract::testing::block_on(adapter.handle(
        Request {
            method: case.request.method,
            path: case.request.path.to_owned(),
            headers: BTreeMap::from([
                ("Accept".to_owned(), case.request.accept.to_owned()),
                (
                    "Content-Type".to_owned(),
                    case.request.content_type.expect("content type").to_owned(),
                ),
                ("Authorization".to_owned(), "Bearer synthetic".to_owned()),
            ]),
            body: b"not a JSON document".to_vec(),
        },
        RequestMetadata {
            request_id: "server-request-order".parse().expect("request id"),
            received_at: aep_domain::time::Timestamp::from_epoch_millis(1_800_000_000_000),
        },
    ));

    assert_eq!(
        response.status, 403,
        "authorization wins over malformed input"
    );
    assert!(observed.borrow().bindings.is_empty());
}
