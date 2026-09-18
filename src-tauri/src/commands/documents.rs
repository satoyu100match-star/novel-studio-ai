use crate::db::Db;
use crate::error::AppResult;
use crate::models::Document;
use crate::repositories::{documents_repository, revisions_repository};
use tauri::State;

#[tauri::command]
pub fn get_document(db: State<Db>, owner_type: String, owner_id: String) -> AppResult<Option<Document>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    documents_repository::get(&conn, &owner_type, &owner_id)
}

/// 保存のたびに、章/シーンの本文をそのままdocumentsへ書き込む(=これが
/// 事実上のCrash Recovery機構。800msデバウンスでSQLiteへ同期保存される
/// ため、localStorageのみに頼る設計とは違い、アプリが突然終了しても
/// 直前の保存以降の入力しか失われない)。あわせてPhase8のバージョン履歴
/// (`revisions_repository::maybe_create_auto`)を考慮し、一定間隔・
/// 内容変化があれば自動スナップショットも残す。
#[tauri::command]
pub fn save_document_body(
    db: State<Db>,
    owner_type: String,
    owner_id: String,
    body: String,
) -> AppResult<Document> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let doc = documents_repository::save_body(&conn, &owner_type, &owner_id, &body)?;
    if owner_type == "chapter" || owner_type == "scene" {
        revisions_repository::maybe_create_auto(&conn, &owner_type, &owner_id, &body)?;
    }
    Ok(doc)
}

#[tauri::command]
pub fn project_char_count(db: State<Db>, project_id: String) -> AppResult<i64> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    documents_repository::total_char_count_for_project(&conn, &project_id)
}
