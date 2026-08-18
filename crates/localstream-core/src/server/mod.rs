use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use axum::extract::rejection::JsonRejection;
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Extension, Path, Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use tokio_util::io::ReaderStream;

use crate::{
    auth::{AuthError, PeerCapability},
    streaming::{range::parse_single_range, StreamingError},
    LibraryScan, LocalStreamCore,
};

use crate::node_identity::{LeafIssuanceError, NodeIdentity, TlsConfigError};

const MAX_HTTPS_CONNECTIONS: usize = 64;
const TLS_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_WEB_ASSET_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct BrowserAssets {
    root: Arc<PathBuf>,
}

impl BrowserAssets {
    pub fn from_directory(root: impl Into<PathBuf>) -> Result<Self, BrowserAssetsError> {
        let root =
            std::fs::canonicalize(root.into()).map_err(|_| BrowserAssetsError::Unavailable)?;
        if !root.is_dir() {
            return Err(BrowserAssetsError::Unavailable);
        }
        let index = root.join("index.html");
        let metadata = std::fs::metadata(index).map_err(|_| BrowserAssetsError::Unavailable)?;
        if !metadata.is_file() || metadata.len() > MAX_WEB_ASSET_BYTES {
            return Err(BrowserAssetsError::Unavailable);
        }
        Ok(Self {
            root: Arc::new(root),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BrowserAssetsError {
    #[error("the browser application assets are unavailable")]
    Unavailable,
}

#[derive(Debug, Clone)]
pub struct HttpsRequestPolicy {
    allowed_hosts: Arc<[String]>,
    allowed_origins: Arc<[String]>,
}

impl HttpsRequestPolicy {
    fn for_loopback(address: SocketAddr) -> Self {
        let port = address.port();
        Self {
            allowed_hosts: Arc::from([
                format!("localhost:{port}"),
                format!("{}:{port}", address.ip()),
            ]),
            allowed_origins: Arc::from([
                format!("https://localhost:{port}"),
                format!("https://{}:{port}", address.ip()),
            ]),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct HttpsLimits {
    max_connections: usize,
    handshake_timeout: Duration,
}

impl Default for HttpsLimits {
    fn default() -> Self {
        Self {
            max_connections: MAX_HTTPS_CONNECTIONS,
            handshake_timeout: TLS_HANDSHAKE_TIMEOUT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub base_url: String,
    pub bind_scope: &'static str,
    pub lan_available: bool,
}

#[derive(Debug)]
pub struct ServerHandle {
    info: ServerInfo,
    shutdown: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
}

impl ServerHandle {
    #[must_use]
    pub fn info(&self) -> ServerInfo {
        self.info.clone()
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.task.abort();
    }
}

#[derive(Debug)]
pub struct HttpsServerHandle {
    info: ServerInfo,
    shutdown: Option<tokio::sync::watch::Sender<bool>>,
    task: Option<JoinHandle<()>>,
}

impl HttpsServerHandle {
    #[must_use]
    pub fn info(&self) -> ServerInfo {
        self.info.clone()
    }

    pub async fn shutdown(mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(true);
        }
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
    }
}

impl Drop for HttpsServerHandle {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(true);
        }
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HttpsServerError {
    #[error("the HTTPS listener is unavailable")]
    ListenerUnavailable,
    #[error("the HTTPS node certificate is unavailable")]
    IdentityUnavailable,
    #[error("the HTTPS TLS configuration is unavailable")]
    TlsUnavailable,
}

impl From<LeafIssuanceError> for HttpsServerError {
    fn from(_: LeafIssuanceError) -> Self {
        Self::IdentityUnavailable
    }
}

impl From<TlsConfigError> for HttpsServerError {
    fn from(_: TlsConfigError) -> Self {
        Self::TlsUnavailable
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    service: &'static str,
    version: &'static str,
    status: &'static str,
    api_version: &'static str,
    lan_available: bool,
}

#[derive(Debug, Serialize)]
struct ApiErrorEnvelope {
    error: ApiErrorBody,
}

#[derive(Debug, Serialize)]
struct ApiErrorBody {
    code: &'static str,
    message: &'static str,
}

struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
    content_range: Option<String>,
    retry_after_seconds: Option<u64>,
}

impl ApiError {
    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: "The request could not be completed.",
            content_range: None,
            retry_after_seconds: None,
        }
    }

    fn media_not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "media_not_found",
            message: "The requested media is unavailable.",
            content_range: None,
            retry_after_seconds: None,
        }
    }

    fn range_not_satisfiable(size: u64) -> Self {
        Self {
            status: StatusCode::RANGE_NOT_SATISFIABLE,
            code: "range_not_satisfiable",
            message: "The requested byte range is not satisfiable.",
            content_range: Some(format!("bytes */{size}")),
            retry_after_seconds: None,
        }
    }

    fn invalid_pairing_request() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_pairing_request",
            message: "The pairing request is invalid.",
            content_range: None,
            retry_after_seconds: None,
        }
    }

    fn pairing_claim_failed() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "pairing_claim_failed",
            message: "The pairing claim could not be completed.",
            content_range: None,
            retry_after_seconds: None,
        }
    }

    fn rate_limited(retry_after_seconds: u64) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "rate_limited",
            message: "Too many pairing attempts. Try again later.",
            content_range: None,
            retry_after_seconds: Some(retry_after_seconds),
        }
    }

    fn payload_too_large() -> Self {
        Self {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            code: "payload_too_large",
            message: "The request body is too large.",
            content_range: None,
            retry_after_seconds: None,
        }
    }

    fn forbidden_request() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "forbidden_request",
            message: "The request origin is not allowed.",
            content_range: None,
            retry_after_seconds: None,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let content_range = self.content_range;
        let mut response = (
            self.status,
            Json(ApiErrorEnvelope {
                error: ApiErrorBody {
                    code: self.code,
                    message: self.message,
                },
            }),
        )
            .into_response();
        if let Some(value) = content_range {
            if let Ok(value) = HeaderValue::from_str(&value) {
                response.headers_mut().insert(header::CONTENT_RANGE, value);
            }
        }
        if let Some(seconds) = self.retry_after_seconds {
            if let Ok(value) = HeaderValue::from_str(&seconds.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
        }
        response
    }
}

pub fn router(core: Arc<LocalStreamCore>) -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/library", get(current_library))
        .route("/api/v1/media/{id}/stream", get(stream_media))
        .with_state(core)
}

pub fn authenticated_router(core: Arc<LocalStreamCore>) -> Router {
    let protected_routes = Router::new()
        .route("/api/v1/library", get(current_library))
        .route("/api/v1/media/{id}/stream", get(stream_media))
        .route_layer(middleware::from_fn_with_state(
            Arc::clone(&core),
            require_library_read,
        ));

    Router::new()
        .route("/api/v1/health", get(health))
        .merge(protected_routes)
        .with_state(core)
}

pub fn encrypted_router(core: Arc<LocalStreamCore>, policy: Arc<HttpsRequestPolicy>) -> Router {
    encrypted_api_router(core, Arc::clone(&policy))
}

pub fn encrypted_router_with_assets(
    core: Arc<LocalStreamCore>,
    policy: Arc<HttpsRequestPolicy>,
    assets: BrowserAssets,
) -> Router {
    encrypted_api_router(core, Arc::clone(&policy))
        .route("/api", axum::routing::any(api_not_found))
        .route("/api/{*path}", axum::routing::any(api_not_found))
        .fallback(serve_browser_asset)
        .layer(middleware::from_fn(require_https_host))
        .layer(Extension(assets))
        .layer(Extension(policy))
}

