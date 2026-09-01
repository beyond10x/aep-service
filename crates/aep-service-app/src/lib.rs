//! Application orchestration for authenticated AEP commands and queries.
//!
//! This crate composes trusted request context, authorization, AEP decisions and transactional
//! persistence without depending on a concrete network transport.

use std::fmt;

use aep_contract::command::{CausationRef, CommandContext, CommandService};
use aep_contract::query::QueryService;
use aep_domain::audit::AuditRecord;
use aep_domain::command::Command;
use aep_domain::ids::{CorrelationId, ExecutionId, IdempotencyKey, RequestId, TaskId};
use aep_domain::time::Timestamp;
use aep_service_auth::VerifiedPrincipal;

/// The immutable data and authority boundary selected by an AEP route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceScope {
    realm: String,
    workspace: String,
}

impl ServiceScope {
    /// Constructs a realm/workspace scope after path decoding.
    pub fn new(realm: impl Into<String>, workspace: impl Into<String>) -> Self {
        Self {
            realm: realm.into(),
            workspace: workspace.into(),
        }
    }

    /// The globally unique authority and storage boundary.
    pub fn realm(&self) -> &str {
        &self.realm
    }

    /// The repository or project boundary inside the realm.
    pub fn workspace(&self) -> &str {
        &self.workspace
    }
}

/// Verified identity, route scope and server-owned transport metadata for one attempt.
///
/// An authorized service handle is selected from this value before semantic dispatch. That keeps
/// query authorization and realm/workspace selection present even though the storage-independent
/// AEP query trait deliberately has no network request-context parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedRequestContext {
    principal: VerifiedPrincipal,
    scope: ServiceScope,
    request_id: RequestId,
    received_at: Timestamp,
}

impl TrustedRequestContext {
    /// Constructs context exclusively from verified identity and server-owned request metadata.
    pub fn new(
        principal: VerifiedPrincipal,
        scope: ServiceScope,
        request_id: RequestId,
        received_at: Timestamp,
    ) -> Self {
        Self {
            principal,
            scope,
            request_id,
            received_at,
        }
    }

    /// The verified authority, executor, grants and delegation identity.
    pub fn principal(&self) -> &VerifiedPrincipal {
        &self.principal
    }

    /// The requested and authorized realm/workspace boundary.
    pub fn scope(&self) -> &ServiceScope {
        &self.scope
    }

    /// The server-derived identity of this transport attempt.
    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// The server clock value recorded for this attempt.
    pub const fn received_at(&self) -> Timestamp {
        self.received_at
    }

    /// Builds the trusted portion of a semantic command envelope.
    ///
    /// Logical correlation values remain caller-supplied, while authority, executor, request id
    /// and time can only come from this trusted context.
    pub fn command_context(
        &self,
        idempotency_key: IdempotencyKey,
        correlation_id: CorrelationId,
        causation: Option<CausationRef>,
        execution_id: Option<ExecutionId>,
        task: Option<TaskId>,
    ) -> CommandContext {
        let mut context = CommandContext::new(
            self.request_id.clone(),
            idempotency_key,
            self.principal.authority().clone(),
            correlation_id,
            self.received_at,
        );
        context.executor = self.principal.executor().cloned();
        context.causation = causation;
        context.execution_id = execution_id;
        context.task = task;
        context
    }
}

/// A safe failure to bind a verified request to its authorized semantic service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceBindingError {
    reason: String,
}

impl ServiceBindingError {
    /// Constructs a binding failure safe to expose as service unavailability.
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    /// The client-safe reason.
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl fmt::Display for ServiceBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for ServiceBindingError {}

/// Selects a semantic command/query implementation already narrowed to trusted request context.
///
/// The selected service still implements AEP's ordinary semantic traits directly; this port adds
/// no second command model. It exists so realm, workspace, roles and delegation scope cannot be
/// dropped before a query is materialized.
pub trait ServiceProvider {
    /// The authorized semantic service handle selected for one request.
    type Service: CommandService<Command = Command> + QueryService<AuditRecord = AuditRecord>;

    /// Binds one trusted request to its authorized semantic service handle.
    fn bind(
        &self,
        context: &TrustedRequestContext,
    ) -> impl std::future::Future<Output = Result<Self::Service, ServiceBindingError>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_context_overwrites_every_attribution_field_the_network_must_not_supply() {
        let principal = VerifiedPrincipal::new(
            "human:alice".parse().expect("authority"),
            Some("agent:planner".parse().expect("executor")),
            "company",
            ["repo".to_owned()],
            ["engineer".to_owned()],
            Some("delegation-1".to_owned()),
        );
        let trusted = TrustedRequestContext::new(
            principal,
            ServiceScope::new("company", "repo"),
            "server-request-1".parse().expect("request id"),
            Timestamp::from_epoch_millis(1_800_000_000_000),
        );
        let context = trusted.command_context(
            "retry-1".parse().expect("key"),
            "correlation-1".parse().expect("correlation"),
            None,
            None,
            None,
        );

        assert_eq!(context.actor.to_string(), "human:alice");
        assert_eq!(
            context.executor.map(|actor| actor.to_string()).as_deref(),
            Some("agent:planner")
        );
        assert_eq!(context.request_id.to_string(), "server-request-1");
        assert_eq!(
            context.issued_at,
            Timestamp::from_epoch_millis(1_800_000_000_000)
        );
        assert_eq!(trusted.scope().realm(), "company");
        assert_eq!(trusted.scope().workspace(), "repo");
    }
}
