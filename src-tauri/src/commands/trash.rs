use crate::db::Db;
use crate::error::AppResult;
use crate::models::TrashItem;
use crate::repositories::trash_repository;
use tauri::State;

#[tauri::command]
pub fn list_trash(db: State<Db>, project_id: String) -> AppResult<Vec<TrashItem>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    trash_repository::list_trash(&conn, &project_id)
}

#[tauri::command]
pub fn restore_trash_item(db: State<Db>, entity_type: String, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    trash_repository::restore_item(&conn, &entity_type, &id)
}

#[tauri::command]
pub fn purge_trash_item(db: State<Db>, entity_type: String, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    trash_repository::purge_item(&conn, &entity_type, &id)
}

/// ゴミ箱を空にする(プロジェクト内の全Soft Delete済みレコードを完全削除)。
/// 取り消せない操作のため、フロント側で必ず確認ダイアログを挟む。
#[tauri::command]
pub fn purge_all_trash(db: State<Db>, project_id: String) -> AppResult<usize> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    trash_repository::purge_all(&conn, &project_id)
}