fn encrypted_api_router(core: Arc<LocalStreamCore>, policy: Arc<HttpsRequestPolicy>) -> Router {
    let begin = Router::new()
        .route("/api/v1/pairing/requests", post(begin_pairing))
        .route_layer(middleware::from_fn_with_state(
            Arc::clone(&core),
            limit_pairing_begin,
        ))
        .route_layer(middleware::from_fn(require_pairing_origin))
        .with_state(Arc::clone(&core));
    let claim = Router::new()
        .route("/api/v1/pairing/claims", post(claim_pairing))
        .route_layer(middleware::from_fn_with_state(
            Arc::clone(&core),
            limit_pairing_claim,
        ))
        .route_layer(middleware::from_fn(require_pairing_origin))
        .with_state(Arc::clone(&core));
    let browser_claim = Router::new()
        .route(
            "/api/v1/pairing/browser-claims",
            post(claim_browser_pairing),
        )
        .route_layer(middleware::from_fn_with_state(
            Arc::clone(&core),
            limit_pairing_claim,
        ))
        .route_layer(middleware::from_fn(require_pairing_origin))
        .with_state(Arc::clone(&core));
    authenticated_router(Arc::clone(&core))
        .merge(begin)
        .merge(claim)
        .merge(browser_claim)
        .layer(DefaultBodyLimit::max(2 * 1024))
        .layer(middleware::from_fn(require_https_host))
        .layer(Extension(policy))
}

async fn api_not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn serve_browser_asset(
    Extension(assets): Extension<BrowserAssets>,
    method: Method,
    uri: Uri,
) -> Response {
    if !matches!(method, Method::GET | Method::HEAD) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(relative) = safe_asset_path(uri.path()) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let requested_asset = !relative.as_os_str().is_empty();
    let immutable = relative.starts_with("assets");
    let candidate = if requested_asset {
        assets.root.join(&relative)
    } else {
        assets.root.join("index.html")
    };
    match read_asset(&assets, candidate).await {
        Ok(bytes) => {
            let response_path = if requested_asset {
                relative.as_path()
            } else {
                std::path::Path::new("index.html")
            };
            asset_response(response_path, bytes, method == Method::HEAD, immutable)
        }
        Err(_) if !immutable => {
            let index = assets.root.join("index.html");
            match read_asset(&assets, index).await {
                Ok(bytes) => asset_response(
                    std::path::Path::new("index.html"),
                    bytes,
                    method == Method::HEAD,
                    false,
                ),
                Err(_) => StatusCode::NOT_FOUND.into_response(),
            }
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

fn safe_asset_path(path: &str) -> Option<PathBuf> {
    let encoded = path.strip_prefix('/')?;
    let decoded = decode_url_path(encoded)?;
    if decoded.contains(['\\', '\0']) || decoded.split('/').any(|part| part == ".." || part == ".")
    {
        return None;
    }
    let mut relative = PathBuf::new();
    for segment in decoded.split('/').filter(|segment| !segment.is_empty()) {
        relative.push(segment);
    }
    Some(relative)
}

fn decode_url_path(encoded: &str) -> Option<String> {
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = *bytes.get(index + 1)?;
            let low = *bytes.get(index + 2)?;
            let value = hex_value(high)? * 16 + hex_value(low)?;
            if matches!(value, b'/' | b'\\' | b'\0') {
                return None;
            }
            decoded.push(value);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

async fn read_asset(assets: &BrowserAssets, candidate: PathBuf) -> Result<Vec<u8>, ()> {
    let candidate = tokio::fs::canonicalize(candidate).await.map_err(|_| ())?;
    if !candidate.starts_with(assets.root.as_ref()) {
        return Err(());
    }
    let metadata = tokio::fs::metadata(&candidate).await.map_err(|_| ())?;
    if !metadata.is_file() || metadata.len() > MAX_WEB_ASSET_BYTES {
        return Err(());
    }
    let file = tokio::fs::File::open(candidate).await.map_err(|_| ())?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_WEB_ASSET_BYTES + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| ())?;
    if bytes.len() as u64 != metadata.len() {
        return Err(());
    }
    Ok(bytes)
}

fn asset_response(path: &std::path::Path, bytes: Vec<u8>, head: bool, immutable: bool) -> Response {
    let length = bytes.len();
    let body = if head {
        Body::empty()
    } else {
        Body::from(bytes)
    };
    let mut response = Response::new(body);
    let content_type = match path.extension().and_then(|extension| extension.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json" | "map") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    };
    if let Ok(value) = HeaderValue::from_str(content_type) {
        response.headers_mut().insert(header::CONTENT_TYPE, value);
    }
    if let Ok(value) = HeaderValue::from_str(&length.to_string()) {
        response.headers_mut().insert(header::CONTENT_LENGTH, value);
    }
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(if immutable {
            "public, max-age=31536000, immutable"
        } else {
            "no-cache, no-store, must-revalidate"
        }),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}

async fn require_https_host(
    Extension(policy): Extension<Arc<HttpsRequestPolicy>>,
    request: Request,
    next: Next,
) -> Response {
    let mut hosts = request.headers().get_all(header::HOST).iter();
    let valid = hosts
        .next()
        .and_then(|value| value.to_str().ok())
        .map(str::to_ascii_lowercase)
        .is_some_and(|host| policy.allowed_hosts.iter().any(|allowed| allowed == &host))
        && hosts.next().is_none();
    if valid {
        next.run(request).await
    } else {
        ApiError::forbidden_request().into_response()
    }
}

async fn require_pairing_origin(
    Extension(policy): Extension<Arc<HttpsRequestPolicy>>,
    request: Request,
    next: Next,
) -> Response {
    let mut origins = request.headers().get_all(header::ORIGIN).iter();
    let origin_valid = origins
        .next()
        .and_then(|value| value.to_str().ok())
        .is_some_and(|origin| {
            policy
                .allowed_origins
                .iter()
                .any(|allowed| allowed == origin)
        })
        && origins.next().is_none();
    let mut fetch_sites = request
        .headers()
        .get_all(header::HeaderName::from_static("sec-fetch-site"))
        .iter();
    let fetch_valid = match fetch_sites.next() {
        Some(value) => {
            value
                .to_str()
                .ok()
                .is_some_and(|value| matches!(value, "same-origin" | "none"))
                && fetch_sites.next().is_none()
        }
        None => true,
    };
    if origin_valid && fetch_valid {
        next.run(request).await
    } else {
        ApiError::forbidden_request().into_response()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BeginPairingBody {
    display_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BeginPairingResponse {
    request_id: String,
    verification_code: String,
    claim_secret: String,
    expires_in_seconds: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ClaimPairingBody {
    request_id: String,
    claim_secret: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClaimPairingResponse {
    peer: crate::auth::TrustedPeer,
    bearer_token: String,
}

async fn limit_pairing_begin(
    State(core): State<Arc<LocalStreamCore>>,
    Extension(remote): Extension<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    limit_pairing_attempt(
        core,
        remote,
        crate::auth::PairingAttemptKind::Begin,
        request,
        next,
    )
    .await
}

async fn limit_pairing_claim(
    State(core): State<Arc<LocalStreamCore>>,
    Extension(remote): Extension<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    limit_pairing_attempt(
        core,
        remote,
        crate::auth::PairingAttemptKind::Claim,
        request,
        next,
    )
    .await
}

async fn limit_pairing_attempt(
    core: Arc<LocalStreamCore>,
    remote: SocketAddr,
    kind: crate::auth::PairingAttemptKind,
    request: Request,
    next: Next,
) -> Response {
    match core.check_pairing_attempt(kind, remote) {
        crate::auth::RateLimitDecision::Allowed => next.run(request).await,
        crate::auth::RateLimitDecision::Limited {
            retry_after_seconds,
        } => ApiError::rate_limited(retry_after_seconds).into_response(),
    }
}

async fn begin_pairing(
    State(core): State<Arc<LocalStreamCore>>,
    body: Result<Json<BeginPairingBody>, JsonRejection>,
) -> Result<Json<BeginPairingResponse>, ApiError> {
    let Json(body) = body.map_err(|rejection| {
        if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ApiError::payload_too_large()
        } else {
            ApiError::invalid_pairing_request()
        }
    })?;
    let receipt = core
        .begin_pairing(&body.display_name)
        .map_err(|error| match error {
            crate::auth::PairingError::Auth(crate::auth::AuthError::InvalidDisplayName) => {
                ApiError::invalid_pairing_request()
            }
            crate::auth::PairingError::CapacityReached => ApiError {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "pairing_unavailable",
                message: "Pairing is temporarily unavailable.",
                content_range: None,
                retry_after_seconds: None,
            },
            _ => ApiError::internal(),
        })?;
    Ok(Json(BeginPairingResponse {
        request_id: receipt.request_id,
        verification_code: receipt.verification_code,
        claim_secret: receipt.claim_secret,
        expires_in_seconds: receipt.expires_in_seconds,
    }))
}

async fn claim_pairing(
    State(core): State<Arc<LocalStreamCore>>,
    body: Result<Json<ClaimPairingBody>, JsonRejection>,
) -> Result<Json<ClaimPairingResponse>, ApiError> {
    let Json(body) = body.map_err(|rejection| {
        if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ApiError::payload_too_large()
        } else {
            ApiError::pairing_claim_failed()
        }
    })?;
    let credential = core
        .claim_pairing(&body.request_id, &body.claim_secret)
        .map_err(|error| match error {
            crate::auth::PairingError::Auth(
                crate::auth::AuthError::RandomnessUnavailable | crate::auth::AuthError::Unavailable,
            )
            | crate::auth::PairingError::Unavailable => ApiError::internal(),
            _ => ApiError::pairing_claim_failed(),
        })?;
    Ok(Json(ClaimPairingResponse {
        peer: credential.peer,
        bearer_token: credential.bearer_token,
    }))
}

async fn claim_browser_pairing(
    State(core): State<Arc<LocalStreamCore>>,
    body: Result<Json<ClaimPairingBody>, JsonRejection>,
) -> Result<Response, ApiError> {
    let Json(body) = body.map_err(|rejection| {
        if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ApiError::payload_too_large()
        } else {
            ApiError::pairing_claim_failed()
        }
    })?;
    let session = core
        .claim_browser_pairing(&body.request_id, &body.claim_secret)
        .map_err(|error| match error {
            crate::auth::PairingError::Auth(
                crate::auth::AuthError::RandomnessUnavailable | crate::auth::AuthError::Unavailable,
            )
            | crate::auth::PairingError::Unavailable => ApiError::internal(),
            _ => ApiError::pairing_claim_failed(),
        })?;
    let cookie = format!(
        "{}={}; Path=/; Max-Age={}; HttpOnly; Secure; SameSite=Strict",
        crate::auth::SESSION_COOKIE_NAME,
        session.session_token,
        session.expires_in_seconds,
    );
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|_| ApiError::internal())?,
    );
    Ok(response)
}

pub async fn start_local_server(core: Arc<LocalStreamCore>) -> std::io::Result<ServerHandle> {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let address = listener.local_addr()?;
    let (shutdown, shutdown_receiver) = oneshot::channel();
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, router(core))
            .with_graceful_shutdown(async move {
                let _ = shutdown_receiver.await;
            })
            .await;
    });

    Ok(ServerHandle {
        info: server_info(address),
        shutdown: Some(shutdown),
        task,
    })
}

pub async fn start_loopback_https_server(
    core: Arc<LocalStreamCore>,
    identity: &NodeIdentity,
) -> Result<HttpsServerHandle, HttpsServerError> {
    start_loopback_https_server_with_options(core, identity, HttpsLimits::default(), None).await
}

pub async fn start_loopback_https_server_with_assets(
    core: Arc<LocalStreamCore>,
    identity: &NodeIdentity,
    assets: BrowserAssets,
) -> Result<HttpsServerHandle, HttpsServerError> {
    start_loopback_https_server_with_options(core, identity, HttpsLimits::default(), Some(assets))
        .await
}

#[cfg(test)]
async fn start_loopback_https_server_with_limits(
    core: Arc<LocalStreamCore>,
    identity: &NodeIdentity,
    limits: HttpsLimits,
) -> Result<HttpsServerHandle, HttpsServerError> {
    start_loopback_https_server_with_options(core, identity, limits, None).await
}

async fn start_loopback_https_server_with_options(
    core: Arc<LocalStreamCore>,
    identity: &NodeIdentity,
    limits: HttpsLimits,
    assets: Option<BrowserAssets>,
) -> Result<HttpsServerHandle, HttpsServerError> {
    if limits.max_connections == 0 || limits.handshake_timeout.is_zero() {
        return Err(HttpsServerError::ListenerUnavailable);
    }
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|_| HttpsServerError::ListenerUnavailable)?;
    let address = listener
        .local_addr()
        .map_err(|_| HttpsServerError::ListenerUnavailable)?;
    let leaf = identity.issue_server_leaf(&[
        "localhost".to_owned(),
        std::net::Ipv4Addr::LOCALHOST.to_string(),
        std::net::Ipv6Addr::LOCALHOST.to_string(),
    ])?;
    let tls_config = Arc::new(leaf.into_server_config()?);
    let tls_acceptor = tokio_rustls::TlsAcceptor::from(tls_config);
    let request_policy = Arc::new(HttpsRequestPolicy::for_loopback(address));
    let connection_permits = Arc::new(tokio::sync::Semaphore::new(limits.max_connections));
    let (shutdown, mut shutdown_receiver) = tokio::sync::watch::channel(false);
    let task = tokio::spawn(async move {
        let graceful = hyper_util::server::graceful::GracefulShutdown::new();
        loop {
            tokio::select! {
                accepted = listener.accept() => {
                    let Ok((socket, peer)) = accepted else { break };
                    let Ok(permit) = Arc::clone(&connection_permits).try_acquire_owned() else {
                        drop(socket);
                        continue;
                    };
                    let acceptor = tls_acceptor.clone();
                    let assets = assets.clone();
                    let request_policy = Arc::clone(&request_policy);
                    let app = match assets {
                        Some(assets) => encrypted_router_with_assets(
                            Arc::clone(&core),
                            request_policy,
                            assets,
                        ),
                        None => encrypted_router(Arc::clone(&core), request_policy),
                    };
                    let service = hyper_util::service::TowerToHyperService::new(
                        app.layer(Extension(peer)),
                    );
                    let watcher = graceful.watcher();
                    let mut connection_shutdown = shutdown_receiver.clone();
                    tokio::spawn(async move {
                        let _permit = permit;
                        let tls_stream = tokio::select! {
                            result = tokio::time::timeout(limits.handshake_timeout, acceptor.accept(socket)) => match result {
                                Ok(Ok(stream)) => stream,
                                Err(_) => return,
                                Ok(Err(_)) => return,
                            },
                            changed = connection_shutdown.changed() => {
                                let _ = changed;
                                return;
                            }
                        };
                        let io = hyper_util::rt::TokioIo::new(tls_stream);
                        let connection =
                            hyper::server::conn::http1::Builder::new().serve_connection(io, service);
                        let _ = watcher.watch(connection).await;
                    });
                }
                changed = shutdown_receiver.changed() => {
                    if changed.is_err() || *shutdown_receiver.borrow() {
                        break;
                    }
                }
            }
        }
        graceful.shutdown().await;
    });

    Ok(HttpsServerHandle {
        info: ServerInfo {
            base_url: format!("https://{address}"),
            bind_scope: "loopback",
            lan_available: false,
        },
        shutdown: Some(shutdown),
        task: Some(task),
    })
}

