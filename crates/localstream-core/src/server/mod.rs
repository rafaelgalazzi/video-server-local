use std::{net::SocketAddr, sync::Arc};

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use tokio_util::io::ReaderStream;

use crate::{
    streaming::{range::parse_single_range, StreamingError},
    LibraryScan, LocalStreamCore,
};

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
}

impl ApiError {
    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: "The request could not be completed.",
            content_range: None,
        }
    }

    fn media_not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "media_not_found",
            message: "The requested media is unavailable.",
            content_range: None,
        }
    }

    fn range_not_satisfiable(size: u64) -> Self {
        Self {
            status: StatusCode::RANGE_NOT_SATISFIABLE,
            code: "range_not_satisfiable",
            message: "The requested byte range is not satisfiable.",
            content_range: Some(format!("bytes */{size}")),
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
    use std::{fs, sync::Arc};

    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tempfile::tempdir;
    use tower::ServiceExt;

    use crate::LocalStreamCore;

    use super::{router, start_local_server};

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
}
