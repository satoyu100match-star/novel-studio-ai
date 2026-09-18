//! バージョン履歴(仕様のRevision/Diff相当)。
//!
//! 800msデバウンスで保存するたびにスナップショットを残すとテーブルが
//! 際限なく肥大化する(数万件規模になりうる)ため、自動保存は「最後の
//! 履歴から一定時間(`AUTO_REVISION_MIN_INTERVAL_SECS`)経過していて、
//! かつ内容が変わっている」場合のみ新しい履歴を作る。手動保存
//! (`create_manual`)はこの間隔に関係なく常に作成でき、`prune_old_auto`
//! でも削除されない(明示的にユーザーが残したいと思って作った
//! スナップショットのため)。

use crate::error::AppResult;
use crate::models::Revision;
use rusqlite::{params, Connection, OptionalExtension};

/// 自動スナップショットの最短間隔。編集のたびに作らないための間引き。
pub const AUTO_REVISION_MIN_INTERVAL_SECS: i64 = 300;
/// 自動スナップショットを1つの章/シーンにつき最大何件残すか(手動分は除く)。
const MAX_AUTO_REVISIONS_PER_OWNER: usize = 50;

const COLUMNS: &str = "id, owner_type, owner_id, body, char_count, trigger, label, created_at";

fn row_to_revision(row: &rusqlite::Row) -> rusqlite::Result<Revision> {
    Ok(Revision {
        id: row.get("id")?,
        owner_type: row.get("owner_type")?,
        owner_id: row.get("owner_id")?,
        body: row.get("body")?,
        char_count: row.get("char_count")?,
        trigger: row.get("trigger")?,
        label: row.get("label")?,
        created_at: row.get("created_at")?,
    })
}

/// 履歴一覧(本文込み)。新しい順。一覧表示では本文プレビューや差分表示
/// (`DiffView`の再利用)に使うため、あえて本文を省略しない
/// (件数上限があるため肥大化の心配は小さい)。
pub fn list_by_owner(conn: &Connection, owner_type: &str, owner_id: &str) -> AppResult<Vec<Revision>> {
    let sql = format!("SELECT {COLUMNS} FROM revisions WHERE owner_type = ?1 AND owner_id = ?2 ORDER BY created_at DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![owner_type, owner_id], row_to_revision)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<Revision>> {
    let sql = format!("SELECT {COLUMNS} FROM revisions WHERE id = ?1");
    Ok(conn.query_row(&sql, params![id], row_to_revision).optional()?)
}

fn latest(conn: &Connection, owner_type: &str, owner_id: &str) -> AppResult<Option<Revision>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM revisions WHERE owner_type = ?1 AND owner_id = ?2 ORDER BY created_at DESC LIMIT 1"
    );
    Ok(conn.query_row(&sql, params![owner_type, owner_id], row_to_revision).optional()?)
}

fn insert(conn: &Connection, owner_type: &str, owner_id: &str, body: &str, trigger: &str, label: Option<&str>) -> AppResult<Revision> {
    let id = uuid::Uuid::new_v4().to_string();
    let char_count = body.chars().count() as i64;
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO revisions (id, owner_type, owner_id, body, char_count, trigger, label, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, owner_type, owner_id, body, char_count, trigger, label, now],
    )?;
    Ok(Revision {
        id,
        owner_type: owner_type.to_string(),
        owner_id: owner_id.to_string(),
        body: body.to_string(),
        char_count,
        trigger: trigger.to_string(),
        label: label.map(|s| s.to_string()),
        created_at: now,
    })
}

/// ユーザーが明示的に「現在の内容をスナップショット保存」した履歴。
pub fn create_manual(conn: &Connection, owner_type: &str, owner_id: &str, body: &str, label: Option<&str>) -> AppResult<Revision> {
    insert(conn, owner_type, owner_id, body, "manual", label)
}

/// 保存(オートセーブ含む)のたびに呼ばれる。最後の履歴から
/// `AUTO_REVISION_MIN_INTERVAL_SECS`以上経過し、かつ内容が変わっている
/// 場合のみ新しい履歴を作る。作った場合は古い自動履歴を間引く。
pub fn maybe_create_auto(conn: &Connection, owner_type: &str, owner_id: &str, body: &str) -> AppResult<Option<Revision>> {
    let prev = latest(conn, owner_type, owner_id)?;
    let should_create = match &prev {
        None => !body.is_empty(),
        Some(p) => {
            if p.body == body {
                false
            } else {
                let prev_time = chrono::DateTime::parse_from_rfc3339(&p.created_at)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());
                let elapsed = chrono::Utc::now().signed_duration_since(prev_time).num_seconds();
                elapsed >= AUTO_REVISION_MIN_INTERVAL_SECS
            }
        }
    };
    if !should_create {
        return Ok(None);
    }
    let revision = insert(conn, owner_type, owner_id, body, "auto", None)?;
    prune_old_auto(conn, owner_type, owner_id)?;
    Ok(Some(revision))
}

