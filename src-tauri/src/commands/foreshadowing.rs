use crate::db::Db;
use crate::error::AppResult;
use crate::models::Foreshadowing;
use crate::repositories::foreshadowing_repository;
use tauri::State;

#[tauri::command]
pub fn list_foreshadowings(db: State<Db>, project_id: String) -> AppResult<Vec<Foreshadowing>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    foreshadowing_repository::list_by_project(&conn, &project_id)
}

#[tauri::command]
pub fn create_foreshadowing(db: State<Db>, project_id: String, title: String) -> AppResult<Foreshadowing> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    foreshadowing_repository::create(&conn, &project_id, &title)
}

#[tauri::command]
pub fn update_foreshadowing(db: State<Db>, id: String, input: Foreshadowing) -> AppResult<Foreshadowing> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    foreshadowing_repository::update(&conn, &id, &input)
}

#[tauri::command]
pub fn delete_foreshadowing(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    foreshadowing_repository::delete(&conn, &id)
}
