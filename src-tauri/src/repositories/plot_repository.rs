use crate::error::AppResult;
use crate::models::{PlotCard, PlotLane, DEFAULT_PLOT_LANE_NAMES};
use rusqlite::{params, Connection, OptionalExtension};

fn row_to_lane(row: &rusqlite::Row) -> rusqlite::Result<PlotLane> {
    Ok(PlotLane {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        name: row.get("name")?,
        order_index: row.get("order_index")?,
    })
}

/// Seeds the default kanban lanes (構想/執筆予定/執筆中/完了) the first time
/// a project's plot board is opened. Mirrors
/// `world_repository::ensure_builtin_categories`.
pub fn ensure_default_lanes(conn: &Connection, project_id: &str) -> AppResult<()> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM plot_lanes WHERE project_id = ?1 AND deleted_at IS NULL",
        params![project_id],
        |r| r.get(0),
    )?;
    if count > 0 {
        return Ok(());
    }
    let now = chrono::Utc::now().to_rfc3339();
    for (i, name) in DEFAULT_PLOT_LANE_NAMES.iter().enumerate() {
        conn.execute(
            "INSERT INTO plot_lanes (id, project_id, name, order_index, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![uuid::Uuid::new_v4().to_string(), project_id, name, i as i64, now],
        )?;
    }
    Ok(())
}

pub fn list_lanes(conn: &Connection, project_id: &str) -> AppResult<Vec<PlotLane>> {
    ensure_default_lanes(conn, project_id)?;
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, order_index FROM plot_lanes
         WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY order_index",
    )?;
    let rows = stmt.query_map(params![project_id], row_to_lane)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_lane(conn: &Connection, project_id: &str, name: &str) -> AppResult<PlotLane> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(order_index) + 1, 0) FROM plot_lanes WHERE project_id = ?1",
        params![project_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO plot_lanes (id, project_id, name, order_index, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![id, project_id, name, next_order, now],
    )?;
    Ok(PlotLane { id, project_id: project_id.to_string(), name: name.to_string(), order_index: next_order })
}

pub fn rename_lane(conn: &Connection, id: &str, name: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE plot_lanes SET name = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, name, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

pub fn delete_lane(conn: &Connection, id: &str) -> AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute("UPDATE plot_lanes SET deleted_at = ?2 WHERE id = ?1", params![id, &now])?;
    conn.execute("UPDATE plot_cards SET deleted_at = ?2 WHERE lane_id = ?1", params![id, &now])?;
    Ok(())
}

fn row_to_card(row: &rusqlite::Row) -> rusqlite::Result<PlotCard> {
    Ok(PlotCard {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        lane_id: row.get("lane_id")?,
        title: row.get("title")?,
        summary: row.get("summary")?,
        chapter_id: row.get("chapter_id")?,
        color: row.get("color")?,
        order_index: row.get("order_index")?,
    })
}
const CARD_COLUMNS: &str = "id, project_id, lane_id, title, summary, chapter_id, color, order_index";

pub fn list_cards(conn: &Connection, project_id: &str) -> AppResult<Vec<PlotCard>> {
    let sql = format!(
        "SELECT {CARD_COLUMNS} FROM plot_cards WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY order_index"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], row_to_card)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_card(conn: &Connection, project_id: &str, lane_id: &str, title: &str) -> AppResult<PlotCard> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(order_index) + 1, 0) FROM plot_cards WHERE lane_id = ?1",
        params![lane_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO plot_cards (id, project_id, lane_id, title, summary, chapter_id, color, order_index, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, NULL, NULL, NULL, ?5, ?6, ?6)",
        params![id, project_id, lane_id, title, next_order, now],
    )?;
    get_card(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created plot card".into()))
}

pub fn get_card(conn: &Connection, id: &str) -> AppResult<Option<PlotCard>> {
    let sql = format!("SELECT {CARD_COLUMNS} FROM plot_cards WHERE id = ?1 AND deleted_at IS NULL");
    Ok(conn.query_row(&sql, params![id], row_to_card).optional()?)
}

pub fn update_card(conn: &Connection, id: &str, input: &PlotCard) -> AppResult<PlotCard> {
    conn.execute(
        "UPDATE plot_cards SET lane_id=?2, title=?3, summary=?4, chapter_id=?5, color=?6, order_index=?7, updated_at=?8
         WHERE id=?1 AND deleted_at IS NULL",
        params![
            id,
            input.lane_id,
            input.title,
            input.summary,
            input.chapter_id,
            input.color,
            input.order_index,
            chrono::Utc::now().to_rfc3339(),
        ],
    )?;
    get_card(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("plot card {id}")))
}

/// Moves a card to a (possibly different) lane at a given position (仕様の
/// ドラッグ&ドロップ想定 -- board UI passes the target lane and index it
/// just computed after the drop).
pub fn move_card(conn: &Connection, id: &str, lane_id: &str, order_index: i64) -> AppResult<PlotCard> {
    conn.execute(
        "UPDATE plot_cards SET lane_id=?2, order_index=?3, updated_at=?4 WHERE id=?1 AND deleted_at IS NULL",
        params![id, lane_id, order_index, chrono::Utc::now().to_rfc3339()],
    )?;
    get_card(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("plot card {id}")))
}

pub fn delete_card(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE plot_cards SET deleted_at = ?2 WHERE id = ?1",
        params![id, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::projects_repository;

    fn setup() -> (Connection, String) {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();
        (conn, project.id)
    }

    #[test]
    fn seeds_default_lanes_and_creates_cards() {
        let (conn, project_id) = setup();
        let lanes = list_lanes(&conn, &project_id).unwrap();
        assert_eq!(lanes.len(), 4);
        assert_eq!(lanes[0].name, "構想");

        let card = create_card(&conn, &project_id, &lanes[0].id, "冒頭の事件").unwrap();
        assert_eq!(card.lane_id, lanes[0].id);

        let moved = move_card(&conn, &card.id, &lanes[1].id, 0).unwrap();
        assert_eq!(moved.lane_id, lanes[1].id);

        assert_eq!(list_cards(&conn, &project_id).unwrap().len(), 1);
        delete_card(&conn, &card.id).unwrap();
        assert_eq!(list_cards(&conn, &project_id).unwrap().len(), 0);
    }
}
