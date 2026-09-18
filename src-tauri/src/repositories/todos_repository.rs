use crate::error::AppResult;
use crate::models::Todo;
use rusqlite::{params, Connection, OptionalExtension};

fn row_to_todo(row: &rusqlite::Row) -> rusqlite::Result<Todo> {
    Ok(Todo {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        title: row.get("title")?,
        done: row.get::<_, i64>("done")? != 0,
        due_date: row.get("due_date")?,
        memo: row.get("memo")?,
        order_index: row.get("order_index")?,
    })
}
const COLUMNS: &str = "id, project_id, title, done, due_date, memo, order_index";

pub fn list_by_project(conn: &Connection, project_id: &str) -> AppResult<Vec<Todo>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM todos WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY done, order_index"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], row_to_todo)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create(conn: &Connection, project_id: &str, title: &str) -> AppResult<Todo> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(order_index) + 1, 0) FROM todos WHERE project_id = ?1",
        params![project_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO todos (id, project_id, title, done, due_date, memo, order_index, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, NULL, NULL, ?4, ?5, ?5)",
        params![id, project_id, title, next_order, now],
    )?;
    get(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created todo".into()))
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<Todo>> {
    let sql = format!("SELECT {COLUMNS} FROM todos WHERE id = ?1 AND deleted_at IS NULL");
    Ok(conn.query_row(&sql, params![id], row_to_todo).optional()?)
}

pub fn update(conn: &Connection, id: &str, input: &Todo) -> AppResult<Todo> {
    conn.execute(
        "UPDATE todos SET title=?2, done=?3, due_date=?4, memo=?5, order_index=?6, updated_at=?7
         WHERE id=?1 AND deleted_at IS NULL",
        params![
            id,
            input.title,
            input.done as i64,
            input.due_date,
            input.memo,
            input.order_index,
            chrono::Utc::now().to_rfc3339(),
        ],
    )?;
    get(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("todo {id}")))
}

pub fn set_done(conn: &Connection, id: &str, done: bool) -> AppResult<Todo> {
    conn.execute(
        "UPDATE todos SET done=?2, updated_at=?3 WHERE id=?1 AND deleted_at IS NULL",
        params![id, done as i64, chrono::Utc::now().to_rfc3339()],
    )?;
    get(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("todo {id}")))
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE todos SET deleted_at = ?2 WHERE id = ?1",
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
    fn create_toggle_and_list_todos() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        let t = create(&conn, &project.id, "3章の時系列を確認する").unwrap();
        assert!(!t.done);
        let done = set_done(&conn, &t.id, true).unwrap();
        assert!(done.done);

        assert_eq!(list_by_project(&conn, &project.id).unwrap().len(), 1);
        delete(&conn, &t.id).unwrap();
        assert_eq!(list_by_project(&conn, &project.id).unwrap().len(), 0);
    }
}
