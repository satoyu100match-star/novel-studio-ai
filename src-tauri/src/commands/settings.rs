use crate::db::Db;
use crate::error::AppResult;
use crate::repositories::settings_repository;
use tauri::State;

#[tauri::command]
pub fn get_setting(db: State<Db>, key: String) -> AppResult<Option<String>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    settings_repository::get(&conn, &key)
}

#[tauri::command]
pub fn set_setting(db: State<Db>, key: String, value: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    settings_repository::set(&conn, &key, &value)
}

#[tauri::command]
pub fn list_settings(db: State<Db>) -> AppResult<Vec<(String, String)>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    settings_repository::list(&conn)
}
