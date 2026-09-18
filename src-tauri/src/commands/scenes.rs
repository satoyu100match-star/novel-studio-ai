use crate::db::Db;
use crate::error::AppResult;
use crate::models::Scene;
use crate::repositories::scenes_repository;
use tauri::State;

#[tauri::command]
pub fn create_scene(db: State<Db>, chapter_id: String, title: String) -> AppResult<Scene> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    scenes_repository::create(&conn, &chapter_id, &title)
}

#[tauri::command]
pub fn list_scenes(db: State<Db>, chapter_id: String) -> AppResult<Vec<Scene>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    scenes_repository::list_by_chapter(&conn, &chapter_id)
}

#[tauri::command]
pub fn get_scene(db: State<Db>, id: String) -> AppResult<Option<Scene>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    scenes_repository::get(&conn, &id)
}

#[tauri::command]
pub fn rename_scene(db: State<Db>, id: String, title: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    scenes_repository::rename(&conn, &id, &title)
}

#[tauri::command]
pub fn delete_scene(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    scenes_repository::soft_delete(&conn, &id)
}
