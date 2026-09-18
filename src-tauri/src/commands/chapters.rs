use crate::db::Db;
use crate::error::AppResult;
use crate::models::{Chapter, Part};
use crate::repositories::{chapters_repository, parts_repository};
use tauri::State;

#[tauri::command]
pub fn create_part(db: State<Db>, project_id: String, title: String) -> AppResult<Part> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    parts_repository::create(&conn, &project_id, &title)
}

#[tauri::command]
pub fn list_parts(db: State<Db>, project_id: String) -> AppResult<Vec<Part>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    parts_repository::list_by_project(&conn, &project_id)
}

#[tauri::command]
pub fn rename_part(db: State<Db>, id: String, title: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    parts_repository::rename(&conn, &id, &title)
}

#[tauri::command]
pub fn delete_part(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    parts_repository::soft_delete(&conn, &id)
}

#[tauri::command]
pub fn create_chapter(
    db: State<Db>,
    project_id: String,
    part_id: Option<String>,
    title: String,
) -> AppResult<Chapter> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    chapters_repository::create(&conn, &project_id, part_id.as_deref(), &title)
}

#[tauri::command]
pub fn list_chapters(db: State<Db>, project_id: String) -> AppResult<Vec<Chapter>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    chapters_repository::list_by_project(&conn, &project_id)
}

#[tauri::command]
pub fn get_chapter(db: State<Db>, id: String) -> AppResult<Option<Chapter>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    chapters_repository::get(&conn, &id)
}

#[tauri::command]
pub fn update_chapter_meta(db: State<Db>, id: String, title: String, status: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    chapters_repository::update_meta(&conn, &id, &title, &status)
}

#[tauri::command]
pub fn delete_chapter(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    chapters_repository::soft_delete(&conn, &id)
}
