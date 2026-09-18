use crate::db::Db;
use crate::error::AppResult;
use crate::models::Note;
use crate::repositories::notes_repository;
use tauri::State;

#[tauri::command]
pub fn create_note(db: State<Db>, project_id: String, folder: String, title: String) -> AppResult<Note> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    notes_repository::create(&conn, &project_id, &folder, &title)
}

#[tauri::command]
pub fn list_notes(db: State<Db>, project_id: String) -> AppResult<Vec<Note>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    notes_repository::list_by_project(&conn, &project_id)
}

#[tauri::command]
pub fn save_note(db: State<Db>, id: String, folder: String, title: String, body: String) -> AppResult<Note> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    notes_repository::save(&conn, &id, &folder, &title, &body)
}

#[tauri::command]
pub fn delete_note(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    notes_repository::soft_delete(&conn, &id)
}
