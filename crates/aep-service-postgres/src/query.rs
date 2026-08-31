//! Indexed, request-scoped reads over the Entity Runtime PostgreSQL provider.

use std::sync::Mutex;

use aep_backend_entity::{
    EntityBackend, Identity, AUDIT_AS, METADATA_KEY, RELATIONS_AS, STORED_AS,
};
use aep_contract::error::QueryError;
use aep_contract::query::{
    AuditQuery, Cursor, EntityEnvelope, EntityQuery, HistoryQuery, Page, QueryService, Relation,
    RelationQuery, RevisionRecord,
};
use aep_contract::registry::TypeDescriptor;
use aep_contract::testing::block_on;
use aep_contract::QueryConsistency;
use aep_domain::artifact::LifecycleRegistry;
use aep_domain::audit::AuditRecord;
use aep_domain::entity::{EntityId, EntityLocator, EntityRef, EntityType};
use entity_core::{Decision, EntityInstance};
use entity_postgres::PostgresStore;
use entity_query::{DocumentPage, DocumentQuery, DocumentQueryProvider, QueryCursor};
use entity_store::{EventProvider, Expect, MemoryStore, StateProvider, Store};
use serde::Serialize;
use serde_json::{json, Value};

const DEFAULT_PAGE: usize = 100;

/// A query handle that never enumerates a complete provider entity type.
pub(crate) struct IndexedPostgresQueries {
    store: Mutex<PostgresStore>,
    lifecycles: LifecycleRegistry,
}

impl std::fmt::Debug for IndexedPostgresQueries {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("IndexedPostgresQueries")
            .field("lifecycles", &self.lifecycles.len())
            .finish_non_exhaustive()
    }
}

impl IndexedPostgresQueries {
    pub(crate) fn connect_in_schema(
        url: &str,
        schema: &str,
        lifecycles: LifecycleRegistry,
    ) -> Result<Self, entity_store::StoreError> {
        Ok(Self {
            store: Mutex::new(PostgresStore::connect_in_schema(url, schema)?),
            lifecycles,
        })
    }

    fn backend(
        &self,
        durable: &PostgresStore,
        instances: Vec<EntityInstance>,
    ) -> Result<EntityBackend<MemoryStore, Identity>, QueryError> {
        let mut scoped = MemoryStore::new();
        for instance in instances {
            let events = if instance.entity == STORED_AS {
                durable
                    .events(&instance.entity, &instance.id)
                    .map_err(unavailable)?
            } else {
                Vec::new()
            };
            scoped
                .commit(&Decision::legacy_import(instance, events), Expect::Absent)
                .map_err(unavailable)?;
        }
        EntityBackend::shaped(scoped, Identity::with_lifecycles(self.lifecycles.clone())).map_err(
            |error| QueryError::Unavailable {
                reason: format!("the selected provider rows could not be materialized: {error}"),
            },
        )
    }

    fn query_page(
        store: &PostgresStore,
        query: &DocumentQuery,
    ) -> Result<DocumentPage, QueryError> {
        store.query_documents(query).map_err(|error| match error {
            entity_query::QueryError::Invalid(reason) => QueryError::Invalid { reason },
            entity_query::QueryError::Store(error) => unavailable(error),
        })
    }
}

#[allow(clippy::unused_async_trait_impl)]
impl QueryService for IndexedPostgresQueries {
    type AuditRecord = AuditRecord;

    async fn get(
        &self,
        reference: &EntityRef,
        consistency: QueryConsistency,
    ) -> Result<EntityEnvelope, QueryError> {
        ensure_consistency(&consistency)?;
        let durable = self.store.lock().expect("the query store is not poisoned");
        let rows = durable
            .load(STORED_AS, reference.id.as_str())
            .map_err(unavailable)?
            .into_iter()
            .collect();
        let backend = self.backend(&durable, rows)?;
        block_on(backend.get(reference, QueryConsistency::Current))
    }

