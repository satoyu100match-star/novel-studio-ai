use crate::db::Db;
use crate::error::AppResult;
use crate::models::BackupInfo;
use tauri::State;

/// Phase 8: Crash Recovery。前回終了時に正常終了マーカーが残っていなければ
/// (=クリーンシャットダウンできなかった)trueを返す。実データは常に
/// SQLiteへ同期的に保存されているため喪失の心配はないが、フロント側で
/// 一度だけ案内バナーを出すためのフラグ。
pub struct UncleanShutdownFlag(pub bool);

#[tauri::command]
pub fn was_unclean_shutdown(flag: State<UncleanShutdownFlag>) -> bool {
    flag.0
}

#[tauri::command]
pub fn list_backups(db: State<Db>) -> AppResult<Vec<BackupInfo>> {
    db.list_backups()
}

#[tauri::command]
pub fn backup_now(db: State<Db>) -> AppResult<BackupInfo> {
    let path = db.backup_now()?;
    let backups = db.list_backups()?;
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or_default().to_string();
    backups
        .into_iter()
        .find(|b| b.filename == filename)
        .ok_or_else(|| crate::error::AppError::Other("バックアップの作成後に情報を取得できませんでした。".into()))
}

/// バックアップからの復元。取り消せない操作(現在のDBの内容を丸ごと
/// 置き換える)のため、フロント側で必ず確認ダイアログを挟む。
#[tauri::command]
pub fn restore_backup(db: State<Db>, filename: String) -> AppResult<()> {
    db.restore_from_backup(&filename)
}
