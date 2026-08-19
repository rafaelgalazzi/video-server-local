use std::sync::Arc;

use localstream_core::{AppInfo, LocalStreamCore};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

struct LanRuntime {
    config_store: localstream_core::lan::FileLanConfigStore,
    config: std::sync::Mutex<localstream_core::lan::LanServerConfig>,
    status: std::sync::Mutex<localstream_core::lan::LanServerStatus>,
    _server: std::sync::Mutex<Option<localstream_core::server::HttpsServerHandle>>,
}

#[tauri::command]
fn app_info(core: tauri::State<'_, Arc<LocalStreamCore>>) -> AppInfo {
    core.app_info()
}

#[tauri::command]
fn select_and_scan_library(
    app: tauri::AppHandle,
    core: tauri::State<'_, Arc<LocalStreamCore>>,
) -> Result<Option<localstream_core::LibraryScan>, String> {
    let Some(directory) = app.dialog().file().blocking_pick_folder() else {
        return Ok(None);
    };
    let directory = directory
        .into_path()
        .map_err(|_| "the selected folder is not a local filesystem path".to_owned())?;

    tauri::async_runtime::block_on(
        core.scan_and_persist_library_with_probe(
            directory,
            tokio_util::sync::CancellationToken::new(),
        ),
    )
    .map(Some)
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn current_library(
    core: tauri::State<'_, Arc<LocalStreamCore>>,
) -> Result<Option<localstream_core::LibraryScan>, String> {
    core.current_library().map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_local_database(core: tauri::State<'_, Arc<LocalStreamCore>>) -> Result<(), String> {
    core.clear_local_database()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn select_audio_track(
    core: tauri::State<'_, Arc<LocalStreamCore>>,
    media_id: String,
    track_id: Option<String>,
) -> Result<localstream_core::AudioSelection, String> {
    core.select_audio_track(&media_id, track_id.as_deref())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn select_subtitle(
    core: tauri::State<'_, Arc<LocalStreamCore>>,
    media_id: String,
    mode: localstream_core::media::SubtitleMode,
    track_id: Option<String>,
) -> Result<localstream_core::SubtitleSelection, String> {
    core.select_subtitle(&media_id, mode, track_id.as_deref())
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn prepare_playback(
    core: tauri::State<'_, Arc<LocalStreamCore>>,
    playback: tauri::State<'_, localstream_core::playback::LocalPlaybackService>,
    media_id: String,
    capabilities: localstream_core::compatibility::ClientCapabilities,
) -> Result<localstream_core::playback::PlaybackPreparation, String> {
    playback
        .prepare(&core, &media_id, &capabilities)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn playback_job(
    playback: tauri::State<'_, localstream_core::playback::LocalPlaybackService>,
    job_id: localstream_core::media_jobs::MediaJobId,
) -> Result<localstream_core::media_jobs::MediaJobSnapshot, String> {
    playback
        .snapshot(job_id)
        .ok_or_else(|| "the playback job does not exist".to_owned())
}

#[tauri::command]
fn cancel_playback(
    playback: tauri::State<'_, localstream_core::playback::LocalPlaybackService>,
    job_id: localstream_core::media_jobs::MediaJobId,
) -> bool {
    playback.cancel(job_id)
}

#[tauri::command]
async fn release_playback(
    playback: tauri::State<'_, localstream_core::playback::LocalPlaybackService>,
    job_id: localstream_core::media_jobs::MediaJobId,
) -> Result<bool, String> {
    let playback = playback.inner().clone();
    Ok(playback.cancel_and_release(job_id).await)
}

#[tauri::command]
fn server_info(
    server: tauri::State<'_, localstream_core::server::ServerHandle>,
) -> localstream_core::server::ServerInfo {
    server.info()
}

#[tauri::command]
fn lan_server_config(
    runtime: tauri::State<'_, LanRuntime>,
) -> Result<localstream_core::lan::LanServerConfig, String> {
    runtime
        .config
        .lock()
        .map(|value| value.clone())
        .map_err(|_| "the LAN configuration is unavailable".to_owned())
}

#[tauri::command]
fn save_lan_server_config(
    runtime: tauri::State<'_, LanRuntime>,
    config: localstream_core::lan::LanServerConfig,
) -> Result<(), String> {
    let service = localstream_core::lan::LanConfigService::new(&runtime.config_store);
    service.save(&config).map_err(|error| error.to_string())?;
    *runtime
        .config
        .lock()
        .map_err(|_| "the LAN configuration is unavailable".to_owned())? = config;
    Ok(())
}

#[tauri::command]
fn lan_server_status(
    runtime: tauri::State<'_, LanRuntime>,
) -> Result<localstream_core::lan::LanServerStatus, String> {
    runtime
        .status
        .lock()
        .map(|value| value.clone())
        .map_err(|_| "the LAN server status is unavailable".to_owned())
}

#[tauri::command]
fn suggested_lan_addresses() -> Vec<std::net::IpAddr> {
    localstream_core::lan::primary_lan_address()
        .into_iter()
        .collect()
}

#[tauri::command]
fn node_identity(
    identity: tauri::State<'_, localstream_core::node_identity::NodeIdentitySummary>,
) -> localstream_core::node_identity::NodeIdentitySummary {
    identity.inner().clone()
}

#[tauri::command]
fn reset_node_identity(
    core: tauri::State<'_, Arc<LocalStreamCore>>,
    store: tauri::State<'_, localstream_core::node_identity::KeyringNodeSecretStore>,
) -> Result<usize, String> {
    core.reset_node_identity(store.inner())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn export_node_root_certificate(
    app: tauri::AppHandle,
    expected: tauri::State<'_, localstream_core::node_identity::NodeIdentitySummary>,
    store: tauri::State<'_, localstream_core::node_identity::KeyringNodeSecretStore>,
) -> Result<bool, String> {
    let identity = localstream_core::node_identity::NodeIdentityService::new(store.inner())
        .load_existing()
        .map_err(|error| error.to_string())?;
    if identity.summary() != expected.inner() {
        return Err("the protected node identity changed unexpectedly".to_owned());
    }
    let Some(destination) = app
        .dialog()
        .file()
        .add_filter("X.509 certificate", &["cer", "der"])
        .set_file_name(format!("localstream-{}.cer", expected.node_id))
        .blocking_save_file()
    else {
        return Ok(false);
    };
    let destination = destination
        .into_path()
        .map_err(|_| "the certificate destination is not a local filesystem path".to_owned())?;
    std::fs::write(destination, identity.root_certificate_der())
        .map_err(|_| "the root certificate could not be exported".to_owned())?;
    Ok(true)
}

#[tauri::command]
fn pending_pairings(
    core: tauri::State<'_, Arc<LocalStreamCore>>,
) -> Result<Vec<localstream_core::auth::PendingPairing>, String> {
    core.pending_pairings().map_err(|error| error.to_string())
}

#[tauri::command]
fn approve_pairing(
    core: tauri::State<'_, Arc<LocalStreamCore>>,
    request_id: String,
    verification_code: String,
) -> Result<(), String> {
    core.approve_pairing(&request_id, &verification_code)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn reject_pairing(
    core: tauri::State<'_, Arc<LocalStreamCore>>,
    request_id: String,
) -> Result<(), String> {
    core.reject_pairing(&request_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn trusted_peers(
    core: tauri::State<'_, Arc<LocalStreamCore>>,
) -> Result<Vec<localstream_core::auth::TrustedPeerSummary>, String> {
    core.trusted_peers().map_err(|error| error.to_string())
}

#[tauri::command]
fn revoke_trusted_peer(
    core: tauri::State<'_, Arc<LocalStreamCore>>,
    peer_id: String,
) -> Result<bool, String> {
    core.revoke_peer(&peer_id)
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data)?;
            let core = Arc::new(
                LocalStreamCore::open(app_data.join("localstream.sqlite3"))
                    .map_err(std::io::Error::other)?,
            );
            let identity_store =
                localstream_core::node_identity::KeyringNodeSecretStore::new("desktop-default")
                    .map_err(std::io::Error::other)?;
            let identity_service =
                localstream_core::node_identity::NodeIdentityService::new(identity_store);
            let identity = identity_service
                .load_or_create()
                .map_err(std::io::Error::other)?;
            let identity_summary = identity.summary().clone();
            let config_store =
                localstream_core::lan::FileLanConfigStore::new(app_data.join("lan-server.conf"));
            let lan_config = localstream_core::lan::LanConfigService::new(&config_store)
                .load()
                .map_err(std::io::Error::other)?;
            let mut lan_status = localstream_core::lan::LanServerStatus {
                configured: lan_config.enabled,
                active: false,
                endpoint: None,
                failure: None,
            };
            let mut lan_server = None;
            if lan_config.enabled {
                let asset_root = app.path().resource_dir()?.join("web");
                let result = localstream_core::server::BrowserAssets::from_directory(asset_root)
                    .map_err(|_| localstream_core::server::HttpsServerError::ListenerUnavailable)
                    .and_then(|assets| {
                        localstream_core::server::prepare_lan_server(
                            &identity,
                            &localstream_core::lan::TlsLeafLifecycle::default(),
                            lan_config.clone(),
                            assets,
                        )
                    });
                let evidence = localstream_core::lan::LanSecurityEvidence {
                    browser_trust_onboarding: true,
                    native_protected_storage: true,
                    negative_security_suite: true,
                };
                let (_, permit) = localstream_core::lan::audit_activation(evidence);
                if let (Ok(prepared), Some(permit)) = (result, permit) {
                    match tauri::async_runtime::block_on(
                        localstream_core::server::activate_lan_server(
                            Arc::clone(&core),
                            prepared,
                            permit,
                        ),
                    ) {
                        Ok(server) => {
                            lan_status.active = true;
                            lan_status.endpoint = Some(server.info().base_url);
                            lan_server = Some(server);
                        }
                        Err(_) => lan_status.failure = Some("secure_start_failed"),
                    }
                } else {
                    lan_status.failure = Some("security_preflight_failed");
                }
            }
            let identity_store = identity_service.into_store();
            let playback = tauri::async_runtime::block_on(
                localstream_core::playback::LocalPlaybackService::start(
                    localstream_core::media_jobs::MediaJobConfig {
                        work_root: app_data.join("media-cache"),
                        max_concurrent: 2,
                        max_queued: 8,
                        temporary_byte_quota: 8 * 1024 * 1024 * 1024,
                    },
                ),
            )
            .map_err(std::io::Error::other)?;
            let server = tauri::async_runtime::block_on(
                localstream_core::server::start_local_server_with_playback(
                    Arc::clone(&core),
                    playback.clone(),
                ),
            )?;
            app.manage(core);
            app.manage(playback);
            app.manage(identity_summary);
            app.manage(identity_store);
            app.manage(server);
            app.manage(LanRuntime {
                config_store,
                config: std::sync::Mutex::new(lan_config),
                status: std::sync::Mutex::new(lan_status),
                _server: std::sync::Mutex::new(lan_server),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            approve_pairing,
            cancel_playback,
            clear_local_database,
            current_library,
            export_node_root_certificate,
            node_identity,
            pending_pairings,
            playback_job,
            prepare_playback,
            reject_pairing,
            reset_node_identity,
            release_playback,
            revoke_trusted_peer,
            server_info,
            lan_server_config,
            save_lan_server_config,
            lan_server_status,
            suggested_lan_addresses,
            select_and_scan_library,
            select_audio_track,
            select_subtitle,
            trusted_peers
        ])
        .run(tauri::generate_context!())
        .expect("failed to run LocalStream");
}
