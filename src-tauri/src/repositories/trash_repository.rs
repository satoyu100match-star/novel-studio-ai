//! ゴミ箱(仕様のTrash/Restore相当)。
//!
//! Soft Delete方針(CLAUDE.md)はPhase1〜4を通じてほぼ全エンティティに
//! 既に適用済み(`deleted_at`列)。このモジュールはその既存の仕組みを
//! 横断的に一覧・復元・完全削除するための薄い層で、新しい削除方式を
//! 追加するものではない。
//!
//! テーブル名はユーザー入力からではなく、`ENTITY_TABLES`という固定の
//! 対応表からしか引かないため、SQLインジェクションの心配なく文字列
//! フォーマットでテーブル名を組み立てられる。

use crate::error::{AppError, AppResult};
use crate::models::TrashItem;
use rusqlite::{params, Connection};

/// (entity_type, table_name)。`restore_item`/`purge_item`はここに無い
/// entity_typeを拒否する。
const ENTITY_TABLES: &[(&str, &str)] = &[
    ("part", "parts"),
    ("chapter", "chapters"),
    ("scene", "scenes"),
    ("character", "characters"),
    ("world_entry", "world_entries"),
    ("location", "locations"),
    ("glossary_entry", "glossary_entries"),
    ("note", "notes"),
    ("plot_card", "plot_cards"),
    ("timeline_event", "timeline_events"),
    ("foreshadowing", "foreshadowings"),
    ("todo", "todos"),
];

fn table_for(entity_type: &str) -> AppResult<&'static str> {
    ENTITY_TABLES
        .iter()
        .find(|(t, _)| *t == entity_type)
        .map(|(_, table)| *table)
        .ok_or_else(|| AppError::Other(format!("未対応のエンティティ種別です: {entity_type}")))
}

pub fn list_trash(conn: &Connection, project_id: &str) -> AppResult<Vec<TrashItem>> {
    let sql = "
        SELECT 'part' as entity_type, id, project_id, title, deleted_at FROM parts WHERE project_id = ?1 AND deleted_at IS NOT NULL
        UNION ALL
        SELECT 'chapter', id, project_id, title, deleted_at FROM chapters WHERE project_id = ?1 AND deleted_at IS NOT NULL
        UNION ALL
        SELECT 'scene', s.id, c.project_id, s.title, s.deleted_at FROM scenes s JOIN chapters c ON s.chapter_id = c.id
            WHERE c.project_id = ?1 AND s.deleted_at IS NOT NULL
        UNION ALL
        SELECT 'character', id, project_id, name, deleted_at FROM characters WHERE project_id = ?1 AND deleted_at IS NOT NULL
        UNION ALL
        SELECT 'world_entry', id, project_id, name, deleted_at FROM world_entries WHERE project_id = ?1 AND deleted_at IS NOT NULL
        UNION ALL
        SELECT 'location', id, project_id, name, deleted_at FROM locations WHERE project_id = ?1 AND deleted_at IS NOT NULL
        UNION ALL
        SELECT 'glossary_entry', id, project_id, term, deleted_at FROM glossary_entries WHERE project_id = ?1 AND deleted_at IS NOT NULL
        UNION ALL
        SELECT 'note', id, project_id, title, deleted_at FROM notes WHERE project_id = ?1 AND deleted_at IS NOT NULL
        UNION ALL
        SELECT 'plot_card', id, project_id, title, deleted_at FROM plot_cards WHERE project_id = ?1 AND deleted_at IS NOT NULL
        UNION ALL
        SELECT 'timeline_event', id, project_id, title, deleted_at FROM timeline_events WHERE project_id = ?1 AND deleted_at IS NOT NULL
        UNION ALL
        SELECT 'foreshadowing', id, project_id, title, deleted_at FROM foreshadowings WHERE project_id = ?1 AND deleted_at IS NOT NULL
        UNION ALL
        SELECT 'todo', id, project_id, title, deleted_at FROM todos WHERE project_id = ?1 AND deleted_at IS NOT NULL
        ORDER BY deleted_at DESC
    ";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(TrashItem {
            entity_type: row.get(0)?,
            id: row.get(1)?,
            project_id: row.get(2)?,
            title: row.get(3)?,
            deleted_at: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// `deleted_at`をNULLに戻す。件の行が既に(何らかの理由で)完全削除
/// されていた場合は0件更新のまま静かに終わる(エラーにはしない --
/// 一覧を再読み込みすれば消えているはずのため)。
pub fn restore_item(conn: &Connection, entity_type: &str, id: &str) -> AppResult<()> {
    let table = table_for(entity_type)?;
    let sql = format!("UPDATE {table} SET deleted_at = NULL WHERE id = ?1 AND deleted_at IS NOT NULL");
    conn.execute(&sql, params![id])?;
    Ok(())
}

/// 完全削除。安全のため、既にSoft Delete済み(`deleted_at IS NOT NULL`)の
/// 行しか対象にしない -- ゴミ箱を経由しない直接の完全削除経路は作らない。
pub fn purge_item(conn: &Connection, entity_type: &str, id: &str) -> AppResult<()> {
    let table = table_for(entity_type)?;
    let sql = format!("DELETE FROM {table} WHERE id = ?1 AND deleted_at IS NOT NULL");
    conn.execute(&sql, params![id])?;
    Ok(())
}

/// ゴミ箱を空にする(プロジェクト内の全Soft Delete済みレコードを完全削除)。
pub fn purge_all(conn: &Connection, project_id: &str) -> AppResult<usize> {
    let items = list_trash(conn, project_id)?;
    let count = items.len();
    for item in items {
        purge_item(conn, &item.entity_type, &item.id)?;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::{chapters_repository, projects_repository, scenes_repository};

    #[test]
    fn lists_restores_and_purges_across_tables() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();
        let chapter = chapters_repository::create(&conn, &project.id, None, "第一章").unwrap();
        let scene = scenes_repository::create(&conn, &chapter.id, "シーン1").unwrap();

        scenes_repository::soft_delete(&conn, &scene.id).unwrap();
        chapters_repository::soft_delete(&conn, &chapter.id).unwrap();

        let trash = list_trash(&conn, &project.id).unwrap();
        assert_eq!(trash.len(), 2);
        let types: Vec<&str> = trash.iter().map(|t| t.entity_type.as_str()).collect();
        assert!(types.contains(&"chapter"));
        assert!(types.contains(&"scene"));

        restore_item(&conn, "chapter", &chapter.id).unwrap();
        let trash_after_restore = list_trash(&conn, &project.id).unwrap();
        assert_eq!(trash_after_restore.len(), 1);
        assert!(chapters_repository::get(&conn, &chapter.id).unwrap().is_some());

        let purged = purge_all(&conn, &project.id).unwrap();
        assert_eq!(purged, 1);
        assert!(list_trash(&conn, &project.id).unwrap().is_empty());

        assert!(restore_item(&conn, "not_a_type", "x").is_err());
    }
}