fn server_info(address: SocketAddr) -> ServerInfo {
    ServerInfo {
        base_url: format!("http://{address}"),
        bind_scope: "loopback",
        lan_available: false,
    }
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "LocalStream",
        version: env!("CARGO_PKG_VERSION"),
        status: "ok",
        api_version: "v1",
        lan_available: false,
    })
}

async fn require_library_read(
    State(core): State<Arc<LocalStreamCore>>,
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();
    let authorization_present = headers.contains_key(header::AUTHORIZATION);
    let session_cookie = strict_session_cookie(headers);
    let session_present = session_cookie.is_some()
        || headers.get_all(header::COOKIE).iter().any(|value| {
            value
                .as_bytes()
                .windows(crate::auth::SESSION_COOKIE_NAME.len())
                .any(|window| window == crate::auth::SESSION_COOKIE_NAME.as_bytes())
        });
    let authenticated = if authorization_present && session_present {
        Err(AuthError::InvalidCredential)
    } else if authorization_present {
        core.authenticate_peer(strict_bearer_token(headers))
    } else {
        core.authenticate_browser_session(session_cookie)
    };
    match authenticated {
        Ok(peer) if peer.capability == PeerCapability::LibraryRead => {
            request.extensions_mut().insert(peer);
            next.run(request).await
        }
        Err(
            AuthError::MissingCredential
            | AuthError::InvalidCredential
            | AuthError::RevokedCredential,
        ) => unauthorized_response(),
        Ok(_)
        | Err(
            AuthError::InvalidDisplayName
            | AuthError::RandomnessUnavailable
            | AuthError::Unavailable,
        ) => ApiError::internal().into_response(),
    }
}

