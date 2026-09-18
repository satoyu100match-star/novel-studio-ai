use crate::db::Db;
use crate::error::AppResult;
use crate::models::SearchHit;
use crate::repositories::search_repository;
use tauri::State;

#[tauri::command]
pub fn search_project(db: State<Db>, project_id: String, query: String) -> AppResult<Vec<SearchHit>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    search_repository::search_project(&conn, &project_id, &query)
}
