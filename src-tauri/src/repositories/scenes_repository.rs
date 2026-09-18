use crate::error::AppResult;
use crate::models::Scene;
use rusqlite::{params, Connection};

fn row_to_scene(row: &rusqlite::Row) -> rusqlite::Result<Scene> {
    Ok(Scene {
        id: row.get("id")?,
        chapter_id: row.get("chapter_id")?,
        title: row.get("title")?,
        summary: row.get("summary")?,
        pov_character: row.get("pov_character")?,
        location: row.get("location")?,
        event_date: row.get("event_date")?,
        start_time: row.get("start_time")?,
        end_time: row.get("end_time")?,
        purpose: row.get("purpose")?,
        conflict: row.get("conflict")?,
        result: row.get("result")?,
        emotion: row.get("emotion")?,
        memo: row.get("memo")?,
        order_index: row.get("order_index")?,
        char_count: row.get::<_, Option<i64>>("char_count")?.unwrap_or(0),
    })
}

const SELECT: &str = "SELECT s.id, s.chapter_id, s.title, s.summary, s.pov_character, s.location,
    s.event_date, s.start_time, s.end_time, s.purpose, s.conflict, s.result, s.emotion, s.memo,
    s.order_index, d.char_count as char_count
    FROM scenes s LEFT JOIN documents d ON d.owner_type = 'scene' AND d.owner_id = s.id";

pub fn create(conn: &Connection, chapter_id: &str, title: &str) -> AppResult<Scene> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(order_index) + 1, 0) FROM scenes WHERE chapter_id = ?1 AND deleted_at IS NULL",
        params![chapter_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO scenes (id, chapter_id, title, order_index, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![id, chapter_id, title, next_order, now],
    )?;
    super::documents_repository::ensure_owner(conn, "scene", &id)?;
    get(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created scene".into()))
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<Scene>> {
    use rusqlite::OptionalExtension;
    let sql = format!("{SELECT} WHERE s.id = ?1 AND s.deleted_at IS NULL");
    Ok(conn.query_row(&sql, params![id], row_to_scene).optional()?)
}

pub fn list_by_chapter(conn: &Connection, chapter_id: &str) -> AppResult<Vec<Scene>> {
    let sql = format!("{SELECT} WHERE s.chapter_id = ?1 AND s.deleted_at IS NULL ORDER BY s.order_index");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![chapter_id], row_to_scene)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn rename(conn: &Connection, id: &str, title: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE scenes SET title = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, title, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

pub fn soft_delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE scenes SET deleted_at = ?2 WHERE id = ?1",
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
    fn create_and_list_scenes() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();
        let chapter = chapters_repository::create(&conn, &project.id, None, "第1章").unwrap();

        create(&conn, &chapter.id, "Scene 1").unwrap();
        create(&conn, &chapter.id, "Scene 2").unwrap();

        let scenes = list_by_chapter(&conn, &chapter.id).unwrap();
        assert_eq!(scenes.len(), 2);
        assert_eq!(scenes[1].title, "Scene 2");
    }
}