fn strict_session_cookie(headers: &HeaderMap) -> Option<&str> {
    let mut cookie_headers = headers.get_all(header::COOKIE).iter();
    let header = cookie_headers.next()?;
    if cookie_headers.next().is_some() {
        return None;
    }
    let header = header.to_str().ok()?;
    let mut session = None;
    for pair in header.split(';') {
        let (name, value) = pair.trim().split_once('=')?;
        if name == crate::auth::SESSION_COOKIE_NAME && session.replace(value).is_some() {
            return None;
        }
    }
    session.filter(|value| !value.is_empty())
}

fn strict_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let mut values = headers.get_all(header::AUTHORIZATION).iter();
    let value = values.next()?;
    if values.next().is_some() {
        return None;
    }
    value.to_str().ok()?.strip_prefix("Bearer ")
}

fn unauthorized_response() -> Response {
    let mut response = (
        StatusCode::UNAUTHORIZED,
        Json(ApiErrorEnvelope {
            error: ApiErrorBody {
                code: "unauthorized",
                message: "Authentication is required.",
            },
        }),
    )
        .into_response();
    response
        .headers_mut()
        .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
    response
}

async fn current_library(
    State(core): State<Arc<LocalStreamCore>>,
) -> Result<Json<Option<LibraryScan>>, ApiError> {
    core.current_library()
        .map(Json)
        .map_err(|_| ApiError::internal())
}

