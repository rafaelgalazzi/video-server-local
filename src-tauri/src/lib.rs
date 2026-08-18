use localstream_core::{AppInfo, LocalStreamCore};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
fn app_info(core: tauri::State<'_, LocalStreamCore>) -> AppInfo {
    core.app_info()
}

#[tauri::command]
fn select_and_scan_library(
    app: tauri::AppHandle,
    core: tauri::State<'_, LocalStreamCore>,
) -> Result<Option<localstream_core::LibraryScan>, String> {
    let Some(directory) = app.dialog().file().blocking_pick_folder() else {
        return Ok(None);
    };
    let directory = directory
        .into_path()
        .map_err(|_| "the selected folder is not a local filesystem path".to_owned())?;

    core.scan_library(directory)
        .map(Some)
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(LocalStreamCore)
        .invoke_handler(tauri::generate_handler![app_info, select_and_scan_library])
        .run(tauri::generate_context!())
        .expect("failed to run LocalStream");
}
