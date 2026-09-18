use crate::db::Db;
use crate::error::AppResult;
use crate::models::Location;
use crate::repositories::locations_repository;
use tauri::State;

#[tauri::command]
pub fn create_location(db: State<Db>, project_id: String, input: Location) -> AppResult<Location> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    locations_repository::create(&conn, &project_id, &input)
}

#[tauri::command]
pub fn list_locations(db: State<Db>, project_id: String) -> AppResult<Vec<Location>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    locations_repository::list_by_project(&conn, &project_id)
}

#[tauri::command]
pub fn update_location(db: State<Db>, id: String, input: Location) -> AppResult<Location> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    locations_repository::update(&conn, &id, &input)
}

#[tauri::command]
pub fn delete_location(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    locations_repository::soft_delete(&conn, &id)
}
