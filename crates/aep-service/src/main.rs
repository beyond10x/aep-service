//! Runnable single-realm AEP authority with an explicit loopback-only development verifier.

use std::collections::BTreeMap;
use std::env;
use std::future::ready;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aep_client::wire::{Method, Request};
use aep_contract::testing::block_on;
use aep_domain::artifact::LifecycleRegistry;
use aep_domain::ids::RequestId;
use aep_domain::time::Timestamp;
use aep_service_auth::{AuthenticationError, CredentialVerifier, VerifiedPrincipal};
use aep_service_http::{
    response_media_type, unavailable_response, AepHttpService, RequestMetadata,
};
use aep_service_postgres::PostgresAuthority;
use axum::body::{to_bytes, Body};
use axum::extract::{Request as AxumRequest, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::Response as AxumResponse;
use axum::routing::{any, get};
use axum::Router;
use clap::{Parser, Subcommand};
use reqwest::header::{AUTHORIZATION, CACHE_CONTROL, PRAGMA};
use serde::Deserialize;
use sha2::{Digest as _, Sha256};
use tokio::sync::Semaphore;
use url::Url;
use uuid::Uuid;

const MAX_BODY_BYTES: usize = 1024 * 1024;

#[derive(Debug, Parser)]
#[command(name = "aep-service", about = "Run one central AEP authority")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Serves one configured realm and workspace.
    Serve(Box<ServeArgs>),
    /// Prints the generated `OpenAPI` document to standard output.
    Openapi,
    /// Inspects an immutable AEP definition tree for deployment.
    Definitions(DefinitionsArgs),
    /// Checks a running process's liveness or readiness endpoint.
    Probe(ProbeArgs),
}

#[derive(Debug, clap::Args)]
struct DefinitionsArgs {
    #[command(subcommand)]
    command: DefinitionsCommand,
}

#[derive(Debug, Subcommand)]
enum DefinitionsCommand {
    /// Validates a definition tree and prints its source-byte digest.
    Digest {
        /// Root of the AEP definition tree.
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Debug, clap::Args)]
struct ProbeArgs {
    /// Listener address to probe.
    #[arg(long, default_value = "127.0.0.1:8080")]
    address: SocketAddr,
    /// Probe readiness instead of process liveness.
    #[arg(long)]
    readiness: bool,
    /// Connection and response timeout in milliseconds.
    #[arg(long, default_value_t = 2_000)]
    timeout_ms: u64,
}

#[derive(Debug, clap::Args)]
struct ServeArgs {
    /// Listener address; development authentication requires a loopback address.
    #[arg(long, default_value = "127.0.0.1:8080")]
    bind: SocketAddr,
    /// Environment variable containing the `PostgreSQL` URL.
    #[arg(long, default_value = "AEP_DATABASE_URL")]
    database_url_env: String,
    /// Globally unique realm served by this process.
    #[arg(long)]
    realm: String,
    /// One workspace served by this process.
    #[arg(long)]
    workspace: String,
    /// `PostgreSQL` schema dedicated to the realm.
    #[arg(long)]
    schema: String,
    /// Root of the immutable AEP definition tree.
    #[arg(long)]
    definitions: PathBuf,
    /// Expected lowercase SHA-256 of sorted definition paths and bytes.
    #[arg(long)]
    definition_digest: String,
    /// Identity origin for hosted authentication; omit only for loopback development.
    #[arg(long, env = "AEP_IDENTITY_ORIGIN")]
    identity_origin: Option<String>,
    /// Exact Identity relying-party audience.
    #[arg(
        long,
        env = "AEP_IDENTITY_AUDIENCE",
        default_value = "urn:b10x:aep-service"
    )]
    identity_audience: String,
    /// Exact Identity tenant admitted to this single-realm authority.
    #[arg(long, env = "AEP_IDENTITY_TENANT")]
    identity_tenant: Option<String>,
    /// Environment variable containing the exact development bearer token.
    #[arg(long, default_value = "AEP_DEV_BEARER_TOKEN")]
    dev_token_env: String,
    /// Human authority attributed to development requests.
    #[arg(long, default_value = "human:developer")]
    dev_authority: String,
    /// Maximum simultaneous blocking database exchanges.
    #[arg(long, default_value_t = 32)]
    database_concurrency: usize,
    /// Permit the development bearer verifier on a non-loopback listener.
    #[arg(long)]
    allow_insecure_dev_listener: bool,
    /// Maximum wait for a database execution slot in milliseconds.
    #[arg(long, default_value_t = 2_000)]
    queue_timeout_ms: u64,
    /// Maximum duration of one database-backed exchange in milliseconds.
    #[arg(long, default_value_t = 30_000)]
    request_timeout_ms: u64,
    /// Maximum graceful-drain duration after SIGINT or SIGTERM in milliseconds.
    #[arg(long, default_value_t = 15_000)]
    shutdown_timeout_ms: u64,
}

