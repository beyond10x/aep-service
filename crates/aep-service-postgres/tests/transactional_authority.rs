//! PostgreSQL evidence for a fresh service handle, a torn command and two competing writers.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use aep_contract::command::{CommandContext, CommandEnvelope, CommandService};
use aep_contract::error::CommandError;
use aep_contract::query::{AuditQuery, EntityQuery, HistoryQuery, QueryService, RelationQuery};
use aep_contract::testing::block_on;
use aep_contract::QueryConsistency;
use aep_domain::artifact::{LifecycleRegistry, RelationKind};
use aep_domain::command::{Command, CreateEntity, CreateRelation, UpdateEntity};
use aep_domain::entity::{
    ActorRef, EntityId, EntityLocator, EntityRef, EntityRevision, EntityType,
};
use aep_domain::node::Node;
use aep_domain::time::Timestamp;
use aep_service_app::{ServiceProvider, ServiceScope, TrustedRequestContext};
use aep_service_auth::VerifiedPrincipal;
use aep_service_postgres::PostgresAuthority;
use postgres::{Client, NoTls};

static SCHEMAS: AtomicUsize = AtomicUsize::new(0);

fn url() -> Option<String> {
    match std::env::var("ENTITY_POSTGRES_URL") {
        Ok(url) if !url.trim().is_empty() => Some(url),
        _ => {
            eprintln!(
                "skipped: ENTITY_POSTGRES_URL unset, so PostgreSQL transaction evidence did not run"
            );
            None
        }
    }
}

fn schema(label: &str) -> String {
    format!(
        "aep_service_{}_{}_{label}",
        std::process::id(),
        SCHEMAS.fetch_add(1, Ordering::Relaxed)
    )
}

struct TestSchema {
    url: String,
    name: String,
}

impl TestSchema {
    fn new(url: &str, label: &str) -> Self {
        Self {
            url: url.to_owned(),
            name: schema(label),
        }
    }
}

impl Drop for TestSchema {
    fn drop(&mut self) {
        let Ok(mut client) = Client::connect(&self.url, NoTls) else {
            return;
        };
        let _ = client.batch_execute(&format!("DROP SCHEMA IF EXISTS {} CASCADE", self.name));
    }
}

fn authority(url: &str, schema: &str) -> PostgresAuthority {
    PostgresAuthority::new(
        url,
        "company-planning",
        "aep-service",
        schema,
        LifecycleRegistry::new(),
    )
    .expect("valid authority configuration")
}

fn trusted() -> TrustedRequestContext {
    TrustedRequestContext::new(
        VerifiedPrincipal::new(
            "human:alice".parse().expect("authority"),
            None,
            "company-planning",
            ["aep-service".to_owned()],
            ["engineer".to_owned()],
            None,
        ),
        ServiceScope::new("company-planning", "aep-service"),
        "server-request".parse().expect("request id"),
        Timestamp::from_epoch_millis(1_800_000_000_000),
    )
}

fn envelope(command: Command, n: u32, actor: &str) -> CommandEnvelope<Command> {
    let kind = command.kind().as_str();
    CommandEnvelope::new(
        format!("cmd-{n}-{actor}").parse().expect("command id"),
        kind,
        command,
        CommandContext::new(
            format!("req-{n}-{actor}").parse().expect("request id"),
            format!("key-{n}-{actor}").parse().expect("idempotency key"),
            ActorRef::parse(&format!("human:{actor}")).expect("actor"),
            "corr-transaction".parse().expect("correlation id"),
            Timestamp::from_epoch_millis(1_800_000_000_000 + u64::from(n)),
        ),
    )
}

fn create(name: &str) -> Command {
    Command::CreateEntity(CreateEntity {
        entity_type: EntityType::parse("aep.story/v1").expect("entity type"),
        locator: EntityLocator::parse(&format!("ep://beyond10x/plan/story/{name}"))
            .expect("locator"),
        data: Node::Map(BTreeMap::from([
            ("status".to_owned(), Node::from("draft")),
            ("title".to_owned(), Node::from(name)),
        ])),
    })
}

