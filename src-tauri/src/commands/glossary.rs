use crate::db::Db;
use crate::error::AppResult;
use crate::models::GlossaryEntry;
use crate::repositories::glossary_repository;
use tauri::State;

#[tauri::command]
pub fn create_glossary_entry(db: State<Db>, project_id: String, input: GlossaryEntry) -> AppResult<GlossaryEntry> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    glossary_repository::create(&conn, &project_id, &input)
}

#[tauri::command]
pub fn list_glossary_entries(db: State<Db>, project_id: String) -> AppResult<Vec<GlossaryEntry>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    glossary_repository::list_by_project(&conn, &project_id)
}

#[tauri::command]
pub fn update_glossary_entry(db: State<Db>, id: String, input: GlossaryEntry) -> AppResult<GlossaryEntry> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    glossary_repository::update(&conn, &id, &input)
}

#[tauri::command]
pub fn delete_glossary_entry(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    glossary_repository::soft_delete(&conn, &id)
}
