use crate::error::AppResult;
use crate::models::{WorldCategory, WorldEntry};
use rusqlite::{params, Connection, OptionalExtension};

/// Standard categories offered on first use of a project's World Bible
/// (spec #25). Users can add their own beyond this list.
pub const BUILTIN_CATEGORY_NAMES: &[&str] = &[
    "国", "都市", "村", "場所", "組織", "企業", "学校", "種族", "宗教", "魔法", "能力",
    "技術", "武器", "アイテム", "文化", "法律", "通貨", "歴史", "事件", "用語",
];

fn row_to_category(row: &rusqlite::Row) -> rusqlite::Result<WorldCategory> {
    Ok(WorldCategory {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        is_builtin: row.get::<_, i64>(3)? != 0,
        order_index: row.get(4)?,
    })
}

pub fn ensure_builtin_categories(conn: &Connection, project_id: &str) -> AppResult<()> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM world_categories WHERE project_id = ?1",
        params![project_id],
        |r| r.get(0),
    )?;
    if count > 0 {
        return Ok(());
    }
    let now = chrono::Utc::now().to_rfc3339();
    for (i, name) in BUILTIN_CATEGORY_NAMES.iter().enumerate() {
        conn.execute(
            "INSERT INTO world_categories (id, project_id, name, is_builtin, order_index, created_at)
             VALUES (?1, ?2, ?3, 1, ?4, ?5)",
            params![uuid::Uuid::new_v4().to_string(), project_id, name, i as i64, now],
        )?;
    }
    Ok(())
}

pub fn list_categories(conn: &Connection, project_id: &str) -> AppResult<Vec<WorldCategory>> {
    ensure_builtin_categories(conn, project_id)?;
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, is_builtin, order_index FROM world_categories
         WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY order_index",
    )?;
    let rows = stmt.query_map(params![project_id], row_to_category)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_category(conn: &Connection, project_id: &str, name: &str) -> AppResult<WorldCategory> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(order_index) + 1, 0) FROM world_categories WHERE project_id = ?1",
        params![project_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO world_categories (id, project_id, name, is_builtin, order_index, created_at)
         VALUES (?1, ?2, ?3, 0, ?4, ?5)",
        params![id, project_id, name, next_order, now],
    )?;
    Ok(WorldCategory { id, project_id: project_id.to_string(), name: name.to_string(), is_builtin: false, order_index: next_order })
}

fn ids_to_json(ids: &[String]) -> String {
    serde_json::to_string(ids).unwrap_or_else(|_| "[]".to_string())
}
fn ids_from_json(s: &str) -> Vec<String> {
    serde_json::from_str(s).unwrap_or_default()
}

fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<WorldEntry> {
    let related_character_ids: String = row.get("related_character_ids")?;
    let related_location_ids: String = row.get("related_location_ids")?;
    let related_entry_ids: String = row.get("related_entry_ids")?;
    Ok(WorldEntry {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        category_id: row.get("category_id")?,
        name: row.get("name")?,
        reading: row.get("reading")?,
        summary: row.get("summary")?,
        detail: row.get("detail")?,
        related_character_ids: ids_from_json(&related_character_ids),
        related_location_ids: ids_from_json(&related_location_ids),
        related_entry_ids: ids_from_json(&related_entry_ids),
        image_path: row.get("image_path")?,
        memo: row.get("memo")?,
    })
}

const ENTRY_COLUMNS: &str = "id, project_id, category_id, name, reading, summary, detail,
    related_character_ids, related_location_ids, related_entry_ids, image_path, memo";

pub fn create_entry(conn: &Connection, project_id: &str, e: &WorldEntry) -> AppResult<WorldEntry> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO world_entries (id, project_id, category_id, name, reading, summary, detail,
            related_character_ids, related_location_ids, related_entry_ids, image_path, memo,
            created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?13)",
        params![
            id, project_id, e.category_id, e.name, e.reading, e.summary, e.detail,
            ids_to_json(&e.related_character_ids), ids_to_json(&e.related_location_ids),
            ids_to_json(&e.related_entry_ids), e.image_path, e.memo, now,
        ],
    )?;
    get_entry(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created world entry".into()))
}

pub fn get_entry(conn: &Connection, id: &str) -> AppResult<Option<WorldEntry>> {
    let sql = format!("SELECT {ENTRY_COLUMNS} FROM world_entries WHERE id = ?1 AND deleted_at IS NULL");
    Ok(conn.query_row(&sql, params![id], row_to_entry).optional()?)
}

pub fn list_entries(conn: &Connection, project_id: &str) -> AppResult<Vec<WorldEntry>> {
    let sql = format!("SELECT {ENTRY_COLUMNS} FROM world_entries WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY name");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], row_to_entry)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn update_entry(conn: &Connection, id: &str, e: &WorldEntry) -> AppResult<WorldEntry> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE world_entries SET category_id=?2, name=?3, reading=?4, summary=?5, detail=?6,
            related_character_ids=?7, related_location_ids=?8, related_entry_ids=?9,
            image_path=?10, memo=?11, updated_at=?12
         WHERE id=?1 AND deleted_at IS NULL",
        params![
            id, e.category_id, e.name, e.reading, e.summary, e.detail,
            ids_to_json(&e.related_character_ids), ids_to_json(&e.related_location_ids),
            ids_to_json(&e.related_entry_ids), e.image_path, e.memo, now,
        ],
    )?;
    get_entry(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("world entry {id}")))
}

pub fn soft_delete_entry(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE world_entries SET deleted_at = ?2 WHERE id = ?1",
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
    fn builtin_categories_seeded_once() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        let categories = list_categories(&conn, &project.id).unwrap();
        assert_eq!(categories.len(), BUILTIN_CATEGORY_NAMES.len());

        // calling again must not duplicate
        let categories_again = list_categories(&conn, &project.id).unwrap();
        assert_eq!(categories_again.len(), BUILTIN_CATEGORY_NAMES.len());
    }

    #[test]
    fn entry_roundtrip_preserves_related_ids() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        let entry = WorldEntry {
            name: "竜脈".into(),
            summary: Some("大地を巡る魔力の流れ".into()),
            related_character_ids: vec!["char-1".into(), "char-2".into()],
            ..Default::default()
        };
        let created = create_entry(&conn, &project.id, &entry).unwrap();
        assert_eq!(created.related_character_ids, vec!["char-1", "char-2"]);

        let fetched = get_entry(&conn, &created.id).unwrap().unwrap();
        assert_eq!(fetched.name, "竜脈");
    }
}