#[derive(Clone)]
struct DevelopmentVerifier {
    authorization: Arc<str>,
    principal: VerifiedPrincipal,
}

impl CredentialVerifier for DevelopmentVerifier {
    fn verify(
        &self,
        authorization: Option<&str>,
    ) -> impl std::future::Future<Output = Result<VerifiedPrincipal, AuthenticationError>> {
        ready(if authorization == Some(self.authorization.as_ref()) {
            Ok(self.principal.clone())
        } else {
            Err(AuthenticationError::new(
                "development bearer token is invalid",
            ))
        })
    }
}

#[derive(Clone)]
struct IdentityVerifier {
    client: IdentityAuthorityClient,
    tenant: Arc<str>,
    realm: Arc<str>,
    workspace: Arc<str>,
}

#[derive(Clone)]
struct IdentityAuthorityClient {
    origin: Url,
    audience: Arc<str>,
    http: reqwest::Client,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityAuthority {
    #[serde(rename = "iss")]
    _issuer: String,
    #[serde(rename = "sub")]
    subject: String,
    #[serde(rename = "aud")]
    audience: String,
    #[serde(rename = "exp")]
    _expires_at: i64,
    #[serde(rename = "email")]
    _email: Option<String>,
    tenant_id: String,
    groups: Vec<String>,
}

impl IdentityAuthorityClient {
    fn new(origin: &str, audience: &str) -> Result<Self, &'static str> {
        let origin = Url::parse(origin).map_err(|_| "Identity origin is invalid")?;
        let internal_http = origin.scheme() == "http"
            && origin.host_str().is_some_and(|host| {
                host == "127.0.0.1" || host == "localhost" || host.ends_with(".svc.cluster.local")
            });
        if !(origin.scheme() == "https" || internal_http)
            || origin.host_str().is_none()
            || !origin.username().is_empty()
            || origin.password().is_some()
            || origin.path() != "/"
            || origin.query().is_some()
            || origin.fragment().is_some()
        {
            return Err("Identity origin is invalid");
        }
        if audience.trim() != audience
            || !(3..=256).contains(&audience.len())
            || !audience.is_ascii()
            || audience
                .bytes()
                .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
        {
            return Err("Identity audience is invalid");
        }
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|_| "Identity HTTP client is unavailable")?;
        Ok(Self {
            origin,
            audience: Arc::from(audience),
            http,
        })
    }

    async fn resolve_session(
        &self,
        authorization: &str,
    ) -> Result<IdentityAuthority, AuthenticationError> {
        let authorization = HeaderValue::from_str(authorization)
            .map_err(|_| AuthenticationError::new("Identity session is malformed"))?;
        let endpoint = self
            .origin
            .join("v1/session-authority")
            .map_err(|_| AuthenticationError::new("Identity endpoint is invalid"))?;
        let response = self
            .http
            .get(endpoint)
            .header(AUTHORIZATION, authorization)
            .header("x-b10x-audience", self.audience.as_ref())
            .send()
            .await
            .map_err(|_| AuthenticationError::new("Identity authority is unavailable"))?;
        let confidential = response
            .headers()
            .get(CACHE_CONTROL)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.split(',').any(|part| part.trim() == "no-store"))
            && response
                .headers()
                .get(PRAGMA)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.split(',').any(|part| part.trim() == "no-cache"));
        if !confidential {
            return Err(AuthenticationError::new(
                "Identity returned a cacheable credential response",
            ));
        }
        if !response.status().is_success() {
            return Err(AuthenticationError::new("Identity refused the session"));
        }
        let authority: IdentityAuthority = response
            .json()
            .await
            .map_err(|_| AuthenticationError::new("Identity authority response is invalid"))?;
        if authority.audience != self.audience.as_ref() {
            return Err(AuthenticationError::new(
                "Identity returned the wrong audience",
            ));
        }
        Ok(authority)
    }
}

