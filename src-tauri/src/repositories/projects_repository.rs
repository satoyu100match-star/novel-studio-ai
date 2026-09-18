use crate::error::AppResult;
use crate::models::{Project, ProjectInput};
use rusqlite::{params, Connection, OptionalExtension};

fn row_to_project(row: &rusqlite::Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get("id")?,
        title: row.get("title")?,
        subtitle: row.get("subtitle")?,
        author_name: row.get("author_name")?,
        pen_name: row.get("pen_name")?,
        genre: row.get("genre")?,
        target_audience: row.get("target_audience")?,
        target_length: row.get("target_length")?,
        deadline: row.get("deadline")?,
        synopsis: row.get("synopsis")?,
        theme: row.get("theme")?,
        concept: row.get("concept")?,
        style_memo: row.get("style_memo")?,
        pov_policy: row.get("pov_policy")?,
        tense: row.get("tense")?,
        ai_policy: row.get("ai_policy")?,
        is_sample: row.get::<_, i64>("is_sample")? != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

const COLUMNS: &str = "id, title, subtitle, author_name, pen_name, genre, target_audience,
    target_length, deadline, synopsis, theme, concept, style_memo, pov_policy, tense,
    ai_policy, is_sample, created_at, updated_at";

pub fn create(conn: &Connection, input: &ProjectInput, is_sample: bool) -> AppResult<Project> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO projects (id, title, subtitle, author_name, pen_name, genre,
            target_audience, target_length, deadline, synopsis, theme, concept, style_memo,
            pov_policy, tense, ai_policy, is_sample, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?18)",
        params![
            id,
            input.title,
            input.subtitle,
            input.author_name,
            input.pen_name,
            input.genre,
            input.target_audience,
            input.target_length,
            input.deadline,
            input.synopsis,
            input.theme,
            input.concept,
            input.style_memo,
            input.pov_policy,
            input.tense,
            input.ai_policy,
            is_sample as i64,
            now,
        ],
    )?;
    get(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created project".into()))
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<Project>> {
    let sql = format!("SELECT {COLUMNS} FROM projects WHERE id = ?1 AND deleted_at IS NULL");
    Ok(conn.query_row(&sql, params![id], row_to_project).optional()?)
}

pub fn list(conn: &Connection) -> AppResult<Vec<Project>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM projects WHERE deleted_at IS NULL ORDER BY updated_at DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_project)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn update(conn: &Connection, id: &str, input: &ProjectInput) -> AppResult<Project> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE projects SET title=?2, subtitle=?3, author_name=?4, pen_name=?5, genre=?6,
            target_audience=?7, target_length=?8, deadline=?9, synopsis=?10, theme=?11,
            concept=?12, style_memo=?13, pov_policy=?14, tense=?15, ai_policy=?16, updated_at=?17
         WHERE id=?1 AND deleted_at IS NULL",
        params![
            id,
            input.title,
            input.subtitle,
            input.author_name,
            input.pen_name,
            input.genre,
            input.target_audience,
            input.target_length,
            input.deadline,
            input.synopsis,
            input.theme,
            input.concept,
            input.style_memo,
            input.pov_policy,
            input.tense,
            input.ai_policy,
            now,
        ],
    )?;
    get(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("project {id}")))
}

/// Soft delete (spec #78): projects move to Trash, not gone immediately.
/// Trash UI itself lands in Phase 8; this just sets the flag now so later
/// phases don't need a schema change to introduce it.
pub fn soft_delete(conn: &Connection, id: &str) -> AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE projects SET deleted_at = ?2 WHERE id = ?1",
        params![id, now],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        conn
    }

    #[test]
    fn create_get_update_list_delete_roundtrip() {
        let conn = test_conn();
        let input = ProjectInput {
            title: "星の向こう側".into(),
            genre: Some("ファンタジー".into()),
            ..Default::default()
        };
        let created = create(&conn, &input, false).unwrap();
        assert_eq!(created.title, "星の向こう側");
        assert!(!created.is_sample);

        let fetched = get(&conn, &created.id).unwrap().unwrap();
        assert_eq!(fetched.genre.as_deref(), Some("ファンタジー"));

        let mut updated_input = input.clone();
        updated_input.title = "星の向こう側 改訂版".into();
        let updated = update(&conn, &created.id, &updated_input).unwrap();
        assert_eq!(updated.title, "星の向こう側 改訂版");

        assert_eq!(list(&conn).unwrap().len(), 1);

        soft_delete(&conn, &created.id).unwrap();
        assert!(get(&conn, &created.id).unwrap().is_none());
        assert_eq!(list(&conn).unwrap().len(), 0);
    }
}