fn retitle(target: &EntityId, title: &str) -> Command {
    Command::UpdateEntity(UpdateEntity {
        target: EntityRef::new(target.clone()),
        changes: BTreeMap::from([("title".to_owned(), Node::from(title))]),
    })
}

fn database(url: &str, schema: &str) -> Client {
    let mut client = Client::connect(url, NoTls)
        .unwrap_or_else(|error| panic!("ENTITY_POSTGRES_URL is set but refused: {error}"));
    client
        .batch_execute(&format!("SET search_path TO {schema}"))
        .expect("test schema selected");
    client
}

fn count(client: &mut Client, table: &str, entity: Option<&str>) -> i64 {
    match entity {
        Some(entity) => client
            .query_one(
                &format!("SELECT COUNT(*) FROM {table} WHERE entity = $1"),
                &[&entity],
            )
            .expect("counted records")
            .get(0),
        None => client
            .query_one(&format!("SELECT COUNT(*) FROM {table}"), &[])
            .expect("counted records")
            .get(0),
    }
}

#[test]
fn a_failure_after_state_relation_event_and_audit_writes_rolls_the_whole_command_back() {
    let Some(url) = url() else { return };
    let test_schema = TestSchema::new(&url, "rollback");
    let authority = authority(&url, &test_schema.name);
    let service = block_on(authority.bind(&trusted())).expect("authority opens");

    let source = block_on(service.execute(envelope(create("source"), 1, "alice")))
        .expect("source created")
        .affected[0]
        .id
        .clone();
    let target = block_on(service.execute(envelope(create("target"), 2, "alice")))
        .expect("target created")
        .affected[0]
        .id
        .clone();

    let mut client = database(&url, &test_schema.name);
    client
        .batch_execute(
            "CREATE FUNCTION fail_applied_command() RETURNS trigger LANGUAGE plpgsql AS $$
                 BEGIN RAISE EXCEPTION 'injected applied-command failure'; END
             $$;
             CREATE TRIGGER fail_applied_command
             BEFORE INSERT ON instances
             FOR EACH ROW WHEN (NEW.entity = 'aep.applied')
             EXECUTE FUNCTION fail_applied_command();",
        )
        .expect("failure injected");

    let before = [
        count(&mut client, "instances", Some("aep.entity")),
        count(&mut client, "instances", Some("aep.relation")),
        count(&mut client, "instances", Some("aep.audit")),
        count(&mut client, "instances", Some("aep.applied")),
        count(&mut client, "events", None),
    ];
    let relation = Command::CreateRelation(CreateRelation {
        kind: RelationKind::Decomposes,
        source: EntityRef::new(source),
        target: EntityRef::new(target),
    });
    let error = block_on(service.execute(envelope(relation, 3, "alice")))
        .expect_err("injected final-record failure refuses the command");
    assert!(
        matches!(error, CommandError::Conflict { ref reason }
            if reason.contains("database refused the command session")
                && reason.contains("writing the instance")),
        "the provider failure remains a named command refusal: {error}"
    );

    let after = [
        count(&mut client, "instances", Some("aep.entity")),
        count(&mut client, "instances", Some("aep.relation")),
        count(&mut client, "instances", Some("aep.audit")),
        count(&mut client, "instances", Some("aep.applied")),
        count(&mut client, "events", None),
    ];
    assert_eq!(after, before, "no prefix of the failed command is visible");
    client
        .batch_execute("DROP TRIGGER fail_applied_command ON instances")
        .expect("failure trigger removed");
    let reopened = block_on(authority.bind(&trusted())).expect("fresh authority opens");
    let relations = block_on(reopened.relations(&RelationQuery::default())).expect("relations");
    assert!(relations.items.is_empty(), "the failed relation is absent");
}