impl CredentialVerifier for IdentityVerifier {
    async fn verify(
        &self,
        authorization: Option<&str>,
    ) -> Result<VerifiedPrincipal, AuthenticationError> {
        let authorization = authorization
            .ok_or_else(|| AuthenticationError::new("an Identity session is required"))?;
        let authority = self
            .client
            .resolve_session(authorization)
            .await
            .map_err(|_| AuthenticationError::new("Identity refused the session"))?;
        if authority.tenant_id != self.tenant.as_ref() {
            return Err(AuthenticationError::new(
                "Identity tenant is outside this AEP authority",
            ));
        }
        let mut digest = Sha256::new();
        digest.update(b"b10x/aep-service/identity-actor/v1\0");
        digest.update(authority.tenant_id.as_bytes());
        digest.update(b"\0");
        digest.update(authority.subject.as_bytes());
        let actor = format!("human:{:x}", digest.finalize());
        Ok(VerifiedPrincipal::new(
            actor
                .parse()
                .map_err(|_| AuthenticationError::new("Identity subject is invalid"))?,
            None,
            self.realm.as_ref(),
            [self.workspace.to_string()],
            authority.groups,
            None,
        ))
    }
}

#[derive(Clone)]
enum RuntimeVerifier {
    Development(DevelopmentVerifier),
    Identity(IdentityVerifier),
}

impl CredentialVerifier for RuntimeVerifier {
    async fn verify(
        &self,
        authorization: Option<&str>,
    ) -> Result<VerifiedPrincipal, AuthenticationError> {
        match self {
            Self::Development(verifier) => verifier.verify(authorization).await,
            Self::Identity(verifier) => verifier.verify(authorization).await,
        }
    }
}

struct RuntimeState {
    service: Arc<AepHttpService<RuntimeVerifier, PostgresAuthority>>,
    database_slots: Arc<Semaphore>,
    queue_timeout: Duration,
    request_timeout: Duration,
    openapi: Arc<[u8]>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Cli { command } = Cli::parse();
    match command {
        Command::Serve(arguments) => serve(*arguments).await,
        Command::Openapi => {
            std::io::stdout().write_all(&aep_service_openapi::document_bytes())?;
            Ok(())
        }
        Command::Definitions(arguments) => match arguments.command {
            DefinitionsCommand::Digest { path } => {
                let bundle = aep_project::load_bundle(&path)?;
                println!("{}", bundle.digest);
                Ok(())
            }
        },
        Command::Probe(arguments) => probe(&arguments),
    }
}