async fn stream_media(
    State(core): State<Arc<LocalStreamCore>>,
    Path(media_id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let mut source = core
        .open_direct_play(&media_id)
        .await
        .map_err(|error| match error {
            StreamingError::NotFound => ApiError::media_not_found(),
            StreamingError::Busy => ApiError {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "stream_capacity_reached",
                message: "Streaming is temporarily unavailable.",
                content_range: None,
                retry_after_seconds: None,
            },
            StreamingError::OutsideApprovedLibrary | StreamingError::Unavailable => {
                ApiError::internal()
            }
        })?;
    let requested_range = headers
        .get(header::RANGE)
        .map(|value| {
            value
                .to_str()
                .map_err(|_| ApiError::range_not_satisfiable(source.size))
        })
        .transpose()?
        .map(|value| {
            parse_single_range(value, source.size)
                .map_err(|_| ApiError::range_not_satisfiable(source.size))
        })
        .transpose()?;

    let (status, start, end) = requested_range.map_or(
        (StatusCode::OK, 0, source.size.saturating_sub(1)),
        |range| (StatusCode::PARTIAL_CONTENT, range.start, range.end),
    );
    let length = if source.size == 0 { 0 } else { end - start + 1 };
    source
        .file
        .seek(std::io::SeekFrom::Start(start))
        .await
        .map_err(|_| ApiError::internal())?;
    let stream = ReaderStream::new(source.file.take(length));

    let mut response = Response::new(Body::from_stream(stream));
    *response.status_mut() = status;
    let response_headers = response.headers_mut();
    response_headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    response_headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(source.content_type),
    );
    response_headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&length.to_string()).map_err(|_| ApiError::internal())?,
    );
    if status == StatusCode::PARTIAL_CONTENT {
        response_headers.insert(
            header::CONTENT_RANGE,
            HeaderValue::from_str(&format!("bytes {start}-{end}/{}", source.size))
                .map_err(|_| ApiError::internal())?,
        );
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::Arc, time::Duration};

    use axum::{
        body::Body,
        extract::Extension,
        http::{header, HeaderMap, HeaderValue, Method, Request, StatusCode},
        middleware,
        routing::get,
        Json, Router,
    };
    use http_body_util::BodyExt;
    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    use tower::ServiceExt;

    use crate::{auth::TrustedPeer, LocalStreamCore};

    use super::{
        authenticated_router, encrypted_router, encrypted_router_with_assets, require_library_read,
        router, start_local_server, start_loopback_https_server,
        start_loopback_https_server_with_assets, start_loopback_https_server_with_limits,
        BrowserAssets, HttpsLimits, HttpsRequestPolicy,
    };

    #[derive(Clone, Default)]
    struct MemoryIdentityStore(Arc<std::sync::Mutex<Option<Vec<u8>>>>);

    impl crate::node_identity::NodeSecretStore for MemoryIdentityStore {
        fn load(&self) -> Result<Option<Vec<u8>>, crate::node_identity::SecretStoreError> {
            Ok(self.0.lock().expect("identity store should lock").clone())
        }

        fn store(&self, secret: &[u8]) -> Result<(), crate::node_identity::SecretStoreError> {
            *self.0.lock().expect("identity store should lock") = Some(secret.to_vec());
            Ok(())
        }

        fn delete(&self) -> Result<(), crate::node_identity::SecretStoreError> {
            *self.0.lock().expect("identity store should lock") = None;
            Ok(())
        }
    }

    fn test_identity() -> crate::node_identity::NodeIdentity {
        crate::node_identity::NodeIdentityService::new(MemoryIdentityStore::default())
            .load_or_create()
            .expect("test identity should generate")
    }

    fn tls_client_config(root_der: &[u8]) -> rustls::ClientConfig {
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(rustls::pki_types::CertificateDer::from(root_der.to_vec()))
            .expect("test root should add");
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let mut config = rustls::ClientConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
            .expect("test protocols should configure")
            .with_root_certificates(roots)
            .with_no_client_auth();
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        config
    }

    async fn https_request(
        address: std::net::SocketAddr,
        root_der: &[u8],
        server_name: &str,
        path: &str,
        bearer: Option<&str>,
    ) -> Result<(StatusCode, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
        let (status, _, body) = https_exchange(
            address,
            root_der,
            server_name,
            hyper::Method::GET,
            path,
            Vec::new(),
            bearer
                .map(|token| vec![(header::AUTHORIZATION, format!("Bearer {token}"))])
                .unwrap_or_default(),
        )
        .await?;
        Ok((status, body))
    }

    async fn https_exchange(
        address: std::net::SocketAddr,
        root_der: &[u8],
        server_name: &str,
        method: hyper::Method,
        path: &str,
        body: Vec<u8>,
        headers: Vec<(header::HeaderName, String)>,
    ) -> Result<(StatusCode, HeaderMap, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
        let tcp = tokio::net::TcpStream::connect(address).await?;
        let connector = tokio_rustls::TlsConnector::from(Arc::new(tls_client_config(root_der)));
        let name = rustls::pki_types::ServerName::try_from(server_name.to_owned())?;
        let tls = connector.connect(name, tcp).await?;
        let io = hyper_util::rt::TokioIo::new(tls);
        let (mut sender, connection) = hyper::client::conn::http1::handshake(io).await?;
        tokio::spawn(async move {
            let _ = connection.await;
        });
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, format!("localhost:{}", address.port()));
        for (name, value) in headers {
            request = request.header(name, value);
        }
        let response = sender.send_request(request.body(Body::from(body))?).await?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.into_body().collect().await?.to_bytes().to_vec();
        Ok((status, headers, body))
    }

    fn persisted_media() -> (tempfile::TempDir, Arc<LocalStreamCore>, String) {
        let directory = tempdir().expect("temporary library should be created");
        fs::write(directory.path().join("Movie.mp4"), b"0123456789")
            .expect("video should be created");
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let scan = core
            .scan_and_persist_library(directory.path())
            .expect("library should persist");
        let id = scan.items[0].id.clone();
        (directory, core, id)
    }

    fn browser_assets() -> (tempfile::TempDir, BrowserAssets) {
        let directory = tempdir().expect("asset directory should create");
        fs::create_dir(directory.path().join("assets")).expect("assets directory should create");
        fs::write(
            directory.path().join("index.html"),
            b"<!doctype html><div id=app></div>",
        )
        .expect("index should write");
        fs::write(
            directory.path().join("assets/app-a1b2c3.js"),
            b"console.log('app')",
        )
        .expect("script should write");
        fs::write(
            directory.path().join("assets/app-a1b2c3.css"),
            b"body{color:#123}",
        )
        .expect("stylesheet should write");
        let assets = BrowserAssets::from_directory(directory.path())
            .expect("browser assets should validate");
        (directory, assets)
    }

    async fn static_request(
        assets: BrowserAssets,
        method: Method,
        path: &str,
    ) -> axum::response::Response {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let policy = Arc::new(HttpsRequestPolicy {
            allowed_hosts: Arc::from(["localhost:443".to_owned()]),
            allowed_origins: Arc::from(["https://localhost:443".to_owned()]),
        });
        encrypted_router_with_assets(core, policy, assets)
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header(header::HOST, "localhost:443")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("static request should complete")
    }

    #[tokio::test]
    async fn browser_assets_serve_content_types_cache_policy_and_head() {
        let (_directory, assets) = browser_assets();
        let script = static_request(assets.clone(), Method::GET, "/assets/app-a1b2c3.js").await;
        assert_eq!(script.status(), StatusCode::OK);
        assert_eq!(
            script.headers()[header::CONTENT_TYPE],
            "text/javascript; charset=utf-8"
        );
        assert_eq!(
            script.headers()[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );
        assert_eq!(script.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");

        let index = static_request(assets.clone(), Method::GET, "/").await;
        assert_eq!(index.status(), StatusCode::OK);
        assert_eq!(
            index.headers()[header::CONTENT_TYPE],
            "text/html; charset=utf-8"
        );
        assert_eq!(
            index.headers()[header::CACHE_CONTROL],
            "no-cache, no-store, must-revalidate"
        );

        let head = static_request(assets, Method::HEAD, "/assets/app-a1b2c3.css").await;
        assert_eq!(head.status(), StatusCode::OK);
        assert_eq!(head.headers()[header::CONTENT_LENGTH], "16");
        assert!(head
            .into_body()
            .collect()
            .await
            .expect("body should collect")
            .to_bytes()
            .is_empty());
    }

    #[tokio::test]
    async fn browser_assets_fallback_for_navigation_but_never_for_assets_or_api() {
        let (_directory, assets) = browser_assets();
        let navigation = static_request(assets.clone(), Method::GET, "/library/movies").await;
        assert_eq!(navigation.status(), StatusCode::OK);
        assert!(response_text(navigation).await.contains("id=app"));

        let missing_asset = static_request(assets.clone(), Method::GET, "/assets/missing.js").await;
        assert_eq!(missing_asset.status(), StatusCode::NOT_FOUND);

        let missing_api = static_request(assets.clone(), Method::GET, "/api/v1/missing").await;
        assert_eq!(missing_api.status(), StatusCode::NOT_FOUND);
        assert!(!response_text(missing_api).await.contains("id=app"));

        let protected_api = static_request(assets, Method::GET, "/api/v1/library").await;
        assert_eq!(protected_api.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn browser_assets_reject_unsafe_or_malformed_paths_and_missing_host() {
        let (_directory, assets) = browser_assets();
        for path in [
            "/assets/%2e%2e/index.html",
            "/assets/%2Findex.html",
            "/assets/%5cindex.html",
            "/assets%2Fapp-a1b2c3.js",
            "/assets/%FF.js",
        ] {
            let response = static_request(assets.clone(), Method::GET, path).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "path: {path}");
        }

        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let policy = Arc::new(HttpsRequestPolicy {
            allowed_hosts: Arc::from(["localhost:443".to_owned()]),
            allowed_origins: Arc::from(["https://localhost:443".to_owned()]),
        });
        let response = encrypted_router_with_assets(core, policy, assets)
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should complete");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn loopback_https_lifecycle_can_host_public_ui_without_exposing_library() {
        let (_directory, assets) = browser_assets();
        let identity = test_identity();
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let server = start_loopback_https_server_with_assets(core, &identity, assets)
            .await
            .expect("HTTPS asset server should start");
        let address: std::net::SocketAddr = server
            .info()
            .base_url
            .strip_prefix("https://")
            .expect("HTTPS URL should have scheme")
            .parse()
            .expect("HTTPS address should parse");

        let (ui_status, _) = https_request(
            address,
            identity.root_certificate_der(),
            "localhost",
            "/",
            None,
        )
        .await
        .expect("UI request should complete");
        assert_eq!(ui_status, StatusCode::OK);

        let (library_status, _) = https_request(
            address,
            identity.root_certificate_der(),
            "localhost",
            "/api/v1/library",
            None,
        )
        .await
        .expect("library request should complete");
        assert_eq!(library_status, StatusCode::UNAUTHORIZED);
        assert_eq!(server.info().bind_scope, "loopback");
        assert!(!server.info().lan_available);
        server.shutdown().await;
    }

    #[tokio::test]
    async fn health_contract_is_versioned_and_reports_loopback_phase() {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let response = router(core)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body should collect")
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).expect("body should be JSON");

        assert_eq!(json["service"], "LocalStream");
        assert_eq!(json["apiVersion"], "v1");
        assert_eq!(json["lanAvailable"], false);
    }

    #[tokio::test]
    async fn library_contract_never_contains_filesystem_paths() {
        let directory = tempdir().expect("temporary library should be created");
        fs::write(directory.path().join("Private Movie.mp4"), b"video")
            .expect("video should be created");
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        core.scan_and_persist_library(directory.path())
            .expect("library should persist");

        let response = router(core)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/library")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body should collect")
            .to_bytes();
        let text = String::from_utf8(body.to_vec()).expect("body should be UTF-8");

        assert!(text.contains("Private Movie"));
        assert!(!text.contains(&directory.path().to_string_lossy().to_string()));
        assert!(!text.contains("rootPath"));
        assert!(!text.contains("path"));
    }

    #[tokio::test]
    async fn lifecycle_binds_only_to_loopback() {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let server = start_local_server(core).await.expect("server should start");
        let info = server.info();

        assert!(info.base_url.starts_with("http://127.0.0.1:"));
        assert_eq!(info.bind_scope, "loopback");
        assert!(!info.lan_available);
    }

    #[tokio::test]
    async fn https_lifecycle_serves_health_and_authenticated_library_on_loopback() {
        let directory = tempdir().expect("temporary library should be created");
        fs::write(directory.path().join("Secure Movie.mp4"), b"video")
            .expect("video should be created");
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        core.scan_and_persist_library(directory.path())
            .expect("library should persist");
        let issued = core
            .issue_peer_credential("TLS Test Client")
            .expect("credential should issue");
        let identity = test_identity();
        let server = start_loopback_https_server(core, &identity)
            .await
            .expect("HTTPS server should start");
        let info = server.info();
        let address: std::net::SocketAddr = info
            .base_url
            .strip_prefix("https://")
            .expect("HTTPS scheme should exist")
            .parse()
            .expect("address should parse");

        assert!(address.ip().is_loopback());
        assert_eq!(info.bind_scope, "loopback");
        assert!(!info.lan_available);
        let (health_status, health_body) = https_request(
            address,
            identity.root_certificate_der(),
            "localhost",
            "/api/v1/health",
            None,
        )
        .await
        .expect("trusted health request should succeed");
        assert_eq!(health_status, StatusCode::OK);
        assert!(String::from_utf8_lossy(&health_body).contains("LocalStream"));

        let (unauthorized, _) = https_request(
            address,
            identity.root_certificate_der(),
            "localhost",
            "/api/v1/library",
            None,
        )
        .await
        .expect("unauthorized HTTPS request should complete");
        assert_eq!(unauthorized, StatusCode::UNAUTHORIZED);
        let (library_status, library_body) = https_request(
            address,
            identity.root_certificate_der(),
            "localhost",
            "/api/v1/library",
            Some(&issued.bearer_token),
        )
        .await
        .expect("authenticated HTTPS request should succeed");
        assert_eq!(library_status, StatusCode::OK);
        assert!(String::from_utf8_lossy(&library_body).contains("Secure Movie"));

        server.shutdown().await;
    }

    #[tokio::test]
    async fn https_lifecycle_rejects_wrong_root_wrong_name_and_plaintext_downgrade() {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let identity = test_identity();
        let wrong_identity = test_identity();
        let server = start_loopback_https_server(core, &identity)
            .await
            .expect("HTTPS server should start");
        let address: std::net::SocketAddr = server
            .info()
            .base_url
            .strip_prefix("https://")
            .expect("HTTPS scheme should exist")
            .parse()
            .expect("address should parse");

        assert!(https_request(
            address,
            wrong_identity.root_certificate_der(),
            "localhost",
            "/api/v1/health",
            None,
        )
        .await
        .is_err());
        assert!(https_request(
            address,
            identity.root_certificate_der(),
            "wrong.local",
            "/api/v1/health",
            None,
        )
        .await
        .is_err());

        let mut plaintext = tokio::net::TcpStream::connect(address)
            .await
            .expect("plaintext socket should connect");
        plaintext
            .write_all(b"GET /api/v1/health HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .expect("plaintext bytes should write");
        let mut response = Vec::new();
        let _ = tokio::time::timeout(Duration::from_secs(2), plaintext.read_to_end(&mut response))
            .await;
        assert!(!response.windows(5).any(|window| window == b"HTTP/"));

        server.shutdown().await;
        let rebound = tokio::net::TcpListener::bind(address)
            .await
            .expect("graceful shutdown should release listener");
        drop(rebound);
    }

    #[tokio::test]
    async fn encrypted_pairing_route_requires_local_approval_and_claims_once() {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let identity = test_identity();
        let server = start_loopback_https_server(Arc::clone(&core), &identity)
            .await
            .expect("HTTPS server should start");
        let address: std::net::SocketAddr = server
            .info()
            .base_url
            .strip_prefix("https://")
            .expect("HTTPS scheme should exist")
            .parse()
            .expect("address should parse");
        let json_headers = vec![
            (header::CONTENT_TYPE, "application/json".to_owned()),
            (
                header::ORIGIN,
                format!("https://localhost:{}", address.port()),
            ),
        ];
        let (status, _, body) = https_exchange(
            address,
            identity.root_certificate_der(),
            "localhost",
            hyper::Method::POST,
            "/api/v1/pairing/requests",
            br#"{"displayName":"Living Room Client"}"#.to_vec(),
            json_headers.clone(),
        )
        .await
        .expect("pairing request should complete");
        assert_eq!(status, StatusCode::OK);
        let receipt: serde_json::Value =
            serde_json::from_slice(&body).expect("receipt should be JSON");
        let request_id = receipt["requestId"]
            .as_str()
            .expect("request ID should exist");
        let claim_secret = receipt["claimSecret"]
            .as_str()
            .expect("claim secret should exist");
        let verification_code = receipt["verificationCode"]
            .as_str()
            .expect("verification code should exist");
        assert_eq!(receipt["expiresInSeconds"], 120);
        let pending = core
            .pending_pairings()
            .expect("pending requests should load");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].display_name, "Living Room Client");

        let claim_body = serde_json::json!({
            "requestId": request_id,
            "claimSecret": claim_secret,
        })
        .to_string()
        .into_bytes();
        let (before_status, _, before_body) = https_exchange(
            address,
            identity.root_certificate_der(),
            "localhost",
            hyper::Method::POST,
            "/api/v1/pairing/claims",
            claim_body.clone(),
            json_headers.clone(),
        )
        .await
        .expect("pre-approval claim should complete");
        assert_eq!(before_status, StatusCode::BAD_REQUEST);
        assert!(String::from_utf8_lossy(&before_body).contains("pairing_claim_failed"));

        for invalid_body in [
            serde_json::json!({
                "requestId": "ls_pair_unknown",
                "claimSecret": claim_secret,
            })
            .to_string()
            .into_bytes(),
            serde_json::json!({
                "requestId": request_id,
                "claimSecret": "ls_claim_invalid",
            })
            .to_string()
            .into_bytes(),
        ] {
            let (failure_status, _, failure_body) = https_exchange(
                address,
                identity.root_certificate_der(),
                "localhost",
                hyper::Method::POST,
                "/api/v1/pairing/claims",
                invalid_body,
                json_headers.clone(),
            )
            .await
            .expect("invalid claim should complete uniformly");
            assert_eq!(failure_status, before_status);
            assert_eq!(failure_body, before_body);
        }

        core.approve_pairing(request_id, verification_code)
            .expect("local approval should succeed");
        let (claim_status, _, claim_response) = https_exchange(
            address,
            identity.root_certificate_der(),
            "localhost",
            hyper::Method::POST,
            "/api/v1/pairing/claims",
            claim_body.clone(),
            json_headers.clone(),
        )
        .await
        .expect("approved claim should complete");
        assert_eq!(claim_status, StatusCode::OK);
        let credential: serde_json::Value =
            serde_json::from_slice(&claim_response).expect("credential should be JSON");
        let bearer = credential["bearerToken"]
            .as_str()
            .expect("bearer should exist");
        assert!(bearer.starts_with("ls_peer_"));
        assert_eq!(credential["peer"]["displayName"], "Living Room Client");

        let (library_status, _) = https_request(
            address,
            identity.root_certificate_der(),
            "localhost",
            "/api/v1/library",
            Some(bearer),
        )
        .await
        .expect("issued credential should authenticate");
        assert_eq!(library_status, StatusCode::OK);

        let (replay_status, _, replay_body) = https_exchange(
            address,
            identity.root_certificate_der(),
            "localhost",
            hyper::Method::POST,
            "/api/v1/pairing/claims",
            claim_body,
            json_headers,
        )
        .await
        .expect("replay should complete uniformly");
        assert_eq!(replay_status, before_status);
        assert_eq!(replay_body, before_body);
        server.shutdown().await;
    }

    #[tokio::test]
    async fn encrypted_pairing_routes_bound_json_and_ignore_forwarding_headers_for_limits() {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let identity = test_identity();
        let server = start_loopback_https_server(core, &identity)
            .await
            .expect("HTTPS server should start");
        let address: std::net::SocketAddr = server
            .info()
            .base_url
            .strip_prefix("https://")
            .expect("HTTPS scheme should exist")
            .parse()
            .expect("address should parse");

        for attempt in 0..6 {
            let headers = vec![
                (header::CONTENT_TYPE, "application/json".to_owned()),
                (
                    header::ORIGIN,
                    format!("https://localhost:{}", address.port()),
                ),
                (
                    header::HeaderName::from_static("x-forwarded-for"),
                    format!("198.51.100.{}", attempt + 1),
                ),
            ];
            let (status, response_headers, body) = https_exchange(
                address,
                identity.root_certificate_der(),
                "localhost",
                hyper::Method::POST,
                "/api/v1/pairing/requests",
                format!(r#"{{"displayName":"Client {attempt}"}}"#).into_bytes(),
                headers,
            )
            .await
            .expect("rate-limit request should complete");
            if attempt < 5 {
                assert_eq!(status, StatusCode::OK);
            } else {
                assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
                assert!(response_headers
                    .get(header::RETRY_AFTER)
                    .and_then(|value| value.to_str().ok())
                    .is_some_and(|value| value.parse::<u64>().is_ok()));
                assert!(String::from_utf8_lossy(&body).contains("rate_limited"));
            }
        }
        server.shutdown().await;

        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let identity = test_identity();
        let server = start_loopback_https_server(core, &identity)
            .await
            .expect("second HTTPS server should start");
        let address: std::net::SocketAddr = server
            .info()
            .base_url
            .strip_prefix("https://")
            .expect("HTTPS scheme should exist")
            .parse()
            .expect("address should parse");
        let (unknown_status, _, unknown_body) = https_exchange(
            address,
            identity.root_certificate_der(),
            "localhost",
            hyper::Method::POST,
            "/api/v1/pairing/requests",
            br#"{"displayName":"Client","unexpected":true}"#.to_vec(),
            vec![
                (header::CONTENT_TYPE, "application/json".to_owned()),
                (
                    header::ORIGIN,
                    format!("https://localhost:{}", address.port()),
                ),
            ],
        )
        .await
        .expect("unknown-field request should complete");
        assert_eq!(unknown_status, StatusCode::BAD_REQUEST);
        assert!(String::from_utf8_lossy(&unknown_body).contains("invalid_pairing_request"));

        let (large_status, _, _) = https_exchange(
            address,
            identity.root_certificate_der(),
            "localhost",
            hyper::Method::POST,
            "/api/v1/pairing/requests",
            vec![b'x'; 2_049],
            vec![
                (header::CONTENT_TYPE, "application/json".to_owned()),
                (
                    header::ORIGIN,
                    format!("https://localhost:{}", address.port()),
                ),
            ],
        )
        .await
        .expect("oversized request should complete");
        assert_eq!(large_status, StatusCode::PAYLOAD_TOO_LARGE);
        server.shutdown().await;
    }

    #[tokio::test]
    async fn encrypted_routes_enforce_authority_origin_and_fetch_metadata_before_pairing() {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let identity = test_identity();
        let server = start_loopback_https_server(Arc::clone(&core), &identity)
            .await
            .expect("HTTPS server should start");
        let address: std::net::SocketAddr = server
            .info()
            .base_url
            .strip_prefix("https://")
            .expect("HTTPS scheme should exist")
            .parse()
            .expect("address should parse");
        let origin = format!("https://localhost:{}", address.port());
        let pairing_body = br#"{"displayName":"Origin Test"}"#.to_vec();

        let missing_host = encrypted_router(
            Arc::clone(&core),
            Arc::new(HttpsRequestPolicy::for_loopback(address)),
        )
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("missing-host request should complete");
        assert_eq!(missing_host.status(), StatusCode::FORBIDDEN);

        for host_headers in [
            vec![(header::HOST, "example.test".to_owned())],
            vec![(header::HOST, format!("localhost:{}", address.port()))],
        ] {
            let (status, _, _) = https_exchange(
                address,
                identity.root_certificate_der(),
                "localhost",
                hyper::Method::GET,
                "/api/v1/health",
                Vec::new(),
                host_headers,
            )
            .await
            .expect("invalid-host request should complete");
            assert_eq!(status, StatusCode::FORBIDDEN);
        }

        let invalid_origin_headers = [
            Vec::new(),
            vec![(header::ORIGIN, "null".to_owned())],
            vec![(
                header::ORIGIN,
                format!("http://localhost:{}", address.port()),
            )],
            vec![(header::ORIGIN, "https://example.test".to_owned())],
            vec![(header::ORIGIN, "not an origin".to_owned())],
            vec![
                (header::ORIGIN, origin.clone()),
                (header::ORIGIN, origin.clone()),
            ],
            vec![
                (header::ORIGIN, origin.clone()),
                (
                    header::HeaderName::from_static("sec-fetch-site"),
                    "cross-site".to_owned(),
                ),
            ],
        ];
        let mut rejection_body = None;
        for mut headers in invalid_origin_headers {
            headers.push((header::CONTENT_TYPE, "application/json".to_owned()));
            let (status, _, body) = https_exchange(
                address,
                identity.root_certificate_der(),
                "localhost",
                hyper::Method::POST,
                "/api/v1/pairing/requests",
                pairing_body.clone(),
                headers,
            )
            .await
            .expect("invalid-origin request should complete");
            assert_eq!(status, StatusCode::FORBIDDEN);
            if let Some(expected) = &rejection_body {
                assert_eq!(&body, expected);
            } else {
                rejection_body = Some(body);
            }
        }
        assert!(core
            .pending_pairings()
            .expect("pending pairings should list")
            .is_empty());

        let (status, _, _) = https_exchange(
            address,
            identity.root_certificate_der(),
            "localhost",
            hyper::Method::POST,
            "/api/v1/pairing/requests",
            pairing_body,
            vec![
                (header::CONTENT_TYPE, "application/json".to_owned()),
                (header::ORIGIN, origin),
                (
                    header::HeaderName::from_static("sec-fetch-site"),
                    "same-origin".to_owned(),
                ),
                (
                    header::HeaderName::from_static("forwarded"),
                    "host=example.test;proto=http".to_owned(),
                ),
                (
                    header::HeaderName::from_static("x-forwarded-host"),
                    "example.test".to_owned(),
                ),
            ],
        )
        .await
        .expect("same-origin request should complete");
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            core.pending_pairings()
                .expect("pending pairings should list")
                .len(),
            1
        );
        server.shutdown().await;
    }

    #[tokio::test]
    async fn https_connection_limit_fails_closed_and_handshake_timeout_releases_capacity() {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let identity = test_identity();
        let server = start_loopback_https_server_with_limits(
            core,
            &identity,
            HttpsLimits {
                max_connections: 1,
                handshake_timeout: Duration::from_millis(100),
            },
        )
        .await
        .expect("limited HTTPS server should start");
        let address: std::net::SocketAddr = server
            .info()
            .base_url
            .strip_prefix("https://")
            .expect("HTTPS scheme should exist")
            .parse()
            .expect("address should parse");

        let stalled = tokio::net::TcpStream::connect(address)
            .await
            .expect("stalled connection should open");
        tokio::time::sleep(Duration::from_millis(25)).await;
        let saturated = https_request(
            address,
            identity.root_certificate_der(),
            "localhost",
            "/api/v1/health",
            None,
        )
        .await;
        assert!(saturated.is_err());

        tokio::time::sleep(Duration::from_millis(125)).await;
        let (status, _) = https_request(
            address,
            identity.root_certificate_der(),
            "localhost",
            "/api/v1/health",
            None,
        )
        .await
        .expect("capacity should recover after handshake timeout");
        assert_eq!(status, StatusCode::OK);
        drop(stalled);
        server.shutdown().await;
    }

    #[tokio::test]
    async fn browser_pairing_sets_secure_cookie_authenticates_gets_and_revokes() {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let identity = test_identity();
        let server = start_loopback_https_server(Arc::clone(&core), &identity)
            .await
            .expect("HTTPS server should start");
        let address: std::net::SocketAddr = server
            .info()
            .base_url
            .strip_prefix("https://")
            .expect("HTTPS scheme should exist")
            .parse()
            .expect("address should parse");
        let json_headers = vec![
            (header::CONTENT_TYPE, "application/json".to_owned()),
            (
                header::ORIGIN,
                format!("https://localhost:{}", address.port()),
            ),
        ];
        let (_, _, request_body) = https_exchange(
            address,
            identity.root_certificate_der(),
            "localhost",
            hyper::Method::POST,
            "/api/v1/pairing/requests",
            br#"{"displayName":"Browser Client"}"#.to_vec(),
            json_headers.clone(),
        )
        .await
        .expect("browser pairing request should complete");
        let receipt: serde_json::Value =
            serde_json::from_slice(&request_body).expect("receipt should parse");
        let request_id = receipt["requestId"]
            .as_str()
            .expect("request ID should exist");
        let claim_secret = receipt["claimSecret"]
            .as_str()
            .expect("claim secret should exist");
        core.approve_pairing(
            request_id,
            receipt["verificationCode"]
                .as_str()
                .expect("verification code should exist"),
        )
        .expect("browser pairing should approve");
        let claim_body = serde_json::json!({
            "requestId": request_id,
            "claimSecret": claim_secret,
        })
        .to_string()
        .into_bytes();

        let (claim_status, claim_headers, response_body) = https_exchange(
            address,
            identity.root_certificate_der(),
            "localhost",
            hyper::Method::POST,
            "/api/v1/pairing/browser-claims",
            claim_body.clone(),
            json_headers.clone(),
        )
        .await
        .expect("browser claim should complete");
        assert_eq!(claim_status, StatusCode::NO_CONTENT);
        assert!(response_body.is_empty());
        let set_cookie = claim_headers[header::SET_COOKIE]
            .to_str()
            .expect("cookie should be text");
        assert!(set_cookie.starts_with("__Host-localstream_session=ls_session_"));
        assert!(set_cookie.contains("; Path=/"));
        assert!(set_cookie.contains("; Max-Age=86400"));
        assert!(set_cookie.contains("; HttpOnly"));
        assert!(set_cookie.contains("; Secure"));
        assert!(set_cookie.contains("; SameSite=Strict"));
        assert!(!set_cookie.to_ascii_lowercase().contains("domain="));
        let cookie_pair = set_cookie
            .split(';')
            .next()
            .expect("cookie pair should exist")
            .to_owned();

        let (library_status, _, _) = https_exchange(
            address,
            identity.root_certificate_der(),
            "localhost",
            hyper::Method::GET,
            "/api/v1/library",
            Vec::new(),
            vec![(header::COOKIE, cookie_pair.clone())],
        )
        .await
        .expect("cookie-authenticated GET should complete");
        assert_eq!(library_status, StatusCode::OK);

        let (replay_status, _, replay_body) = https_exchange(
            address,
            identity.root_certificate_der(),
            "localhost",
            hyper::Method::POST,
            "/api/v1/pairing/browser-claims",
            claim_body,
            json_headers,
        )
        .await
        .expect("browser claim replay should complete");
        assert_eq!(replay_status, StatusCode::BAD_REQUEST);
        assert!(String::from_utf8_lossy(&replay_body).contains("pairing_claim_failed"));

        let malformed = vec![(header::COOKIE, "__Host-localstream_session=bad".to_owned())];
        let duplicate = vec![
            (header::COOKIE, cookie_pair.clone()),
            (header::COOKIE, cookie_pair.clone()),
        ];
        let mut failures = Vec::new();
        for headers in [malformed, duplicate] {
            let (status, _, body) = https_exchange(
                address,
                identity.root_certificate_der(),
                "localhost",
                hyper::Method::GET,
                "/api/v1/library",
                Vec::new(),
                headers,
            )
            .await
            .expect("invalid cookie request should complete");
            failures.push((status, body));
        }
        assert_eq!(failures[0], failures[1]);
        assert_eq!(failures[0].0, StatusCode::UNAUTHORIZED);

        let peer = core
            .trusted_peers()
            .expect("browser peer should list")
            .into_iter()
            .find(|peer| peer.display_name == "Browser Client")
            .expect("browser peer should exist");
        core.revoke_peer(&peer.id)
            .expect("browser peer should revoke");
        let (revoked_status, _, revoked_body) = https_exchange(
            address,
            identity.root_certificate_der(),
            "localhost",
            hyper::Method::GET,
            "/api/v1/library",
            Vec::new(),
            vec![(header::COOKIE, cookie_pair)],
        )
        .await
        .expect("revoked cookie request should complete");
        assert_eq!(revoked_status, failures[0].0);
        assert_eq!(revoked_body, failures[0].1);
        server.shutdown().await;
    }

    #[tokio::test]
    async fn trusted_local_http_router_does_not_expose_pairing_routes() {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let response = router(core)
            .oneshot(
                Request::builder()
                    .method(hyper::Method::POST)
                    .uri("/api/v1/pairing/requests")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"displayName":"Client"}"#))
                    .expect("request should build"),
            )
            .await
            .expect("local router request should complete");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn stream_contract_returns_the_full_file_without_disclosing_its_path() {
        let (directory, core, id) = persisted_media();
        let response = router(core)
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/media/{id}/stream"))
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["accept-ranges"], "bytes");
        assert_eq!(response.headers()["content-type"], "video/mp4");
        assert_eq!(response.headers()["content-length"], "10");
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body should stream")
            .to_bytes();
        assert_eq!(&body[..], b"0123456789");
        assert!(!String::from_utf8_lossy(&body)
            .contains(&directory.path().to_string_lossy().to_string()));
    }

    #[tokio::test]
    async fn stream_contract_honors_a_single_byte_range() {
        let (_directory, core, id) = persisted_media();
        let response = router(core)
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/media/{id}/stream"))
                    .header("range", "bytes=2-5")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), 206);
        assert_eq!(response.headers()["content-range"], "bytes 2-5/10");
        assert_eq!(response.headers()["content-length"], "4");
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body should stream")
            .to_bytes();
        assert_eq!(&body[..], b"2345");
    }

    #[tokio::test]
    async fn stream_contract_rejects_an_unsatisfiable_range_safely() {
        let (_directory, core, id) = persisted_media();
        let response = router(core)
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/media/{id}/stream"))
                    .header("range", "bytes=20-")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), 416);
        assert_eq!(response.headers()["content-range"], "bytes */10");
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body should collect")
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).expect("body should be JSON");
        assert_eq!(json["error"]["code"], "range_not_satisfiable");
    }

    #[tokio::test]
    async fn stream_contract_returns_a_safe_error_for_an_unknown_id() {
        let (_directory, core, _id) = persisted_media();
        let response = router(core)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/media/unknown/stream")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), 404);
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body should collect")
            .to_bytes();
        let text = String::from_utf8(body.to_vec()).expect("body should be UTF-8");
        assert!(text.contains("media_not_found"));
        assert!(!text.contains("Movie.mp4"));
    }

    async fn response_text(response: axum::response::Response) -> String {
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body should collect")
            .to_bytes();
        String::from_utf8(body.to_vec()).expect("body should be UTF-8")
    }

    #[tokio::test]
    async fn authenticated_router_keeps_health_public() {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let response = authenticated_router(core)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn protected_routes_return_one_safe_response_for_missing_invalid_and_revoked_tokens() {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let issued = core
            .issue_peer_credential("Revoked TV")
            .expect("credential should issue");
        core.revoke_peer(&issued.peer.id)
            .expect("credential should revoke");
        let credentials = [
            None,
            Some("Basic not-a-bearer"),
            Some("Bearer ls_peer_invalid"),
            Some(issued.bearer_token.as_str()),
        ];
        let mut expected_body = None;

        for credential in credentials {
            let mut builder = Request::builder().uri("/api/v1/library");
            if let Some(credential) = credential {
                builder = builder.header("authorization", credential);
            }
            let response = authenticated_router(Arc::clone(&core))
                .oneshot(builder.body(Body::empty()).expect("request should build"))
                .await
                .expect("request should succeed");

            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(response.headers()["www-authenticate"], "Bearer");
            let body = response_text(response).await;
            assert!(body.contains("unauthorized"));
            if let Some(expected) = &expected_body {
                assert_eq!(&body, expected);
            } else {
                expected_body = Some(body);
            }
        }
    }

    #[tokio::test]
    async fn protected_routes_reject_duplicate_authorization_headers() {
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        let issued = core
            .issue_peer_credential("Living Room TV")
            .expect("credential should issue");
        let mut request = Request::builder()
            .uri("/api/v1/library")
            .body(Body::empty())
            .expect("request should build");
        request.headers_mut().append(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", issued.bearer_token))
                .expect("header should build"),
        );
        request.headers_mut().append(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer duplicate"),
        );

        let response = authenticated_router(core)
            .oneshot(request)
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn valid_library_read_credential_accesses_library_and_peer_extension() {
        let directory = tempdir().expect("temporary library should be created");
        fs::write(directory.path().join("Authorized Movie.mp4"), b"video")
            .expect("video should be created");
        let core = Arc::new(LocalStreamCore::in_memory().expect("core should open"));
        core.scan_and_persist_library(directory.path())
            .expect("library should persist");
        let issued = core
            .issue_peer_credential("Living Room TV")
            .expect("credential should issue");
        let authorization = format!("Bearer {}", issued.bearer_token);

        let response = authenticated_router(Arc::clone(&core))
            .oneshot(
                Request::builder()
                    .uri("/api/v1/library")
                    .header("authorization", &authorization)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response_text(response).await.contains("Authorized Movie"));

        async fn identity(Extension(peer): Extension<TrustedPeer>) -> Json<TrustedPeer> {
            Json(peer)
        }
        let identity_router = Router::new().route("/identity", get(identity)).route_layer(
            middleware::from_fn_with_state(Arc::clone(&core), require_library_read),
        );
        let response = identity_router
            .oneshot(
                Request::builder()
                    .uri("/identity")
                    .header("authorization", authorization)
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");
        let identity = response_text(response).await;
        assert!(identity.contains("Living Room TV"));
        assert!(!identity.contains("token"));
    }

    #[tokio::test]
    async fn valid_credential_streams_an_authorized_byte_range() {
        let (_directory, core, id) = persisted_media();
        let issued = core
            .issue_peer_credential("Bedroom Tablet")
            .expect("credential should issue");
        let response = authenticated_router(core)
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/media/{id}/stream"))
                    .header("authorization", format!("Bearer {}", issued.bearer_token))
                    .header("range", "bytes=4-7")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(response.headers()["content-range"], "bytes 4-7/10");
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body should stream")
            .to_bytes();
        assert_eq!(&body[..], b"4567");
    }
}
