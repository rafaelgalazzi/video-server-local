use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};

use crate::{LibraryScan, LocalStreamCore};

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

struct ApiError;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiErrorEnvelope {
                error: ApiErrorBody {
                    code: "internal_error",
                    message: "The request could not be completed.",
                },
            }),
        )
            .into_response()
    }
}

pub fn router(core: Arc<LocalStreamCore>) -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/library", get(current_library))
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
    core.current_library().map(Json).map_err(|_| ApiError)
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
}
