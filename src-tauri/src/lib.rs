use std::sync::Arc;

use localstream_core::{AppInfo, LocalStreamCore};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

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

    core.scan_and_persist_library(directory)
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
fn server_info(
    server: tauri::State<'_, localstream_core::server::ServerHandle>,
) -> localstream_core::server::ServerInfo {
    server.info()
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
            let server = tauri::async_runtime::block_on(
                localstream_core::server::start_local_server(Arc::clone(&core)),
            )?;
            app.manage(core);
            app.manage(server);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            current_library,
            server_info,
            select_and_scan_library
        ])
        .run(tauri::generate_context!())
        .expect("failed to run LocalStream");
}
