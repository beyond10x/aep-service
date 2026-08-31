//! Runnable single-realm AEP authority with an explicit loopback-only development verifier.

use std::collections::BTreeMap;
use std::env;
use std::future::ready;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use aep_client::wire::{Method, Request};
use aep_contract::testing::block_on;
use aep_domain::ids::RequestId;
use aep_domain::time::Timestamp;
use aep_service_auth::{AuthenticationError, CredentialVerifier, VerifiedPrincipal};
use aep_service_http::{AepHttpService, RequestMetadata};
use aep_service_postgres::PostgresAuthority;
use axum::body::{to_bytes, Body};
use axum::extract::{Request as AxumRequest, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::Response as AxumResponse;
use axum::routing::{any, get};
use axum::Router;
use clap::{Parser, Subcommand};
use tokio::sync::Semaphore;

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
    Serve(ServeArgs),
}

#[derive(Debug, clap::Args)]
struct ServeArgs {
    /// Listener address; development authentication requires a loopback address.
    #[arg(long, default_value = "127.0.0.1:8080")]
    bind: SocketAddr,
    /// Environment variable containing the PostgreSQL URL.
    #[arg(long, default_value = "AEP_DATABASE_URL")]
    database_url_env: String,
    /// Globally unique realm served by this process.
    #[arg(long)]
    realm: String,
    /// One workspace served by this process.
    #[arg(long)]
    workspace: String,
    /// PostgreSQL schema dedicated to the realm.
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Cli { command } = Cli::parse();
    match command {
        Command::Serve(arguments) => serve(arguments).await,
    }
}

async fn serve(arguments: ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    if !arguments.bind.ip().is_loopback() {
        return Err("the development verifier refuses a non-loopback listener".into());
    }
    if arguments.database_concurrency == 0 {
        return Err("database concurrency must be greater than zero".into());
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
    });
    let application = Router::new()
        .route("/livez", get(|| async { StatusCode::OK }))
        .route("/readyz", get(|| async { StatusCode::OK }))
        .fallback(any(dispatch))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(arguments.bind).await?;
    axum::serve(listener, application)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn dispatch(State(state): State<Arc<RuntimeState>>, request: AxumRequest) -> AxumResponse {
    let Ok(_permit) = state.database_slots.acquire().await else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let (parts, body) = request.into_parts();
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
    let metadata = request_metadata();
    let service = Arc::clone(&state.service);
    let Ok(response) =
        tokio::task::spawn_blocking(move || block_on(service.handle(exchange, metadata))).await
    else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
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
    outgoing
}

fn request_metadata() -> RequestMetadata {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let request_id: RequestId = format!("request-{millis}-{sequence}")
        .parse()
        .expect("server request identities are non-empty");
    RequestMetadata {
        request_id,
        received_at: Timestamp::from_epoch_millis(u64::try_from(millis).unwrap_or(u64::MAX)),
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
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
