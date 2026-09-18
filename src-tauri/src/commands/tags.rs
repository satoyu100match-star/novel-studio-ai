use crate::db::Db;
use crate::error::AppResult;
use crate::models::Tag;
use crate::repositories::tags_repository;
use tauri::State;

#[tauri::command]
pub fn list_tags(db: State<Db>, project_id: String) -> AppResult<Vec<Tag>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    tags_repository::list_by_project(&conn, &project_id)
}
