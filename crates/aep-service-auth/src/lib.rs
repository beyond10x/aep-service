//! Verification and authorization boundaries for human and delegated-agent identities.
//!
//! Token formats and issuers are adapters. The application consumes only trusted principals,
//! delegation scopes and the decision explaining why an operation was or was not permitted.

use std::collections::BTreeSet;
use std::fmt;

use aep_domain::entity::ActorRef;

/// A credential-verifier result containing no credential bytes.
///
/// `authority` answers on whose behalf the operation occurs. `executor` is present only when a
/// different actor actually performs it. The verifier has already intersected owner grants,
/// delegation scope and executor restrictions before constructing this value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPrincipal {
    authority: ActorRef,
    executor: Option<ActorRef>,
    realm: String,
    workspace_grants: BTreeSet<String>,
    roles: BTreeSet<String>,
    delegation_id: Option<String>,
}

impl VerifiedPrincipal {
    /// Constructs the trusted result of credential and delegation verification.
    ///
    /// An executor equal to the authority is normalized away so a human request has one canonical
    /// attribution shape.
    pub fn new(
        authority: ActorRef,
        executor: Option<ActorRef>,
        realm: impl Into<String>,
        workspace_grants: impl IntoIterator<Item = String>,
        roles: impl IntoIterator<Item = String>,
        delegation_id: Option<String>,
    ) -> Self {
        let executor = executor.filter(|candidate| candidate != &authority);
        Self {
            authority,
            executor,
            realm: realm.into(),
            workspace_grants: workspace_grants.into_iter().collect(),
            roles: roles.into_iter().collect(),
            delegation_id,
        }
    }

    /// The actor on whose behalf a request occurs.
    pub fn authority(&self) -> &ActorRef {
        &self.authority
    }

    /// What actually performs the request, when different from the authority.
    pub fn executor(&self) -> Option<&ActorRef> {
        self.executor.as_ref()
    }

    /// The one realm in which this principal was verified.
    pub fn realm(&self) -> &str {
        &self.realm
    }

    /// The workspaces remaining after every grant-narrowing step.
    pub fn workspace_grants(&self) -> &BTreeSet<String> {
        &self.workspace_grants
    }

    /// Roles remaining after every grant-narrowing step.
    pub fn roles(&self) -> &BTreeSet<String> {
        &self.roles
    }

    /// The verified delegation identity, when an executor acts for an authority.
    pub fn delegation_id(&self) -> Option<&str> {
        self.delegation_id.as_deref()
    }

    /// Whether this principal may enter the requested realm and workspace boundary.
    pub fn authorizes(&self, realm: &str, workspace: &str) -> bool {
        self.realm == realm && self.workspace_grants.contains(workspace)
    }
}

/// A safe authentication refusal from a credential adapter.
///
/// The reason must describe the failure class and must never contain credential bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticationError {
    reason: String,
}

impl AuthenticationError {
    /// Constructs an authentication refusal with a client-safe reason.
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    /// The safe refusal reason.
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl fmt::Display for AuthenticationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for AuthenticationError {}

/// Verifies a transport credential and any delegated-agent proof it carries.
///
/// The raw authorization value is confined to this port. Callers retain neither it nor any token
/// claims after verification.
pub trait CredentialVerifier {
    /// Returns a trusted principal or a safe authentication refusal.
    fn verify(
        &self,
        authorization: Option<&str>,
    ) -> impl std::future::Future<Output = Result<VerifiedPrincipal, AuthenticationError>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_human_executor_is_canonicalized_but_delegated_executor_is_retained() {
        let alice: ActorRef = "human:alice".parse().expect("actor");
        let human = VerifiedPrincipal::new(
            alice.clone(),
            Some(alice.clone()),
            "company",
            ["repo".to_owned()],
            ["engineer".to_owned()],
            None,
        );
        assert_eq!(human.executor(), None);
        assert!(human.authorizes("company", "repo"));

        let planner = "agent:planner".parse().expect("actor");
        let delegated = VerifiedPrincipal::new(
            alice,
            Some(planner),
            "company",
            ["repo".to_owned()],
            ["engineer".to_owned()],
            Some("delegation-1".to_owned()),
        );
        assert_eq!(
            delegated.executor().map(ToString::to_string).as_deref(),
            Some("agent:planner")
        );
        assert!(!delegated.authorizes("another-realm", "repo"));
        assert!(!delegated.authorizes("company", "another-repo"));
    }
}
