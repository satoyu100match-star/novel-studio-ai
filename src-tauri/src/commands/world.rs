use crate::db::Db;
use crate::error::AppResult;
use crate::models::{WorldCategory, WorldEntry};
use crate::repositories::{tags_repository, world_repository};
use tauri::State;

#[tauri::command]
pub fn list_world_categories(db: State<Db>, project_id: String) -> AppResult<Vec<WorldCategory>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    world_repository::list_categories(&conn, &project_id)
}

#[tauri::command]
pub fn create_world_category(db: State<Db>, project_id: String, name: String) -> AppResult<WorldCategory> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    world_repository::create_category(&conn, &project_id, &name)
}

#[tauri::command]
pub fn create_world_entry(db: State<Db>, project_id: String, input: WorldEntry) -> AppResult<WorldEntry> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    world_repository::create_entry(&conn, &project_id, &input)
}

#[tauri::command]
pub fn list_world_entries(db: State<Db>, project_id: String) -> AppResult<Vec<WorldEntry>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    world_repository::list_entries(&conn, &project_id)
}

#[tauri::command]
pub fn get_world_entry(db: State<Db>, id: String) -> AppResult<Option<WorldEntry>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    world_repository::get_entry(&conn, &id)
}

#[tauri::command]
pub fn update_world_entry(db: State<Db>, id: String, input: WorldEntry) -> AppResult<WorldEntry> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    world_repository::update_entry(&conn, &id, &input)
}

#[tauri::command]
pub fn delete_world_entry(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    world_repository::soft_delete_entry(&conn, &id)
}

#[tauri::command]
pub fn set_world_entry_tags(db: State<Db>, project_id: String, entry_id: String, tags: Vec<String>) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    tags_repository::set_entity_tags(&conn, &project_id, "world_entry", &entry_id, &tags)
}

#[tauri::command]
pub fn get_world_entry_tags(db: State<Db>, entry_id: String) -> AppResult<Vec<String>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    tags_repository::get_entity_tags(&conn, "world_entry", &entry_id)
}