async fn serve(arguments: ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    let hosted = arguments.identity_origin.is_some() || arguments.identity_tenant.is_some();
    if arguments.identity_origin.is_some() != arguments.identity_tenant.is_some() {
        return Err("Identity origin and tenant must be configured together".into());
    }
    validate_listener(
        arguments.bind,
        hosted,
        arguments.allow_insecure_dev_listener,
    )?;
    if !hosted && !arguments.bind.ip().is_loopback() {
        eprintln!(
            "WARNING: development bearer authentication is exposed on non-loopback {}",
            arguments.bind
        );
    }
    if arguments.database_concurrency == 0 {
        return Err("database concurrency must be greater than zero".into());
    }
    if arguments.queue_timeout_ms == 0
        || arguments.request_timeout_ms == 0
        || arguments.shutdown_timeout_ms == 0
    {
        return Err("queue, request and shutdown timeouts must be greater than zero".into());
    }
    let verifier = runtime_verifier(&arguments)?;
    let database_url = env::var(&arguments.database_url_env).map_err(|_| {
        format!(
            "{} must contain the PostgreSQL URL",
            arguments.database_url_env
        )
    })?;
    let bundle =
        aep_project::load_pinned_bundle(&arguments.definitions, &arguments.definition_digest)?;
    let lifecycles = bundle.registry.lifecycles().clone();

    let authority = prepare_authority(
        database_url,
        arguments.realm.clone(),
        arguments.workspace.clone(),
        arguments.schema,
        lifecycles,
    )
    .await
    .map_err(std::io::Error::other)?;
    let state = Arc::new(RuntimeState {
        service: Arc::new(AepHttpService::new(verifier, authority)),
        database_slots: Arc::new(Semaphore::new(arguments.database_concurrency)),
        queue_timeout: Duration::from_millis(arguments.queue_timeout_ms),
        request_timeout: Duration::from_millis(arguments.request_timeout_ms),
        openapi: Arc::from(aep_service_openapi::document_bytes()),
    });
    let application = Router::new()
        .route("/livez", get(healthy))
        .route("/readyz", get(healthy))
        .route("/openapi.json", get(openapi))
        .fallback(any(dispatch))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(arguments.bind).await?;
    let (shutdown, receiving) = tokio::sync::oneshot::channel();
    let server = axum::serve(listener, application).with_graceful_shutdown(async {
        let _ = receiving.await;
    });
    let mut server = tokio::spawn(async move { server.await });
    tokio::select! {
        result = &mut server => flatten_server_result(result),
        () = shutdown_signal() => {
            let _ = shutdown.send(());
            if let Ok(result) = tokio::time::timeout(
                Duration::from_millis(arguments.shutdown_timeout_ms),
                &mut server,
            ).await {
                flatten_server_result(result)
            } else {
                server.abort();
                let _ = server.await;
                Err("graceful shutdown exceeded its configured timeout".into())
            }
        }
    }
}

fn runtime_verifier(arguments: &ServeArgs) -> Result<RuntimeVerifier, Box<dyn std::error::Error>> {
    let verifier = if let (Some(origin), Some(tenant)) = (
        arguments.identity_origin.as_deref(),
        arguments.identity_tenant.as_deref(),
    ) {
        RuntimeVerifier::Identity(IdentityVerifier {
            client: IdentityAuthorityClient::new(origin, &arguments.identity_audience)?,
            tenant: Arc::from(tenant),
            realm: Arc::from(arguments.realm.as_str()),
            workspace: Arc::from(arguments.workspace.as_str()),
        })
    } else {
        let raw_token = env::var(&arguments.dev_token_env).map_err(|_| {
            format!(
                "{} must contain the development token",
                arguments.dev_token_env
            )
        })?;
        if raw_token.trim().is_empty() {
            return Err("the development bearer token must not be empty".into());
        }
        RuntimeVerifier::Development(DevelopmentVerifier {
            authorization: Arc::from(format!("Bearer {raw_token}")),
            principal: VerifiedPrincipal::new(
                arguments.dev_authority.parse()?,
                None,
                arguments.realm.clone(),
                [arguments.workspace.clone()],
                ["developer".to_owned()],
                None,
            ),
        })
    };
    Ok(verifier)
}

async fn prepare_authority(
    database_url: String,
    realm: String,
    workspace: String,
    schema: String,
    lifecycles: LifecycleRegistry,
) -> Result<PostgresAuthority, String> {
    tokio::task::spawn_blocking(move || {
        let authority = PostgresAuthority::new(database_url, realm, workspace, schema, lifecycles)
            .map_err(|error| error.to_string())?;
        authority.prepare().map_err(|error| error.to_string())?;
        Ok(authority)
    })
    .await
    .map_err(|_| "PostgreSQL authority preparation task failed".to_owned())?
}

fn validate_listener(
    bind: SocketAddr,
    hosted: bool,
    allow_insecure: bool,
) -> Result<(), &'static str> {
    if !hosted && !bind.ip().is_loopback() && !allow_insecure {
        Err("the development verifier refuses a non-loopback listener unless --allow-insecure-dev-listener is explicit")
    } else {
        Ok(())
    }
}

