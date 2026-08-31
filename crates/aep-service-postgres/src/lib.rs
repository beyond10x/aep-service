//! Transactional PostgreSQL authority for one configured AEP realm and workspace.
//!
//! Each request gets a newly opened EP backend over Entity Runtime's PostgreSQL provider. The
//! backend may hydrate a candidate view in order to apply today's EP contract, but that view is
//! never retained as service authority: Entity Runtime reads and locks every expected revision in
//! the committing database transaction. A stale candidate loses with a revision conflict, and the
//! complete EP batch lands or rolls back together.

use std::fmt;
use std::future::ready;

use aep_backend_postgres::PostgresBackend;
use aep_service_app::{ServiceBindingError, ServiceProvider, TrustedRequestContext};

/// Configuration rejected before any database connection is attempted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityConfigError {
    reason: String,
}

impl AuthorityConfigError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    /// The operator-safe explanation.
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl fmt::Display for AuthorityConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for AuthorityConfigError {}

/// Opens fresh semantic service handles for one realm/workspace PostgreSQL authority.
///
/// Wave 1 deliberately configures one workspace. A later identity story can introduce
/// workspace-scoped coordinates without letting two workspaces share today's unscoped EP ids.
/// Keeping the admitted scope explicit now prevents that later change from becoming a data leak.
pub struct PostgresAuthority {
    database_url: String,
    realm: String,
    workspace: String,
    schema: String,
}

impl fmt::Debug for PostgresAuthority {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PostgresAuthority")
            .field("database_url", &"[redacted]")
            .field("realm", &self.realm)
            .field("workspace", &self.workspace)
            .field("schema", &self.schema)
            .finish()
    }
}

impl PostgresAuthority {
    /// Configures one authority without opening the database.
    ///
    /// PostgreSQL silently truncates identifiers beyond 63 bytes. Restricting the schema to a
    /// portable unquoted identifier prevents two configured realms from being truncated onto the
    /// same storage boundary.
    pub fn new(
        database_url: impl Into<String>,
        realm: impl Into<String>,
        workspace: impl Into<String>,
        schema: impl Into<String>,
    ) -> Result<Self, AuthorityConfigError> {
        let database_url = database_url.into();
        let realm = realm.into();
        let workspace = workspace.into();
        let schema = schema.into();
        non_empty("database URL", &database_url)?;
        non_empty("realm", &realm)?;
        non_empty("workspace", &workspace)?;
        validate_schema(&schema)?;
        Ok(Self {
            database_url,
            realm,
            workspace,
            schema,
        })
    }

    /// The globally unique authority and storage boundary admitted by this process.
    pub fn realm(&self) -> &str {
        &self.realm
    }

    /// The one repository/project boundary admitted by the Wave 1 deployment.
    pub fn workspace(&self) -> &str {
        &self.workspace
    }

    /// The PostgreSQL schema dedicated to this realm.
    pub fn schema(&self) -> &str {
        &self.schema
    }

    fn bind_now(
        &self,
        context: &TrustedRequestContext,
    ) -> Result<PostgresBackend, ServiceBindingError> {
        let scope = context.scope();
        if scope.realm() != self.realm || scope.workspace() != self.workspace {
            return Err(ServiceBindingError::unavailable(
                "the requested authority is not served by this process",
            ));
        }
        if !context
            .principal()
            .authorizes(scope.realm(), scope.workspace())
        {
            return Err(ServiceBindingError::unavailable(
                "the trusted principal is not admitted to the requested authority",
            ));
        }
        PostgresBackend::connect_in_schema(&self.database_url, &self.schema).map_err(|_| {
            ServiceBindingError::unavailable("the PostgreSQL authority is unavailable")
        })
    }
}

impl ServiceProvider for PostgresAuthority {
    type Service = PostgresBackend;

    fn bind(
        &self,
        context: &TrustedRequestContext,
    ) -> impl std::future::Future<Output = Result<Self::Service, ServiceBindingError>> {
        ready(self.bind_now(context))
    }
}

fn non_empty(name: &str, value: &str) -> Result<(), AuthorityConfigError> {
    if value.trim().is_empty() {
        Err(AuthorityConfigError::new(format!(
            "the {name} must not be empty"
        )))
    } else {
        Ok(())
    }
}

fn validate_schema(schema: &str) -> Result<(), AuthorityConfigError> {
    non_empty("schema", schema)?;
    if schema.len() > 63 {
        return Err(AuthorityConfigError::new(
            "the schema must be at most 63 bytes",
        ));
    }
    let mut chars = schema.chars();
    let starts_valid = chars
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_lowercase());
    if !starts_valid
        || !chars.all(|character| {
            character == '_' || character.is_ascii_lowercase() || character.is_ascii_digit()
        })
    {
        return Err(AuthorityConfigError::new(
            "the schema must match [a-z_][a-z0-9_]*",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use aep_contract::testing::block_on;
    use aep_domain::time::Timestamp;
    use aep_service_app::ServiceScope;
    use aep_service_auth::VerifiedPrincipal;

    use super::*;

    fn context(realm: &str, workspace: &str) -> TrustedRequestContext {
        TrustedRequestContext::new(
            VerifiedPrincipal::new(
                "human:alice".parse().expect("authority"),
                None,
                realm,
                [workspace.to_owned()],
                ["engineer".to_owned()],
                None,
            ),
            ServiceScope::new(realm, workspace),
            "request-1".parse().expect("request id"),
            Timestamp::from_epoch_millis(1_800_000_000_000),
        )
    }

    #[test]
    fn invalid_or_ambiguous_schema_names_are_refused_before_use() {
        for schema in ["", "Upper", "two-realms", "1realm"] {
            let error = PostgresAuthority::new("postgres://unused", "company", "repo", schema)
                .expect_err("invalid schema");
            assert!(
                error.reason().contains("schema"),
                "the refusal names the bad coordinate: {error}"
            );
        }
        let too_long = "r".repeat(64);
        let error = PostgresAuthority::new("postgres://unused", "company", "repo", too_long)
            .expect_err("truncated schemas are unsafe");
        assert_eq!(error.reason(), "the schema must be at most 63 bytes");
    }

    #[test]
    fn an_unserved_scope_is_refused_without_touching_the_database() {
        let authority = PostgresAuthority::new(
            "postgres://this-host-must-never-be-contacted.invalid/database",
            "company",
            "repo",
            "company_planning",
        )
        .expect("configuration");

        let error = block_on(authority.bind(&context("another-realm", "repo")))
            .expect_err("realm mismatch");
        assert_eq!(
            error.reason(),
            "the requested authority is not served by this process"
        );
    }

    #[test]
    fn debug_output_never_contains_database_credentials() {
        let authority = PostgresAuthority::new(
            "postgres://alice:secret@example.invalid/database",
            "company",
            "repo",
            "company_planning",
        )
        .expect("configuration");

        let shown = format!("{authority:?}");
        assert!(shown.contains("[redacted]"));
        assert!(!shown.contains("secret"));
    }
}
