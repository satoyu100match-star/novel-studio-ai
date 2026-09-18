//! Exposes build-time app identity to the frontend so the product name
//! doesn't get hardcoded in dozens of UI strings -- see CLAUDE.md
//! ("app name must stay renameable"). The single source of truth is
//! `tauri.conf.json`; this command just relays it.

use serde::Serialize;
use tauri::AppHandle;

#[derive(Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub identifier: String,
}

#[tauri::command]
pub fn app_info(app: AppHandle) -> AppInfo {
    let package_info = app.package_info();
    AppInfo {
        name: package_info.name.clone(),
        version: package_info.version.to_string(),
        identifier: app.config().identifier.clone(),
    }
}
