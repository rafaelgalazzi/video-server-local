use localstream_core::{AppInfo, LocalStreamCore};

#[tauri::command]
fn app_info(core: tauri::State<'_, LocalStreamCore>) -> AppInfo {
    core.app_info()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(LocalStreamCore)
        .invoke_handler(tauri::generate_handler![app_info])
        .run(tauri::generate_context!())
        .expect("failed to run LocalStream");
}
