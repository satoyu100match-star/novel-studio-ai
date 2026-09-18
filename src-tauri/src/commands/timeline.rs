use crate::db::Db;
use crate::error::AppResult;
use crate::models::TimelineEvent;
use crate::repositories::timeline_repository;
use tauri::State;

#[tauri::command]
pub fn list_timeline_events(db: State<Db>, project_id: String) -> AppResult<Vec<TimelineEvent>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    timeline_repository::list_by_project(&conn, &project_id)
}

#[tauri::command]
pub fn create_timeline_event(db: State<Db>, project_id: String, title: String) -> AppResult<TimelineEvent> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    timeline_repository::create(&conn, &project_id, &title)
}

#[tauri::command]
pub fn update_timeline_event(db: State<Db>, id: String, input: TimelineEvent) -> AppResult<TimelineEvent> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    timeline_repository::update(&conn, &id, &input)
}

#[tauri::command]
pub fn delete_timeline_event(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    timeline_repository::delete(&conn, &id)
}
