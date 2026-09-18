use crate::error::AppResult;
use crate::models::Part;
use rusqlite::{params, Connection};

fn row_to_part(row: &rusqlite::Row) -> rusqlite::Result<Part> {
    Ok(Part {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        title: row.get("title")?,
        order_index: row.get("order_index")?,
    })
}

pub fn create(conn: &Connection, project_id: &str, title: &str) -> AppResult<Part> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(order_index) + 1, 0) FROM parts WHERE project_id = ?1 AND deleted_at IS NULL",
        params![project_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO parts (id, project_id, title, order_index, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![id, project_id, title, next_order, now],
    )?;
    Ok(Part { id, project_id: project_id.to_string(), title: title.to_string(), order_index: next_order })
}

pub fn list_by_project(conn: &Connection, project_id: &str) -> AppResult<Vec<Part>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, order_index FROM parts
         WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY order_index",
    )?;
    let rows = stmt.query_map(params![project_id], row_to_part)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn rename(conn: &Connection, id: &str, title: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE parts SET title = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, title, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

pub fn soft_delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE parts SET deleted_at = ?2 WHERE id = ?1",
        params![id, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::projects_repository;
    use crate::models::ProjectInput;

    #[test]
    fn create_and_list_parts_ordered() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        create(&conn, &project.id, "第一部 王都編").unwrap();
        create(&conn, &project.id, "第二部 辺境編").unwrap();

        let parts = list_by_project(&conn, &project.id).unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].order_index, 0);
        assert_eq!(parts[1].order_index, 1);
    }
}