async fn dispatch(State(state): State<Arc<RuntimeState>>, request: AxumRequest) -> AxumResponse {
    let (parts, body) = request.into_parts();
    let metadata = request_metadata();
    let media_type = response_media_type(
        parts
            .headers
            .get(header::ACCEPT)
            .and_then(|value| value.to_str().ok()),
    );
    let Ok(permit) = tokio::time::timeout(
        state.queue_timeout,
        Arc::clone(&state.database_slots).acquire_owned(),
    )
    .await
    else {
        return wire_response(unavailable_response(
            metadata.request_id,
            "database execution queue is full",
            media_type,
        ));
    };
    let Ok(permit) = permit else {
        return wire_response(unavailable_response(
            metadata.request_id,
            "database execution queue is closed",
            media_type,
        ));
    };
    let method = match parts.method.as_str() {
        "GET" => Method::Get,
        "POST" => Method::Post,
        _ => return StatusCode::METHOD_NOT_ALLOWED.into_response(),
    };
    let body = match to_bytes(body, MAX_BODY_BYTES).await {
        Ok(body) => body.to_vec(),
        Err(_) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
    };
    let headers = parts
        .headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), value.to_owned()))
        })
        .collect::<BTreeMap<_, _>>();
    let exchange = Request {
        method,
        path: parts.uri.path().to_owned(),
        headers,
        body,
    };
    let service = Arc::clone(&state.service);
    let timeout_request_id = metadata.request_id.clone();
    let execution = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        block_on(service.handle(exchange, metadata))
    });
    let response = match tokio::time::timeout(state.request_timeout, execution).await {
        Ok(Ok(response)) => response,
        Ok(Err(_)) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Err(_) => {
            return wire_response(unavailable_response(
                timeout_request_id,
                "database exchange exceeded its deadline",
                media_type,
            ));
        }
    };
    wire_response(response)
}