    async fn resolve(&self, locator: &EntityLocator) -> Result<EntityId, QueryError> {
        let query = DocumentQuery::for_entity(STORED_AS)
            .matching(METADATA_KEY, json!({"metadata": {"locator": locator}}))
            .with_limit(2);
        let durable = self.store.lock().expect("the query store is not poisoned");
        let rows = Self::query_page(&durable, &query)?.items;
        if rows.len() > 1 {
            return Err(QueryError::Unavailable {
                reason: format!("locator `{locator}` names more than one durable entity"),
            });
        }
        let backend = self.backend(&durable, rows)?;
        block_on(backend.resolve(locator))
    }

    async fn query(&self, query: &EntityQuery) -> Result<Page<EntityEnvelope>, QueryError> {
        ensure_consistency(&query.consistency)?;
        let identity = entity_query_identity(query)?;
        let mut documents =
            DocumentQuery::for_entity(STORED_AS).with_limit(query.limit.unwrap_or(DEFAULT_PAGE));
        if let Some(entity_type) = &query.entity_type {
            documents =
                documents.matching(METADATA_KEY, json!({"metadata": {"type": entity_type}}));
        }
        for (field, value) in &query.matching {
            documents = documents.matching(field, serde_json::to_value(value).map_err(invalid)?);
        }
        if let Some(after) = &query.after {
            documents = documents.after(provider_cursor(after, identity)?);
        }

        let durable = self.store.lock().expect("the query store is not poisoned");
        let page = Self::query_page(&durable, &documents)?;
        let mut rows = page.items;
        if let Some(anchor) = &query.related_to {
            let mut relations = Vec::new();
            for row in &rows {
                let mut relation = DocumentQuery::for_entity(RELATIONS_AS)
                    .matching(
                        "relation",
                        json!({"source": anchor, "target": {"id": row.id}}),
                    )
                    .with_limit(1);
                if let Some(kind) = query.relation {
                    relation = relation.matching("relation", json!({"kind": kind}));
                }
                relations.extend(Self::query_page(&durable, &relation)?.items);
            }
            rows.extend(relations);
        }
        let backend = self.backend(&durable, rows)?;
        let mut semantic = query.clone();
        semantic.limit = None;
        semantic.after = None;
        semantic.consistency = QueryConsistency::Current;
        let mut result = block_on(backend.query(&semantic))?;
        result.next = page.next.map(|next| indexed_cursor(identity, &next));
        Ok(result)
    }

    async fn relations(&self, query: &RelationQuery) -> Result<Page<Relation>, QueryError> {
        ensure_consistency(&query.consistency)?;
        let identity = relation_query_identity(query)?;
        let mut documents =
            DocumentQuery::for_entity(RELATIONS_AS).with_limit(query.limit.unwrap_or(DEFAULT_PAGE));
        if let Some(source) = &query.source {
            documents = documents.matching("relation", json!({"source": source}));
        }
        if let Some(target) = &query.target {
            documents = documents.matching("relation", json!({"target": target}));
        }
        if let Some(kind) = query.kind {
            documents = documents.matching("relation", json!({"kind": kind}));
        }
        if let Some(after) = &query.after {
            documents = documents.after(provider_cursor(after, identity)?);
        }
        let durable = self.store.lock().expect("the query store is not poisoned");
        let page = Self::query_page(&durable, &documents)?;
        let backend = self.backend(&durable, page.items)?;
        let mut semantic = query.clone();
        semantic.limit = None;
        semantic.after = None;
        semantic.consistency = QueryConsistency::Current;
        let mut result = block_on(backend.relations(&semantic))?;
        result.next = page.next.map(|next| indexed_cursor(identity, &next));
        Ok(result)
    }

    async fn history(&self, reference: &EntityRef) -> Result<Vec<RevisionRecord>, QueryError> {
        let durable = self.store.lock().expect("the query store is not poisoned");
        let rows = durable
            .load(STORED_AS, reference.id.as_str())
            .map_err(unavailable)?
            .into_iter()
            .collect();
        let backend = self.backend(&durable, rows)?;
        block_on(backend.history(reference))
    }

