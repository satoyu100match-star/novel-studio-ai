use crate::db::Db;
use crate::error::AppResult;
use crate::models::{Project, ProjectInput};
use crate::repositories::projects_repository;
use crate::sample_project;
use tauri::State;

#[tauri::command]
pub fn create_project(db: State<Db>, input: ProjectInput) -> AppResult<Project> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    projects_repository::create(&conn, &input, false)
}

/// Phase 10: サンプル作品を作成する。初回起動時のOnboardingから、または
/// 作品一覧画面からいつでも呼び出せる(何度でも作れる -- 既存のサンプルを
/// いじってしまった後にもう一度まっさらな状態で触りたい、というニーズに
/// 対応するため、重複チェックはしない)。
#[tauri::command]
pub fn create_sample_project(db: State<Db>) -> AppResult<Project> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    sample_project::create(&conn)
}

#[tauri::command]
pub fn list_projects(db: State<Db>) -> AppResult<Vec<Project>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    projects_repository::list(&conn)
}

#[tauri::command]
pub fn get_project(db: State<Db>, id: String) -> AppResult<Option<Project>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    projects_repository::get(&conn, &id)
}

#[tauri::command]
pub fn update_project(db: State<Db>, id: String, input: ProjectInput) -> AppResult<Project> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    projects_repository::update(&conn, &id, &input)
}

#[tauri::command]
pub fn delete_project(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    projects_repository::soft_delete(&conn, &id)
}
