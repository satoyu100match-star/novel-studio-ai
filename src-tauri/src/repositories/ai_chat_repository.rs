use crate::error::AppResult;
use crate::models::AiChatMessage;
use rusqlite::{params, Connection};

fn row_to_message(row: &rusqlite::Row) -> rusqlite::Result<AiChatMessage> {
    Ok(AiChatMessage {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        role: row.get("role")?,
        content: row.get("content")?,
        context_summary: row.get("context_summary")?,
        provider: row.get("provider")?,
        model: row.get("model")?,
        created_at: row.get("created_at")?,
    })
}
const COLUMNS: &str = "id, project_id, role, content, context_summary, provider, model, created_at";

pub fn list_by_project(conn: &Connection, project_id: &str) -> AppResult<Vec<AiChatMessage>> {
    let sql = format!("SELECT {COLUMNS} FROM ai_chat_messages WHERE project_id = ?1 ORDER BY created_at");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], row_to_message)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[allow(clippy::too_many_arguments)]
pub fn append(
    conn: &Connection,
    project_id: &str,
    role: &str,
    content: &str,
    context_summary: Option<&str>,
    provider: Option<&str>,
    model: Option<&str>,
) -> AppResult<AiChatMessage> {
    if role != "user" && role != "assistant" {
        return Err(crate::error::AppError::Other(format!("invalid ai chat role: {role}")));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO ai_chat_messages (id, project_id, role, content, context_summary, provider, model, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, project_id, role, content, context_summary, provider, model, now],
    )?;
    Ok(AiChatMessage {
        id,
        project_id: project_id.to_string(),
        role: role.to_string(),
        content: content.to_string(),
        context_summary: context_summary.map(|s| s.to_string()),
        provider: provider.map(|s| s.to_string()),
        model: model.map(|s| s.to_string()),
        created_at: now,
    })
}

pub fn clear(conn: &Connection, project_id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM ai_chat_messages WHERE project_id = ?1", params![project_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::projects_repository;

    #[test]
    fn append_and_list_messages_in_order() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        append(&conn, &project.id, "user", "3章の伏線を整理して", None, None, None).unwrap();
        append(&conn, &project.id, "assistant", "了解しました。", Some("作品概要, 第3章"), Some("anthropic"), Some("claude-x")).unwrap();

        let msgs = list_by_project(&conn, &project.id).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, "user");
        assert_eq!(msgs[1].provider.as_deref(), Some("anthropic"));

        assert!(append(&conn, &project.id, "system", "x", None, None, None).is_err());

        clear(&conn, &project.id).unwrap();
        assert_eq!(list_by_project(&conn, &project.id).unwrap().len(), 0);
    }
}