    async fn history_page(&self, query: &HistoryQuery) -> Result<Page<RevisionRecord>, QueryError> {
        ensure_consistency(&query.consistency)?;
        let history = self.history(&query.entity).await?;
        Page::paginate(history, query.limit, query.after.as_ref())
    }

    async fn audit(&self, query: &AuditQuery) -> Result<Page<Self::AuditRecord>, QueryError> {
        let identity = audit_query_identity(query)?;
        let mut documents =
            DocumentQuery::for_entity(AUDIT_AS).with_limit(query.limit.unwrap_or(DEFAULT_PAGE));
        if let Some(entity) = &query.entity {
            documents = documents.matching("record", json!({"subject": entity}));
        }
        if let Some(correlation) = &query.correlation_id {
            documents = documents.matching("record", json!({"correlation_id": correlation}));
        }
        if let Some(command) = &query.command_id {
            documents = documents.matching("record", json!({"command_id": command}));
        }
        if let Some(actor) = &query.actor {
            documents = documents.matching("record", json!({"actor": actor}));
        }
        if let Some(kind) = &query.kind {
            documents = documents.matching("record", json!({"kind": kind}));
        }
        if let Some(after) = &query.after {
            documents = documents.after(provider_cursor(after, identity)?);
        }
        let durable = self.store.lock().expect("the query store is not poisoned");
        let page = Self::query_page(&durable, &documents)?;
        let backend = self.backend(&durable, page.items)?;
        let mut semantic = query.clone();
        semantic.limit = None;
        semantic.after = None;
        let mut result = block_on(backend.audit(&semantic))?;
        result.next = page.next.map(|next| indexed_cursor(identity, &next));
        Ok(result)
    }

    async fn describe_type(&self, entity_type: &EntityType) -> Result<TypeDescriptor, QueryError> {
        let durable = self.store.lock().expect("the query store is not poisoned");
        let backend = self.backend(&durable, Vec::new())?;
        block_on(backend.describe_type(entity_type))
    }
}

fn ensure_consistency(consistency: &QueryConsistency) -> Result<(), QueryError> {
    let Some(token) = consistency.token() else {
        return Ok(());
    };
    let valid = token
        .as_str()
        .strip_prefix("seq-")
        .is_some_and(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()));
    if valid {
        // Commands return only after their PostgreSQL transaction commits. Reads use the same
        // primary authority, so every token this authority issued is already visible here.
        Ok(())
    } else {
        Err(QueryError::ConsistencyTimeout {
            token: token.to_string(),
        })
    }
}

fn entity_query_identity(query: &EntityQuery) -> Result<u64, QueryError> {
    let mut query = query.clone();
    query.after = None;
    identity(&query)
}

fn relation_query_identity(query: &RelationQuery) -> Result<u64, QueryError> {
    let mut query = query.clone();
    query.after = None;
    identity(&query)
}

fn audit_query_identity(query: &AuditQuery) -> Result<u64, QueryError> {
    let mut query = query.clone();
    query.after = None;
    identity(&query)
}

fn identity(value: &impl Serialize) -> Result<u64, QueryError> {
    serde_json::to_vec(value)
        .map(|bytes| digest(&bytes))
        .map_err(invalid)
}

fn indexed_cursor(identity: u64, cursor: &QueryCursor) -> Cursor {
    Cursor(format!("indexed-{identity:016x}.{}", cursor.as_str()))
}

fn provider_cursor(cursor: &Cursor, identity: u64) -> Result<QueryCursor, QueryError> {
    let prefix = format!("indexed-{identity:016x}.");
    let raw = cursor
        .0
        .strip_prefix(&prefix)
        .ok_or_else(|| QueryError::Invalid {
            reason: "the cursor belongs to another indexed query".to_owned(),
        })?;
    serde_json::from_value(Value::String(raw.to_owned())).map_err(invalid)
}

fn digest(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

fn invalid(error: impl std::fmt::Display) -> QueryError {
    QueryError::Invalid {
        reason: error.to_string(),
    }
}

fn unavailable(error: impl std::fmt::Display) -> QueryError {
    QueryError::Unavailable {
        reason: format!("the PostgreSQL authority could not answer the indexed query: {error}"),
    }
}
