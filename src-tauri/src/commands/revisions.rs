use crate::db::Db;
use crate::error::AppResult;
use crate::models::{Document, Revision};
use crate::repositories::{documents_repository, revisions_repository};
use tauri::State;

#[tauri::command]
pub fn list_revisions(db: State<Db>, owner_type: String, owner_id: String) -> AppResult<Vec<Revision>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    revisions_repository::list_by_owner(&conn, &owner_type, &owner_id)
}

#[tauri::command]
pub fn get_revision(db: State<Db>, id: String) -> AppResult<Option<Revision>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    revisions_repository::get(&conn, &id)
}

/// ユーザーが明示的に押した「現在の内容をスナップショット保存」。
/// 現在の本文をdocumentsから読み、そのまま履歴として保存する。
#[tauri::command]
pub fn create_manual_revision(
    db: State<Db>,
    owner_type: String,
    owner_id: String,
    label: Option<String>,
) -> AppResult<Revision> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let body = documents_repository::get(&conn, &owner_type, &owner_id)?.map(|d| d.body).unwrap_or_default();
    revisions_repository::create_manual(&conn, &owner_type, &owner_id, &body, label.as_deref())
}

/// 指定履歴の内容を現在の本文として復元する。復元前の状態は自動で
/// バックアップされる(`revisions_repository::restore`参照)。
#[tauri::command]
pub fn restore_revision(db: State<Db>, id: String) -> AppResult<Document> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    revisions_repository::restore(&conn, &id)
}
