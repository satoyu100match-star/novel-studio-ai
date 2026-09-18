use crate::error::AppResult;
use crate::models::TimelineEvent;
use rusqlite::{params, Connection, OptionalExtension};

fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<TimelineEvent> {
    Ok(TimelineEvent {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        title: row.get("title")?,
        event_date: row.get("event_date")?,
        description: row.get("description")?,
        chapter_id: row.get("chapter_id")?,
        scene_id: row.get("scene_id")?,
        order_index: row.get("order_index")?,
    })
}
const COLUMNS: &str = "id, project_id, title, event_date, description, chapter_id, scene_id, order_index";

pub fn list_by_project(conn: &Connection, project_id: &str) -> AppResult<Vec<TimelineEvent>> {
    // 作中時系列順ではなく、著者が並べた順(order_index)で返す。event_dateは
    // 自由記述(架空暦を許容するため -- 仕様#33)であり文字列ソートが常に
    // 意味を持つとは限らないため。
    let sql = format!(
        "SELECT {COLUMNS} FROM timeline_events WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY order_index"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], row_to_event)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create(conn: &Connection, project_id: &str, title: &str) -> AppResult<TimelineEvent> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(order_index) + 1, 0) FROM timeline_events WHERE project_id = ?1",
        params![project_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO timeline_events (id, project_id, title, event_date, description, chapter_id, scene_id, order_index, created_at, updated_at)
         VALUES (?1, ?2, ?3, NULL, NULL, NULL, NULL, ?4, ?5, ?5)",
        params![id, project_id, title, next_order, now],
    )?;
    get(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created timeline event".into()))
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<TimelineEvent>> {
    let sql = format!("SELECT {COLUMNS} FROM timeline_events WHERE id = ?1 AND deleted_at IS NULL");
    Ok(conn.query_row(&sql, params![id], row_to_event).optional()?)
}

pub fn update(conn: &Connection, id: &str, input: &TimelineEvent) -> AppResult<TimelineEvent> {
    conn.execute(
        "UPDATE timeline_events SET title=?2, event_date=?3, description=?4, chapter_id=?5, scene_id=?6, order_index=?7, updated_at=?8
         WHERE id=?1 AND deleted_at IS NULL",
        params![
            id,
            input.title,
            input.event_date,
            input.description,
            input.chapter_id,
            input.scene_id,
            input.order_index,
            chrono::Utc::now().to_rfc3339(),
        ],
    )?;
    get(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("timeline event {id}")))
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE timeline_events SET deleted_at = ?2 WHERE id = ?1",
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
    fn create_update_and_list_events() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        let ev = create(&conn, &project.id, "王が失踪する").unwrap();
        let mut input = ev.clone();
        input.event_date = Some("建国暦512年 春".into());
        let updated = update(&conn, &ev.id, &input).unwrap();
        assert_eq!(updated.event_date.as_deref(), Some("建国暦512年 春"));

        assert_eq!(list_by_project(&conn, &project.id).unwrap().len(), 1);
        delete(&conn, &ev.id).unwrap();
        assert_eq!(list_by_project(&conn, &project.id).unwrap().len(), 0);
    }
}
