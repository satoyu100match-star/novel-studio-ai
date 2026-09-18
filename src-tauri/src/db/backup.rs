//! Auto Backup(DBファイル全体のスナップショット)。
//!
//! `rusqlite`の`backup`機能(追加クレート機能)を有効にするのではなく、
//! SQLite標準SQLの`VACUUM INTO`を使う: 既に有効化済みの`bundled`機能の
//! SQLiteだけで完結し、WAL中の未コミットデータも含めて一貫したスナップ
//! ショットを単一ファイルへ書き出せる(コピー中の書き込みで壊れた
//! バックアップができる心配がない)。

use crate::error::{AppError, AppResult};
use crate::models::BackupInfo;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// 起動時に自動保持しておくバックアップの世代数。手動バックアップも
/// 同じ間引き対象に含める(区別せず「最新N件」を残す方針)。
pub const MAX_BACKUPS: usize = 10;

fn backup_filename(now: chrono::DateTime<chrono::Utc>) -> String {
    format!("backup_{}.db", now.format("%Y%m%dT%H%M%SZ"))
}

/// バックアップ先ファイルパスを`VACUUM INTO`のSQL文字列リテラルとして
/// 安全に埋め込む(シングルクォートをエスケープ)。パス自体は
/// `backup_filename`で生成した既知の形式のみを使うため、実運用上は
/// エスケープ不要なケースがほとんどだが、Windowsのユーザー名にアポスト
/// ロフィが含まれるなど、親ディレクトリ側に由来する想定外の文字が
/// 混ざる可能性はゼロではないため、念のため防御的に処理する。
fn quote_sql_string(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

/// 今すぐバックアップを1件作成し、`MAX_BACKUPS`件を超える古いものを
/// 削除する。
pub fn backup_now(conn: &Connection, backups_dir: &Path) -> AppResult<PathBuf> {
    std::fs::create_dir_all(backups_dir)?;
    let filename = backup_filename(chrono::Utc::now());
    let dest = backups_dir.join(&filename);
    let sql = format!("VACUUM INTO '{}'", quote_sql_string(&dest));
    conn.execute_batch(&sql)?;
    prune_backups(backups_dir, MAX_BACKUPS)?;
    Ok(dest)
}

/// バックアップファイル名の昇順(=作成日時の昇順、`backup_YYYYMMDDTHHMMSSZ.db`
/// という命名のため文字列ソート=時系列ソートになる)で並べ、
/// 新しい方から`keep`件だけ残して残りを削除する。
pub fn prune_backups(backups_dir: &Path, keep: usize) -> AppResult<()> {
    let mut files = list_backup_files(backups_dir)?;
    files.sort();
    if files.len() > keep {
        for old in &files[..files.len() - keep] {
            let _ = std::fs::remove_file(backups_dir.join(old));
        }
    }
    Ok(())
}

fn list_backup_files(backups_dir: &Path) -> AppResult<Vec<String>> {
    if !backups_dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(backups_dir)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if name.starts_with("backup_") && name.ends_with(".db") {
                names.push(name.to_string());
            }
        }
    }
    Ok(names)
}

pub fn list_backups(backups_dir: &Path) -> AppResult<Vec<BackupInfo>> {
    let mut names = list_backup_files(backups_dir)?;
    names.sort();
    names.reverse(); // 新しい順
    let mut out = Vec::new();
    for name in names {
        let meta = std::fs::metadata(backups_dir.join(&name))?;
        // ファイル名の "backup_YYYYMMDDTHHMMSSZ.db" から作成日時を復元する
        // (ファイルシステムのmtimeより、バックアップ実行時刻そのものの方が
        // 意味的に正確なため)。
        let created_at = name
            .strip_prefix("backup_")
            .and_then(|s| s.strip_suffix(".db"))
            .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%SZ").ok())
            .map(|dt| dt.and_utc().to_rfc3339())
            .unwrap_or_default();
        out.push(BackupInfo { filename: name, created_at, size_bytes: meta.len() });
    }
    Ok(out)
}

/// バックアップファイルから現在のDBを復元する。呼び出し側(`Db::
/// restore_from_backup`)が接続を安全に閉じてから呼ぶ前提。
pub fn validate_backup_filename(filename: &str) -> AppResult<()> {
    let valid = filename.starts_with("backup_")
        && filename.ends_with(".db")
        && !filename.contains('/')
        && !filename.contains('\\')
        && !filename.contains("..");
    if !valid {
        return Err(AppError::Other("不正なバックアップファイル名です。".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn backup_now_creates_a_restorable_snapshot_and_prunes_old_ones() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("app.db");
        let backups_dir = dir.path().join("backups");

        let mut conn = Connection::open(&db_path).unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES ('k', 'v', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();

        let backup_path = backup_now(&conn, &backups_dir).unwrap();
        assert!(backup_path.exists());

        let backups = list_backups(&backups_dir).unwrap();
        assert_eq!(backups.len(), 1);

        // バックアップは独立したファイルなので、元のconnを閉じずに開ける。
        let backup_conn = Connection::open(&backup_path).unwrap();
        let value: String =
            backup_conn.query_row("SELECT value FROM app_settings WHERE key = 'k'", [], |r| r.get(0)).unwrap();
        assert_eq!(value, "v");
    }

    #[test]
    fn prune_backups_keeps_only_the_newest_n() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(&dir).unwrap();
        for i in 0..5 {
            std::fs::write(dir.path().join(format!("backup_2026010{i}T000000Z.db")), "x").unwrap();
        }
        prune_backups(dir.path(), 2).unwrap();
        let remaining = list_backup_files(dir.path()).unwrap();
        assert_eq!(remaining.len(), 2);
        assert!(remaining.contains(&"backup_20260104T000000Z.db".to_string()));
        assert!(remaining.contains(&"backup_20260103T000000Z.db".to_string()));
    }

    #[test]
    fn rejects_unsafe_backup_filenames() {
        assert!(validate_backup_filename("backup_20260101T000000Z.db").is_ok());
        assert!(validate_backup_filename("../evil.db").is_err());
        assert!(validate_backup_filename("not_a_backup.db").is_err());
    }
}
