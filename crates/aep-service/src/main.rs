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
use tokio::sync::Semaphore;
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
    /// Inspects an immutable EP definition tree for deployment.
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
        /// Root of the EP definition tree.
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
    /// Root of the immutable EP definition tree.
    #[arg(long)]
    definitions: PathBuf,
    /// Expected lowercase SHA-256 of sorted definition paths and bytes.
    #[arg(long)]
    definition_digest: String,
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

struct RuntimeState {
    service: Arc<AepHttpService<DevelopmentVerifier, PostgresAuthority>>,
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
    validate_listener(arguments.bind, arguments.allow_insecure_dev_listener)?;
    if !arguments.bind.ip().is_loopback() {
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
    let database_url = env::var(&arguments.database_url_env).map_err(|_| {
        format!(
            "{} must contain the PostgreSQL URL",
            arguments.database_url_env
        )
    })?;
    let raw_token = env::var(&arguments.dev_token_env).map_err(|_| {
        format!(
            "{} must contain the development token",
            arguments.dev_token_env
        )
    })?;
    if raw_token.trim().is_empty() {
        return Err("the development bearer token must not be empty".into());
    }
    let bundle =
        aep_project::load_pinned_bundle(&arguments.definitions, &arguments.definition_digest)?;
    let lifecycles = bundle.registry.lifecycles().clone();

    let authority = PostgresAuthority::new(
        database_url,
        arguments.realm.clone(),
        arguments.workspace.clone(),
        arguments.schema,
        lifecycles,
    )?;
    authority.prepare()?;
    let principal = VerifiedPrincipal::new(
        arguments.dev_authority.parse()?,
        None,
        arguments.realm,
        [arguments.workspace],
        ["developer".to_owned()],
        None,
    );
    let verifier = DevelopmentVerifier {
        authorization: Arc::from(format!("Bearer {raw_token}")),
        principal,
    };
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

fn validate_listener(bind: SocketAddr, allow_insecure: bool) -> Result<(), &'static str> {
    if !bind.ip().is_loopback() && !allow_insecure {
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
            validate_listener(arguments.bind, arguments.allow_insecure_dev_listener),
            Err("the development verifier refuses a non-loopback listener unless --allow-insecure-dev-listener is explicit")
        );
        assert_eq!(validate_listener(arguments.bind, true), Ok(()));
    }

    #[test]
    fn definition_digest_is_an_explicit_read_only_subcommand() {
        let arguments = Cli::try_parse_from([
            "aep-service",
            "definitions",
            "digest",
            "--path",
            "../engineering-protocols",
        ])
        .unwrap();
        let Command::Definitions(arguments) = arguments.command else {
            panic!("definition command");
        };
        let DefinitionsCommand::Digest { path } = arguments.command;
        assert_eq!(path, PathBuf::from("../engineering-protocols"));
    }
}
