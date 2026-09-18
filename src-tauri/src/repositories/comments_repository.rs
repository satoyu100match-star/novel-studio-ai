use crate::error::AppResult;
use crate::models::Comment;
use rusqlite::{params, Connection, OptionalExtension};

fn row_to_comment(row: &rusqlite::Row) -> rusqlite::Result<Comment> {
    Ok(Comment {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        owner_type: row.get("owner_type")?,
        owner_id: row.get("owner_id")?,
        anchor_start: row.get("anchor_start")?,
        anchor_end: row.get("anchor_end")?,
        quote: row.get("quote")?,
        body: row.get("body")?,
        resolved: row.get::<_, i64>("resolved")? != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}
const COLUMNS: &str =
    "id, project_id, owner_type, owner_id, anchor_start, anchor_end, quote, body, resolved, created_at, updated_at";

pub fn list_for_owner(conn: &Connection, owner_type: &str, owner_id: &str) -> AppResult<Vec<Comment>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM comments WHERE owner_type = ?1 AND owner_id = ?2 AND deleted_at IS NULL ORDER BY created_at"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![owner_type, owner_id], row_to_comment)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    project_id: &str,
    owner_type: &str,
    owner_id: &str,
    anchor_start: Option<i64>,
    anchor_end: Option<i64>,
    quote: Option<&str>,
    body: &str,
) -> AppResult<Comment> {
    if owner_type != "chapter" && owner_type != "scene" {
        return Err(crate::error::AppError::Other(format!("invalid comment owner_type: {owner_type}")));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO comments (id, project_id, owner_type, owner_id, anchor_start, anchor_end, quote, body, resolved, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, ?9)",
        params![id, project_id, owner_type, owner_id, anchor_start, anchor_end, quote, body, now],
    )?;
    get(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created comment".into()))
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<Comment>> {
    let sql = format!("SELECT {COLUMNS} FROM comments WHERE id = ?1 AND deleted_at IS NULL");
    Ok(conn.query_row(&sql, params![id], row_to_comment).optional()?)
}

pub fn set_resolved(conn: &Connection, id: &str, resolved: bool) -> AppResult<Comment> {
    conn.execute(
        "UPDATE comments SET resolved=?2, updated_at=?3 WHERE id=?1 AND deleted_at IS NULL",
        params![id, resolved as i64, chrono::Utc::now().to_rfc3339()],
    )?;
    get(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("comment {id}")))
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE comments SET deleted_at = ?2 WHERE id = ?1",
        params![id, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::{chapters_repository, projects_repository};

    #[test]
    fn create_resolve_and_list_comments() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();
        let chapter = chapters_repository::create(&conn, &project.id, None, "第一章").unwrap();

        let c = create(&conn, &project.id, "chapter", &chapter.id, Some(10), Some(20), Some("その夜、"), "ここの時刻表現を見直す").unwrap();
        assert!(!c.resolved);
        let resolved = set_resolved(&conn, &c.id, true).unwrap();
        assert!(resolved.resolved);

        assert_eq!(list_for_owner(&conn, "chapter", &chapter.id).unwrap().len(), 1);
        assert!(create(&conn, &project.id, "invalid", &chapter.id, None, None, None, "x").is_err());

        delete(&conn, &c.id).unwrap();
        assert_eq!(list_for_owner(&conn, "chapter", &chapter.id).unwrap().len(), 0);
    }
}