/// 自動履歴のみを対象に、新しい方から`MAX_AUTO_REVISIONS_PER_OWNER`件を
/// 残して古いものを削除する。手動履歴は対象外(ユーザーが残した
/// スナップショットを勝手に消さない)。
fn prune_old_auto(conn: &Connection, owner_type: &str, owner_id: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM revisions WHERE owner_type = ?1 AND owner_id = ?2 AND trigger = 'auto' AND id NOT IN (
            SELECT id FROM revisions WHERE owner_type = ?1 AND owner_id = ?2 AND trigger = 'auto'
            ORDER BY created_at DESC LIMIT ?3
        )",
        params![owner_type, owner_id, MAX_AUTO_REVISIONS_PER_OWNER as i64],
    )?;
    Ok(())
}

/// 指定した履歴の内容を現在の本文として復元する。復元前の状態を
/// 失わないよう、復元直前の本文を(最新履歴と同一でなければ)手動履歴
/// として先に保存してから上書きする。
pub fn restore(conn: &Connection, id: &str) -> AppResult<crate::models::Document> {
    let target = get(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("revision {id}")))?;

    if let Some(current) = super::documents_repository::get(conn, &target.owner_type, &target.owner_id)? {
        let already_saved = latest(conn, &target.owner_type, &target.owner_id)?
            .map(|r| r.body == current.body)
            .unwrap_or(false);
        if !already_saved && !current.body.is_empty() {
            insert(conn, &target.owner_type, &target.owner_id, &current.body, "manual", Some("復元前のバックアップ"))?;
        }
    }

    super::documents_repository::save_body(conn, &target.owner_type, &target.owner_id, &target.body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::{chapters_repository, documents_repository, projects_repository};

    fn setup_chapter() -> (Connection, String) {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();
        let chapter = chapters_repository::create(&conn, &project.id, None, "第一章").unwrap();
        (conn, chapter.id)
    }

    #[test]
    fn manual_revision_always_created_regardless_of_interval() {
        let (conn, chapter_id) = setup_chapter();
        create_manual(&conn, "chapter", &chapter_id, "本文1", Some("初回")).unwrap();
        create_manual(&conn, "chapter", &chapter_id, "本文2", None).unwrap();
        let list = list_by_owner(&conn, "chapter", &chapter_id).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].body, "本文2"); // 新しい順
    }

    #[test]
    fn auto_revision_skips_when_unchanged_or_too_soon() {
        let (conn, chapter_id) = setup_chapter();
        let first = maybe_create_auto(&conn, "chapter", &chapter_id, "本文A").unwrap();
        assert!(first.is_some());

        // 直後にもう一度同じ内容で呼んでも増えない(未変更)
        let second = maybe_create_auto(&conn, "chapter", &chapter_id, "本文A").unwrap();
        assert!(second.is_none());

        // 内容が変わっても、間隔が短すぎればまだ作られない
        let third = maybe_create_auto(&conn, "chapter", &chapter_id, "本文B").unwrap();
        assert!(third.is_none());

        assert_eq!(list_by_owner(&conn, "chapter", &chapter_id).unwrap().len(), 1);
    }

    #[test]
    fn restore_snapshots_current_state_before_overwriting() {
        let (conn, chapter_id) = setup_chapter();
        documents_repository::save_body(&conn, "chapter", &chapter_id, "古い本文").unwrap();
        let old_revision = create_manual(&conn, "chapter", &chapter_id, "古い本文", Some("v1")).unwrap();
        documents_repository::save_body(&conn, "chapter", &chapter_id, "新しい本文").unwrap();

        let restored = restore(&conn, &old_revision.id).unwrap();
        assert_eq!(restored.body, "古い本文");

        // 復元前の「新しい本文」がバックアップとして残っている
        let history = list_by_owner(&conn, "chapter", &chapter_id).unwrap();
        assert!(history.iter().any(|r| r.body == "新しい本文" && r.label.as_deref() == Some("復元前のバックアップ")));
    }
}
