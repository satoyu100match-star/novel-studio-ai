use crate::error::AppResult;
use crate::models::Document;
use rusqlite::{params, Connection, OptionalExtension};

/// Chapters/scenes are created with an empty document row up front so
/// reads never have to special-case "no document yet" -- see
/// `chapters_repository::create` / `scenes_repository::create`.
pub fn ensure_owner(conn: &Connection, owner_type: &str, owner_id: &str) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO documents (id, owner_type, owner_id, body, char_count, updated_at)
         VALUES (?1, ?2, ?3, '', 0, ?4)",
        params![uuid::Uuid::new_v4().to_string(), owner_type, owner_id, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, owner_type: &str, owner_id: &str) -> AppResult<Option<Document>> {
    Ok(conn
        .query_row(
            "SELECT id, owner_type, owner_id, body, char_count, updated_at FROM documents
             WHERE owner_type = ?1 AND owner_id = ?2",
            params![owner_type, owner_id],
            |row| {
                Ok(Document {
                    id: row.get(0)?,
                    owner_type: row.get(1)?,
                    owner_id: row.get(2)?,
                    body: row.get(3)?,
                    char_count: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .optional()?)
}

/// Character count uses `.chars().count()` (Unicode scalar values), which
/// is the standard approximation for Japanese manuscript character
/// counts -- see docs/MANUSCRIPT.md for the distinction between this
/// running count and actual typeset page count (Phase 2).
pub fn save_body(conn: &Connection, owner_type: &str, owner_id: &str, body: &str) -> AppResult<Document> {
    ensure_owner(conn, owner_type, owner_id)?;
    let char_count = body.chars().count() as i64;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE documents SET body = ?3, char_count = ?4, updated_at = ?5
         WHERE owner_type = ?1 AND owner_id = ?2",
        params![owner_type, owner_id, body, char_count, now],
    )?;
    get(conn, owner_type, owner_id)?.ok_or_else(|| crate::error::AppError::Other("document vanished after save".into()))
}

/// Sums chapter-level and scene-level document char counts separately and
/// adds them, rather than a single UNION ALL query, so a project that
/// tracks manuscript text at the chapter level, the scene level, or a mix
/// of both is counted correctly either way.
pub fn total_char_count_for_project(conn: &Connection, project_id: &str) -> AppResult<i64> {
    let chapter_sum: i64 = conn.query_row(
        "SELECT COALESCE(SUM(d.char_count), 0) FROM documents d
         JOIN chapters c ON d.owner_type = 'chapter' AND d.owner_id = c.id
         WHERE c.project_id = ?1 AND c.deleted_at IS NULL",
        params![project_id],
        |r| r.get(0),
    )?;
    let scene_sum: i64 = conn.query_row(
        "SELECT COALESCE(SUM(d.char_count), 0) FROM documents d
         JOIN scenes s ON d.owner_type = 'scene' AND d.owner_id = s.id
         JOIN chapters c ON s.chapter_id = c.id
         WHERE c.project_id = ?1 AND c.deleted_at IS NULL AND s.deleted_at IS NULL",
        params![project_id],
        |r| r.get(0),
    )?;
    Ok(chapter_sum + scene_sum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::{chapters_repository, projects_repository};

    #[test]
    fn save_body_updates_char_count() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();
        let chapter = chapters_repository::create(&conn, &project.id, None, "第1章").unwrap();

        let doc = save_body(&conn, "chapter", &chapter.id, "こんにちは、世界。").unwrap();
        assert_eq!(doc.char_count, 9);

        let total = total_char_count_for_project(&conn, &project.id).unwrap();
        assert_eq!(total, 9);
    }
}
