use crate::error::AppResult;
use crate::models::Location;
use rusqlite::{params, Connection, OptionalExtension};

fn ids_to_json(ids: &[String]) -> String {
    serde_json::to_string(ids).unwrap_or_else(|_| "[]".to_string())
}
fn ids_from_json(s: &str) -> Vec<String> {
    serde_json::from_str(s).unwrap_or_default()
}

const COLUMNS: &str = "id, project_id, parent_location_id, name, description, coordinates,
    related_character_ids, image_path, memo";

fn row_to_location(row: &rusqlite::Row) -> rusqlite::Result<Location> {
    let related: String = row.get("related_character_ids")?;
    Ok(Location {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        parent_location_id: row.get("parent_location_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        coordinates: row.get("coordinates")?,
        related_character_ids: ids_from_json(&related),
        image_path: row.get("image_path")?,
        memo: row.get("memo")?,
    })
}

pub fn create(conn: &Connection, project_id: &str, l: &Location) -> AppResult<Location> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO locations (id, project_id, parent_location_id, name, description,
            coordinates, related_character_ids, image_path, memo, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?10)",
        params![
            id, project_id, l.parent_location_id, l.name, l.description, l.coordinates,
            ids_to_json(&l.related_character_ids), l.image_path, l.memo, now,
        ],
    )?;
    get(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created location".into()))
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<Location>> {
    let sql = format!("SELECT {COLUMNS} FROM locations WHERE id = ?1 AND deleted_at IS NULL");
    Ok(conn.query_row(&sql, params![id], row_to_location).optional()?)
}

pub fn list_by_project(conn: &Connection, project_id: &str) -> AppResult<Vec<Location>> {
    let sql = format!("SELECT {COLUMNS} FROM locations WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY name");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], row_to_location)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn update(conn: &Connection, id: &str, l: &Location) -> AppResult<Location> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE locations SET parent_location_id=?2, name=?3, description=?4, coordinates=?5,
            related_character_ids=?6, image_path=?7, memo=?8, updated_at=?9
         WHERE id=?1 AND deleted_at IS NULL",
        params![
            id, l.parent_location_id, l.name, l.description, l.coordinates,
            ids_to_json(&l.related_character_ids), l.image_path, l.memo, now,
        ],
    )?;
    get(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("location {id}")))
}

pub fn soft_delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE locations SET deleted_at = ?2 WHERE id = ?1",
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
    fn create_and_list_with_hierarchy() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        let kingdom = create(&conn, &project.id, &Location { name: "王国".into(), ..Default::default() }).unwrap();
        let capital = create(
            &conn,
            &project.id,
            &Location { name: "王都".into(), parent_location_id: Some(kingdom.id.clone()), ..Default::default() },
        )
        .unwrap();

        assert_eq!(capital.parent_location_id, Some(kingdom.id));
        assert_eq!(list_by_project(&conn, &project.id).unwrap().len(), 2);
    }
}
