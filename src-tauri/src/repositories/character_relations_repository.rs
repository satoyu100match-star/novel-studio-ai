use crate::error::AppResult;
use crate::models::CharacterRelation;
use rusqlite::{params, Connection};

fn row_to_relation(row: &rusqlite::Row) -> rusqlite::Result<CharacterRelation> {
    Ok(CharacterRelation {
        id: row.get(0)?,
        project_id: row.get(1)?,
        from_character_id: row.get(2)?,
        to_character_id: row.get(3)?,
        label: row.get(4)?,
        color: row.get(5)?,
        direction: row.get(6)?,
        detail: row.get(7)?,
        start_at: row.get(8)?,
        end_at: row.get(9)?,
    })
}

const SELECT: &str = "SELECT id, project_id, from_character_id, to_character_id, label, color,
    direction, detail, start_at, end_at FROM character_relations";

#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    project_id: &str,
    from_id: &str,
    to_id: &str,
    label: &str,
    direction: &str,
    color: Option<&str>,
    detail: Option<&str>,
) -> AppResult<CharacterRelation> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO character_relations
            (id, project_id, from_character_id, to_character_id, label, color, direction, detail, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![id, project_id, from_id, to_id, label, color, direction, detail, now],
    )?;
    let sql = format!("{SELECT} WHERE id = ?1");
    Ok(conn.query_row(&sql, params![id], row_to_relation)?)
}

pub fn list_by_project(conn: &Connection, project_id: &str) -> AppResult<Vec<CharacterRelation>> {
    let sql = format!("{SELECT} WHERE project_id = ?1 AND deleted_at IS NULL");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], row_to_relation)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn soft_delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE character_relations SET deleted_at = ?2 WHERE id = ?1",
        params![id, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Character, ProjectInput};
    use crate::repositories::{characters_repository, projects_repository};

    #[test]
    fn create_and_list_relations() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();
        let alice = characters_repository::create(&conn, &project.id, &Character { name: "アリス".into(), ..Default::default() }).unwrap();
        let bob = characters_repository::create(&conn, &project.id, &Character { name: "ボブ".into(), ..Default::default() }).unwrap();

        create(&conn, &project.id, &alice.id, &bob.id, "幼馴染", "bidirectional", None, None).unwrap();

        let relations = list_by_project(&conn, &project.id).unwrap();
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].label, "幼馴染");
    }
}
