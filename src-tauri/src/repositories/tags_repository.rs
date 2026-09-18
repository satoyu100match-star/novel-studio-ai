use crate::error::AppResult;
use crate::models::Tag;
use rusqlite::{params, Connection, OptionalExtension};

fn row_to_tag(row: &rusqlite::Row) -> rusqlite::Result<Tag> {
    Ok(Tag { id: row.get(0)?, project_id: row.get(1)?, name: row.get(2)?, color: row.get(3)? })
}

pub fn get_or_create(conn: &Connection, project_id: &str, name: &str) -> AppResult<Tag> {
    if let Some(existing) = conn
        .query_row(
            "SELECT id, project_id, name, color FROM tags WHERE project_id = ?1 AND name = ?2",
            params![project_id, name],
            row_to_tag,
        )
        .optional()?
    {
        return Ok(existing);
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO tags (id, project_id, name, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, project_id, name, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(Tag { id, project_id: project_id.to_string(), name: name.to_string(), color: None })
}

pub fn list_by_project(conn: &Connection, project_id: &str) -> AppResult<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT id, project_id, name, color FROM tags WHERE project_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map(params![project_id], row_to_tag)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Replaces the full tag set attached to one entity (simplest correct
/// approach at Phase 3 data volumes: delete then re-insert, inside the
/// caller's connection so it's atomic with whatever else the command does).
pub fn set_entity_tags(conn: &Connection, project_id: &str, entity_type: &str, entity_id: &str, tag_names: &[String]) -> AppResult<()> {
    conn.execute(
        "DELETE FROM entity_tags WHERE entity_type = ?1 AND entity_id = ?2",
        params![entity_type, entity_id],
    )?;
    for name in tag_names {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            continue;
        }
        let tag = get_or_create(conn, project_id, trimmed)?;
        conn.execute(
            "INSERT OR IGNORE INTO entity_tags (id, tag_id, entity_type, entity_id) VALUES (?1, ?2, ?3, ?4)",
            params![uuid::Uuid::new_v4().to_string(), tag.id, entity_type, entity_id],
        )?;
    }
    Ok(())
}

pub fn get_entity_tags(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT t.name FROM entity_tags et JOIN tags t ON t.id = et.tag_id
         WHERE et.entity_type = ?1 AND et.entity_id = ?2 ORDER BY t.name",
    )?;
    let rows = stmt.query_map(params![entity_type, entity_id], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::projects_repository;

    #[test]
    fn set_entity_tags_creates_and_replaces() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        set_entity_tags(&conn, &project.id, "character", "char-1", &["重要".into(), "恋愛".into()]).unwrap();
        assert_eq!(get_entity_tags(&conn, "character", "char-1").unwrap(), vec!["恋愛", "重要"]);

        set_entity_tags(&conn, &project.id, "character", "char-1", &["重要".into()]).unwrap();
        assert_eq!(get_entity_tags(&conn, "character", "char-1").unwrap(), vec!["重要"]);

        assert_eq!(list_by_project(&conn, &project.id).unwrap().len(), 2); // both tags remain defined project-wide
    }
}
