use crate::error::AppResult;
use crate::models::GlossaryEntry;
use rusqlite::{params, Connection, OptionalExtension};

fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<GlossaryEntry> {
    Ok(GlossaryEntry {
        id: row.get(0)?,
        project_id: row.get(1)?,
        term: row.get(2)?,
        reading: row.get(3)?,
        definition: row.get(4)?,
    })
}
const COLUMNS: &str = "id, project_id, term, reading, definition";

pub fn create(conn: &Connection, project_id: &str, e: &GlossaryEntry) -> AppResult<GlossaryEntry> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO glossary_entries (id, project_id, term, reading, definition, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?6)",
        params![id, project_id, e.term, e.reading, e.definition, now],
    )?;
    get(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created glossary entry".into()))
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<GlossaryEntry>> {
    let sql = format!("SELECT {COLUMNS} FROM glossary_entries WHERE id = ?1 AND deleted_at IS NULL");
    Ok(conn.query_row(&sql, params![id], row_to_entry).optional()?)
}

pub fn list_by_project(conn: &Connection, project_id: &str) -> AppResult<Vec<GlossaryEntry>> {
    let sql = format!("SELECT {COLUMNS} FROM glossary_entries WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY term");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], row_to_entry)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn update(conn: &Connection, id: &str, e: &GlossaryEntry) -> AppResult<GlossaryEntry> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE glossary_entries SET term=?2, reading=?3, definition=?4, updated_at=?5
         WHERE id=?1 AND deleted_at IS NULL",
        params![id, e.term, e.reading, e.definition, now],
    )?;
    get(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("glossary entry {id}")))
}

pub fn soft_delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE glossary_entries SET deleted_at = ?2 WHERE id = ?1",
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
    fn create_and_list_terms_sorted() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        create(&conn, &project.id, &GlossaryEntry { term: "魔導炉".into(), definition: Some("魔力を動力に変える装置".into()), ..Default::default() }).unwrap();
        create(&conn, &project.id, &GlossaryEntry { term: "王都".into(), ..Default::default() }).unwrap();

        let terms = list_by_project(&conn, &project.id).unwrap();
        assert_eq!(terms.len(), 2);
        assert_eq!(terms[0].term, "王都"); // sorted
    }
}
