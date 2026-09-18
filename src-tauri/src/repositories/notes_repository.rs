use crate::error::AppResult;
use crate::models::Note;
use rusqlite::{params, Connection, OptionalExtension};

fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        project_id: row.get(1)?,
        folder: row.get(2)?,
        title: row.get(3)?,
        body: row.get(4)?,
        updated_at: row.get(5)?,
    })
}
const COLUMNS: &str = "id, project_id, folder, title, body, updated_at";

pub fn create(conn: &Connection, project_id: &str, folder: &str, title: &str) -> AppResult<Note> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO notes (id, project_id, folder, title, body, created_at, updated_at)
         VALUES (?1,?2,?3,?4,'',?5,?5)",
        params![id, project_id, folder, title, now],
    )?;
    get(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created note".into()))
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<Note>> {
    let sql = format!("SELECT {COLUMNS} FROM notes WHERE id = ?1 AND deleted_at IS NULL");
    Ok(conn.query_row(&sql, params![id], row_to_note).optional()?)
}

pub fn list_by_project(conn: &Connection, project_id: &str) -> AppResult<Vec<Note>> {
    let sql = format!("SELECT {COLUMNS} FROM notes WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY updated_at DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], row_to_note)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn save(conn: &Connection, id: &str, folder: &str, title: &str, body: &str) -> AppResult<Note> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE notes SET folder=?2, title=?3, body=?4, updated_at=?5 WHERE id=?1 AND deleted_at IS NULL",
        params![id, folder, title, body, now],
    )?;
    get(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("note {id}")))
}

pub fn soft_delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE notes SET deleted_at = ?2 WHERE id = ?1",
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
    fn create_save_and_list_notes() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        let note = create(&conn, &project.id, "アイデア", "新しい伏線案").unwrap();
        let saved = save(&conn, &note.id, "アイデア", "新しい伏線案", "懐中時計が実は...").unwrap();
        assert_eq!(saved.body, "懐中時計が実は...");

        assert_eq!(list_by_project(&conn, &project.id).unwrap().len(), 1);
    }
}