#[test]
fn two_fresh_service_handles_cannot_silently_overwrite_one_revision() {
    let Some(url) = url() else { return };
    let test_schema = TestSchema::new(&url, "race");
    let authority = authority(&url, &test_schema.name);
    let creator = block_on(authority.bind(&trusted())).expect("authority opens");
    let id = block_on(creator.execute(envelope(create("contested"), 1, "alice")))
        .expect("entity created")
        .affected[0]
        .id
        .clone();

    let first = block_on(authority.bind(&trusted())).expect("first fresh handle");
    let second = block_on(authority.bind(&trusted())).expect("second fresh handle");
    let (a, b) = std::thread::scope(|scope| {
        let a = scope.spawn(|| {
            block_on(
                first.execute(
                    envelope(retitle(&id, "A won"), 2, "alice")
                        .expecting(EntityRevision::new(1).expect("created revision")),
                ),
            )
        });
        let b = scope.spawn(|| {
            block_on(
                second.execute(
                    envelope(retitle(&id, "B won"), 2, "bob")
                        .expecting(EntityRevision::new(1).expect("created revision")),
                ),
            )
        });
        (a.join().expect("writer a"), b.join().expect("writer b"))
    });
    let outcomes = [a, b];
    assert_eq!(
        outcomes.iter().filter(|outcome| outcome.is_ok()).count(),
        1,
        "exactly one stale candidate may commit: {outcomes:?}"
    );
    let refusal = outcomes
        .iter()
        .find_map(|outcome| outcome.as_ref().err())
        .expect("one writer refused");
    assert!(
        matches!(
            refusal,
            CommandError::RevisionConflict {
                expected,
                actual,
                ..
            } if expected.get() == 1 && actual.get() == 2
        ),
        "the loser is told which durable revision won: {refusal}"
    );

    let reader = block_on(authority.bind(&trusted())).expect("fresh reader");
    let held = block_on(reader.get(&EntityRef::new(id), QueryConsistency::Current))
        .expect("winner visible");
    assert_eq!(held.metadata.revision.get(), 2, "one update landed");
}

#[test]
fn indexed_queries_page_current_rows_history_relations_and_audit_without_realm_hydration() {
    let Some(url) = url() else { return };
    let test_schema = TestSchema::new(&url, "indexed");
    let authority = authority(&url, &test_schema.name);
    let service = block_on(authority.bind(&trusted())).expect("authority opens");

    let first = block_on(service.execute(envelope(create("first"), 1, "alice")))
        .expect("first entity created");
    let first_id = first.affected[0].id.clone();
    let second_id = block_on(service.execute(envelope(create("second"), 2, "alice")))
        .expect("second entity created")
        .affected[0]
        .id
        .clone();
    block_on(service.execute(envelope(retitle(&first_id, "first revised"), 3, "alice")))
        .expect("first entity revised");
    block_on(service.execute(envelope(
        Command::CreateRelation(CreateRelation {
            kind: RelationKind::Decomposes,
            source: EntityRef::new(first_id.clone()),
            target: EntityRef::new(second_id),
        }),
        4,
        "alice",
    )))
    .expect("relation created");

    let held = block_on(service.get(
        &EntityRef::new(first_id.clone()),
        QueryConsistency::at_least(first.consistency),
    ))
    .expect("the command token is already visible on the primary authority");
    assert_eq!(held.metadata.revision.get(), 2);

    let mut entities = EntityQuery::of_type(EntityType::parse("aep.story/v1").expect("type"));
    entities.limit = Some(1);
    let first_page = block_on(service.query(&entities)).expect("first entity page");
    assert_eq!(first_page.items.len(), 1);
    entities.after = first_page.next;
    let second_page = block_on(service.query(&entities)).expect("second entity page");
    assert_eq!(second_page.items.len(), 1);
    assert_ne!(
        first_page.items[0].metadata.id,
        second_page.items[0].metadata.id
    );

    let relations =
        block_on(service.relations(&RelationQuery::from(EntityRef::new(first_id.clone()))))
            .expect("indexed outgoing relations");
    assert_eq!(relations.items.len(), 1);

    let mut history = HistoryQuery::for_entity(EntityRef::new(first_id.clone()));
    history.limit = Some(1);
    let revisions = block_on(service.history_page(&history)).expect("first history page");
    assert_eq!(revisions.items.len(), 1);
    assert!(revisions.next.is_some(), "the update is on the next page");

    let audit = block_on(service.audit(&AuditQuery::for_entity(EntityRef::new(first_id))))
        .expect("indexed audit records");
    assert!(!audit.items.is_empty());
}
