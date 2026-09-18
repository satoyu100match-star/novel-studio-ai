use crate::error::AppResult;
use crate::models::{Foreshadowing, FORESHADOWING_STATUSES};
use rusqlite::{params, Connection, OptionalExtension};

fn row_to_foreshadowing(row: &rusqlite::Row) -> rusqlite::Result<Foreshadowing> {
    Ok(Foreshadowing {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        title: row.get("title")?,
        detail: row.get("detail")?,
        status: row.get("status")?,
        planted_chapter_id: row.get("planted_chapter_id")?,
        payoff_chapter_id: row.get("payoff_chapter_id")?,
        memo: row.get("memo")?,
    })
}
const COLUMNS: &str = "id, project_id, title, detail, status, planted_chapter_id, payoff_chapter_id, memo";

pub fn list_by_project(conn: &Connection, project_id: &str) -> AppResult<Vec<Foreshadowing>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM foreshadowings WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY created_at"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], row_to_foreshadowing)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create(conn: &Connection, project_id: &str, title: &str) -> AppResult<Foreshadowing> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO foreshadowings (id, project_id, title, detail, status, planted_chapter_id, payoff_chapter_id, memo, created_at, updated_at)
         VALUES (?1, ?2, ?3, NULL, 'idea', NULL, NULL, NULL, ?4, ?4)",
        params![id, project_id, title, now],
    )?;
    get(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created foreshadowing".into()))
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<Foreshadowing>> {
    let sql = format!("SELECT {COLUMNS} FROM foreshadowings WHERE id = ?1 AND deleted_at IS NULL");
    Ok(conn.query_row(&sql, params![id], row_to_foreshadowing).optional()?)
}

pub fn update(conn: &Connection, id: &str, input: &Foreshadowing) -> AppResult<Foreshadowing> {
    if !FORESHADOWING_STATUSES.contains(&input.status.as_str()) {
        return Err(crate::error::AppError::Other(format!("invalid foreshadowing status: {}", input.status)));
    }
    conn.execute(
        "UPDATE foreshadowings SET title=?2, detail=?3, status=?4, planted_chapter_id=?5, payoff_chapter_id=?6, memo=?7, updated_at=?8
         WHERE id=?1 AND deleted_at IS NULL",
        params![
            id,
            input.title,
            input.detail,
            input.status,
            input.planted_chapter_id,
            input.payoff_chapter_id,
            input.memo,
            chrono::Utc::now().to_rfc3339(),
        ],
    )?;
    get(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("foreshadowing {id}")))
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE foreshadowings SET deleted_at = ?2 WHERE id = ?1",
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
    fn create_update_status_and_reject_invalid_status() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        let f = create(&conn, &project.id, "懐中時計の謎").unwrap();
        assert_eq!(f.status, "idea");

        let mut input = f.clone();
        input.status = "planted".into();
        let updated = update(&conn, &f.id, &input).unwrap();
        assert_eq!(updated.status, "planted");

        let mut bad = updated.clone();
        bad.status = "not_a_status".into();
        assert!(update(&conn, &f.id, &bad).is_err());

        assert_eq!(list_by_project(&conn, &project.id).unwrap().len(), 1);
    }
}
