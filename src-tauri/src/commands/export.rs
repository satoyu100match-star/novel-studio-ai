use crate::db::Db;
use crate::error::AppResult;
use crate::export::{self, ExportOutput, EXPORT_FORMATS};
use tauri::State;

/// Phase 9: Exportで選べる形式の一覧("txt"/"markdown"/"html"/"docx"/"pdf"/
/// "epub")。フロントの選択UIはこの値から動的に構築する(ハードコード
/// した選択肢と実装がズレるのを防ぐため)。
#[tauri::command]
pub fn list_export_formats() -> Vec<String> {
    EXPORT_FORMATS.iter().map(|s| s.to_string()).collect()
}

#[tauri::command]
pub fn export_project(db: State<Db>, project_id: String, format: String) -> AppResult<ExportOutput> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    export::export(&conn, &project_id, &format)
}