fn wire_response(response: aep_client::wire::Response) -> AxumResponse {
    let mut outgoing = AxumResponse::new(Body::from(response.body));
    *outgoing.status_mut() =
        StatusCode::from_u16(response.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    for (name, value) in response.headers {
        if let (Ok(name), Ok(value)) = (
            header::HeaderName::try_from(name),
            HeaderValue::try_from(value),
        ) {
            outgoing.headers_mut().insert(name, value);
        }
    }
    add_security_headers(&mut outgoing);
    outgoing
}

fn request_metadata() -> RequestMetadata {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let request_id: RequestId = Uuid::now_v7()
        .to_string()
        .parse()
        .expect("server request identities are non-empty");
    RequestMetadata {
        request_id,
        received_at: Timestamp::from_epoch_millis(u64::try_from(millis).unwrap_or(u64::MAX)),
    }
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("SIGTERM handler installs");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

async fn healthy() -> AxumResponse {
    let mut response = StatusCode::OK.into_response();
    add_security_headers(&mut response);
    response
}

async fn openapi(State(state): State<Arc<RuntimeState>>) -> AxumResponse {
    let mut response = AxumResponse::new(Body::from(state.openapi.to_vec()));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    add_security_headers(&mut response);
    response
}

fn add_security_headers(response: &mut AxumResponse) {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
}

fn flatten_server_result(
    result: Result<Result<(), std::io::Error>, tokio::task::JoinError>,
) -> Result<(), Box<dyn std::error::Error>> {
    result??;
    Ok(())
}

fn probe(arguments: &ProbeArgs) -> Result<(), Box<dyn std::error::Error>> {
    let timeout = Duration::from_millis(arguments.timeout_ms);
    if timeout.is_zero() {
        return Err("probe timeout must be greater than zero".into());
    }
    let mut stream = TcpStream::connect_timeout(&arguments.address, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let path = if arguments.readiness {
        "/readyz"
    } else {
        "/livez"
    };
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        arguments.address
    )?;
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes)?;
    let response = String::from_utf8(bytes)?;
    let status = response.lines().next().unwrap_or_default();
    if !status.contains(" 200 ") {
        return Err(format!("probe failed: {status}").into());
    }
    Ok(())
}

trait IntoResponse {
    fn into_response(self) -> AxumResponse;
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> AxumResponse {
        let mut response = AxumResponse::new(Body::empty());
        *response.status_mut() = self;
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn identity_authority(headers: axum::http::HeaderMap) -> AxumResponse {
        assert_eq!(
            headers
                .get("x-b10x-audience")
                .and_then(|value| value.to_str().ok()),
            Some("urn:b10x:aep-service")
        );
        let mut response = AxumResponse::new(Body::from(
            r#"{"iss":"https://identity.example","sub":"alice","aud":"urn:b10x:aep-service","exp":1900000000,"email":null,"tenant_id":"tenant-one","groups":["engineer"]}"#,
        ));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
        response
            .headers_mut()
            .insert(PRAGMA, HeaderValue::from_static("no-cache"));
        response
    }

    #[test]
    fn transport_request_identities_are_uuid_version_seven_values() {
        let metadata = request_metadata();
        let parsed = Uuid::parse_str(metadata.request_id.as_ref()).unwrap();
        assert_eq!(parsed.get_version(), Some(uuid::Version::SortRand));
    }

    #[test]
    fn non_loopback_development_authentication_requires_the_named_override() {
        let arguments = Cli::try_parse_from([
            "aep-service",
            "serve",
            "--bind",
            "0.0.0.0:8080",
            "--realm",
            "realm",
            "--workspace",
            "workspace",
            "--schema",
            "realm",
            "--definitions",
            ".",
            "--definition-digest",
            "digest",
        ])
        .unwrap();
        let Command::Serve(arguments) = arguments.command else {
            panic!("serve arguments");
        };
        assert!(!arguments.bind.ip().is_loopback());
        assert!(!arguments.allow_insecure_dev_listener);
        assert_eq!(
            validate_listener(
                arguments.bind,
                false,
                arguments.allow_insecure_dev_listener
            ),
            Err("the development verifier refuses a non-loopback listener unless --allow-insecure-dev-listener is explicit")
        );
        assert_eq!(validate_listener(arguments.bind, false, true), Ok(()));
        assert_eq!(validate_listener(arguments.bind, true, false), Ok(()));
    }

    #[test]
    fn definition_digest_is_an_explicit_read_only_subcommand() {
        let arguments =
            Cli::try_parse_from(["aep-service", "definitions", "digest", "--path", "../aep"])
                .unwrap();
        let Command::Definitions(arguments) = arguments.command else {
            panic!("definition command");
        };
        let DefinitionsCommand::Digest { path } = arguments.command;
        assert_eq!(path, PathBuf::from("../aep"));
    }

    #[tokio::test]
    async fn hosted_identity_derives_the_actor_and_refuses_another_tenant() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Identity listener");
        let address = listener.local_addr().expect("Identity address");
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new().route("/v1/session-authority", get(identity_authority)),
            )
            .await
        });
        let client =
            IdentityAuthorityClient::new(&format!("http://{address}/"), "urn:b10x:aep-service")
                .expect("Identity client");
        let verifier = IdentityVerifier {
            client: client.clone(),
            tenant: Arc::from("tenant-one"),
            realm: Arc::from("engineering"),
            workspace: Arc::from("central"),
        };
        let principal = verifier
            .verify(Some("Bearer synthetic-session"))
            .await
            .expect("verified principal");
        assert!(principal.authority().is_human());
        assert_ne!(principal.authority().name(), "alice");
        assert!(principal.authorizes("engineering", "central"));

        let refused = IdentityVerifier {
            client,
            tenant: Arc::from("another-tenant"),
            realm: Arc::from("engineering"),
            workspace: Arc::from("central"),
        }
        .verify(Some("Bearer synthetic-session"))
        .await
        .expect_err("cross-tenant session is refused");
        assert_eq!(
            refused.reason(),
            "Identity tenant is outside this AEP authority"
        );
        server.abort();
        let _ = server.await;
    }

    #[tokio::test]
    async fn database_preparation_runs_outside_the_async_runtime() {
        let error = prepare_authority(
            "postgresql://127.0.0.1:1/unavailable".to_owned(),
            "realm".to_owned(),
            "workspace".to_owned(),
            "aep_runtime_boundary".to_owned(),
            LifecycleRegistry::new(),
        )
        .await
        .expect_err("the deliberately unavailable database is refused");

        assert_eq!(error, "the PostgreSQL authority is unavailable");
    }
}
