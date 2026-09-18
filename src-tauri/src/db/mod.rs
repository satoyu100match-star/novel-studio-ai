//! Local SQLite access.
//!
//! One connection, guarded by a mutex, shared across the app via Tauri's
//! managed state. This is intentionally simple for Phase 0 -- a
//! connection pool can be introduced later if profiling on real long-form
//! manuscripts shows it's needed (see docs/ARCHITECTURE.md, principle:
//! don't add complexity ahead of a measured need).

pub mod backup;
pub mod migrations;

use crate::error::{AppError, AppResult};
use crate::models::BackupInfo;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct Db {
    pub conn: Mutex<Connection>,
    pub path: PathBuf,
    pub backups_dir: PathBuf,
}

fn path_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(suffix);
    PathBuf::from(os)
}

impl Db {
    /// Open (creating if needed) the SQLite database at `path` and bring
    /// its schema up to date.
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut conn = Connection::open(path)?;
        // WAL mode: readers don't block the writer, which matters once the
        // AI assistant and manuscript autosave can be in flight together.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", true)?;

        migrations::run(&mut conn)?;

        let backups_dir = path.parent().map(|p| p.join("backups")).unwrap_or_else(|| PathBuf::from("backups"));

        Ok(Db {
            conn: Mutex::new(conn),
            path: path.to_path_buf(),
            backups_dir,
        })
    }

    /// Phase 8: Auto Backup。今すぐDB全体のスナップショットを1件作成する。
    pub fn backup_now(&self) -> AppResult<PathBuf> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        backup::backup_now(&conn, &self.backups_dir)
    }

    pub fn list_backups(&self) -> AppResult<Vec<BackupInfo>> {
        backup::list_backups(&self.backups_dir)
    }

    /// Phase 8: バックアップからの復元。現在の接続を安全に閉じてから
    /// DBファイルをバックアップの内容で上書きし、新しい接続を張り直す。
    /// WAL/SHMのサイドカーファイルは復元前の状態を引きずらないよう削除する
    /// (`VACUUM INTO`で作られたバックアップ自体はWALを使わない単一
    /// ファイルのため、復元後は素のDELETEジャーナルから始まり、直後に
    /// 改めてWALモードへ切り替える)。復元後にマイグレーションを再実行
    /// するのは、バックアップが古いアプリバージョン由来でスキーマが
    /// 古い可能性があるため。
    pub fn restore_from_backup(&self, filename: &str) -> AppResult<()> {
        backup::validate_backup_filename(filename)?;
        let backup_path = self.backups_dir.join(filename);
        if !backup_path.exists() {
            return Err(AppError::NotFound(format!("バックアップファイルが見つかりません: {filename}")));
        }

        let mut guard = self.conn.lock().expect("db mutex poisoned");
        let old = std::mem::replace(&mut *guard, Connection::open_in_memory()?);
        drop(old);

        let _ = std::fs::remove_file(path_with_suffix(&self.path, "-wal"));
        let _ = std::fs::remove_file(path_with_suffix(&self.path, "-shm"));
        std::fs::copy(&backup_path, &self.path)?;

        let mut new_conn = Connection::open(&self.path)?;
        new_conn.pragma_update(None, "journal_mode", "WAL")?;
        new_conn.pragma_update(None, "foreign_keys", true)?;
        migrations::run(&mut new_conn)?;

        *guard = new_conn;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn opens_file_backed_db_and_persists_across_reopen() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("nested").join("novel-studio-ai.db");

        {
            let db = Db::open(&db_path).unwrap();
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                rusqlite::params!["theme", "dark", "2026-01-01T00:00:00Z"],
            )
            .unwrap();
        }

        assert!(db_path.exists(), "db file should have been created");

        let db = Db::open(&db_path).unwrap();
        let conn = db.conn.lock().unwrap();
        let value: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                rusqlite::params!["theme"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "dark");
    }

    #[test]
    fn backup_and_restore_round_trip() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("app.db");
        let db = Db::open(&db_path).unwrap();

        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO app_settings (key, value, updated_at) VALUES ('k', 'before', '2026-01-01T00:00:00Z')
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [],
            )
            .unwrap();
        }

        let backup_path = db.backup_now().unwrap();
        let filename = backup_path.file_name().unwrap().to_str().unwrap().to_string();

        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO app_settings (key, value, updated_at) VALUES ('k', 'after', '2026-01-01T00:00:01Z')
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [],
            )
            .unwrap();
        }

        db.restore_from_backup(&filename).unwrap();

        let conn = db.conn.lock().unwrap();
        let value: String =
            conn.query_row("SELECT value FROM app_settings WHERE key = 'k'", [], |r| r.get(0)).unwrap();
        assert_eq!(value, "before");
    }

    #[test]
    fn restore_rejects_unknown_or_unsafe_filenames() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("app.db");
        let db = Db::open(&db_path).unwrap();

        assert!(db.restore_from_backup("../evil.db").is_err());
        assert!(db.restore_from_backup("backup_does_not_exist.db").is_err());
    }
}
