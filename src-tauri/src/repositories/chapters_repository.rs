use crate::error::AppResult;
use crate::models::{Chapter, CHAPTER_STATUSES};
use rusqlite::{params, Connection};

fn row_to_chapter(row: &rusqlite::Row) -> rusqlite::Result<Chapter> {
    Ok(Chapter {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        part_id: row.get("part_id")?,
        title: row.get("title")?,
        subtitle: row.get("subtitle")?,
        synopsis: row.get("synopsis")?,
        memo: row.get("memo")?,
        pov: row.get("pov")?,
        status: row.get("status")?,
        start_at: row.get("start_at")?,
        end_at: row.get("end_at")?,
        location: row.get("location")?,
        order_index: row.get("order_index")?,
        char_count: row.get::<_, Option<i64>>("char_count")?.unwrap_or(0),
    })
}

const SELECT: &str = "SELECT c.id, c.project_id, c.part_id, c.title, c.subtitle, c.synopsis,
    c.memo, c.pov, c.status, c.start_at, c.end_at, c.location, c.order_index,
    d.char_count as char_count
    FROM chapters c LEFT JOIN documents d ON d.owner_type = 'chapter' AND d.owner_id = c.id";

pub fn create(conn: &Connection, project_id: &str, part_id: Option<&str>, title: &str) -> AppResult<Chapter> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(order_index) + 1, 0) FROM chapters WHERE project_id = ?1 AND deleted_at IS NULL",
        params![project_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO chapters (id, project_id, part_id, title, status, order_index, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'not_started', ?5, ?6, ?6)",
        params![id, project_id, part_id, title, next_order, now],
    )?;
    super::documents_repository::ensure_owner(conn, "chapter", &id)?;
    get(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created chapter".into()))
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<Chapter>> {
    use rusqlite::OptionalExtension;
    let sql = format!("{SELECT} WHERE c.id = ?1 AND c.deleted_at IS NULL");
    Ok(conn.query_row(&sql, params![id], row_to_chapter).optional()?)
}

pub fn list_by_project(conn: &Connection, project_id: &str) -> AppResult<Vec<Chapter>> {
    let sql = format!("{SELECT} WHERE c.project_id = ?1 AND c.deleted_at IS NULL ORDER BY c.order_index");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], row_to_chapter)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn update_meta(conn: &Connection, id: &str, title: &str, status: &str) -> AppResult<()> {
    if !CHAPTER_STATUSES.contains(&status) {
        return Err(crate::error::AppError::Other(format!("invalid chapter status: {status}")));
    }
    conn.execute(
        "UPDATE chapters SET title = ?2, status = ?3, updated_at = ?4 WHERE id = ?1",
        params![id, title, status, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

pub fn soft_delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE chapters SET deleted_at = ?2 WHERE id = ?1",
        params![id, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::projects_repository;

    #[test]
    fn create_list_and_update_chapter() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        let chapter = create(&conn, &project.id, None, "第1章").unwrap();
        assert_eq!(chapter.status, "not_started");
        assert_eq!(chapter.char_count, 0);

        update_meta(&conn, &chapter.id, "第1章 改題", "writing").unwrap();
        let updated = get(&conn, &chapter.id).unwrap().unwrap();
        assert_eq!(updated.title, "第1章 改題");
        assert_eq!(updated.status, "writing");

        assert_eq!(list_by_project(&conn, &project.id).unwrap().len(), 1);
    }
}
