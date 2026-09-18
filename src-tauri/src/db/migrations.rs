//! Migration runner.
//!
//! Rules (see docs/DATABASE.md and CLAUDE.md):
//! - Every schema change is a new, numbered, append-only `.sql` file under
//!   `src-tauri/migrations/`. Never edit a migration that has already
//!   shipped -- add a new one instead.
//! - Applied migrations are tracked in `schema_migrations` so upgrading an
//!   existing user's database only ever applies what's new.
//! - Each migration runs inside a transaction; a failure rolls back and
//!   stops startup rather than leaving a half-migrated database.

use crate::error::{AppError, AppResult};
use rusqlite::Connection;

/// Ordered list of (version, name, sql). Version numbers must be unique,
/// ascending, and never reused. Add new entries at the end.
const MIGRATIONS: &[(i64, &str, &str)] = &[
    (1, "init", include_str!("../../migrations/0001_init.sql")),
    (2, "phase1_core", include_str!("../../migrations/0002_phase1_core.sql")),
    (3, "phase3_worldbuilding", include_str!("../../migrations/0003_phase3_worldbuilding.sql")),
    (4, "phase4_story", include_str!("../../migrations/0004_phase4_story.sql")),
    (5, "phase5_ai", include_str!("../../migrations/0005_phase5_ai.sql")),
    (6, "phase7_analysis", include_str!("../../migrations/0006_phase7_analysis.sql")),
    (7, "phase8_safety", include_str!("../../migrations/0007_phase8_safety.sql")),
];

pub fn run(conn: &mut Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version     INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            applied_at  TEXT NOT NULL
        );",
    )?;

    let already_applied: Vec<i64> = {
        let mut stmt = conn.prepare("SELECT version FROM schema_migrations ORDER BY version")?;
        let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    for (version, name, sql) in MIGRATIONS {
        if already_applied.contains(version) {
            continue;
        }

        let tx = conn.transaction()?;
        tx.execute_batch(sql).map_err(|e| {
            AppError::Migration(format!("migration {version} ({name}) failed: {e}"))
        })?;
        tx.execute(
            "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![version, name, chrono::Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
        log::info!("applied migration {version} ({name})");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_all_migrations_and_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        run(&mut conn).unwrap();

        // app_settings table from 0001_init.sql should now exist.
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='app_settings'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Running again must not error and must not re-apply anything.
        run(&mut conn).unwrap();
        let applied: i64 = conn
            .query_row("SELECT count(*) FROM schema_migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(applied, MIGRATIONS.len() as i64);
    }
}
