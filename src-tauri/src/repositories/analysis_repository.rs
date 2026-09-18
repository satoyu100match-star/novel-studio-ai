use crate::error::AppResult;
use crate::models::AiAnalysisReport;
use rusqlite::{params, Connection};

const COLUMNS: &str = "id, project_id, analysis_type, target_summary, context_summary, result, provider, model, created_at";

fn row_to_report(row: &rusqlite::Row) -> rusqlite::Result<AiAnalysisReport> {
    Ok(AiAnalysisReport {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        analysis_type: row.get("analysis_type")?,
        target_summary: row.get("target_summary")?,
        context_summary: row.get("context_summary")?,
        result: row.get("result")?,
        provider: row.get("provider")?,
        model: row.get("model")?,
        created_at: row.get("created_at")?,
    })
}

/// 分析種別を指定しなければ、プロジェクト内の全レポートを新しい順で返す。
pub fn list_by_project(
    conn: &Connection,
    project_id: &str,
    analysis_type: Option<&str>,
) -> AppResult<Vec<AiAnalysisReport>> {
    let rows: Vec<AiAnalysisReport> = if let Some(t) = analysis_type {
        let sql = format!(
            "SELECT {COLUMNS} FROM ai_analysis_reports WHERE project_id = ?1 AND analysis_type = ?2 ORDER BY created_at DESC"
        );
        let mut stmt = conn.prepare(&sql)?;
        let out = stmt.query_map(params![project_id, t], row_to_report)?.collect::<Result<Vec<_>, _>>()?;
        out
    } else {
        let sql = format!("SELECT {COLUMNS} FROM ai_analysis_reports WHERE project_id = ?1 ORDER BY created_at DESC");
        let mut stmt = conn.prepare(&sql)?;
        let out = stmt.query_map(params![project_id], row_to_report)?.collect::<Result<Vec<_>, _>>()?;
        out
    };
    Ok(rows)
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    conn: &Connection,
    project_id: &str,
    analysis_type: &str,
    target_summary: Option<&str>,
    context_summary: Option<&str>,
    result: &str,
    provider: &str,
    model: &str,
) -> AppResult<AiAnalysisReport> {
    if !crate::models::AI_ANALYSIS_TYPES.contains(&analysis_type) {
        return Err(crate::error::AppError::Other(format!("invalid analysis type: {analysis_type}")));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO ai_analysis_reports (id, project_id, analysis_type, target_summary, context_summary, result, provider, model, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, project_id, analysis_type, target_summary, context_summary, result, provider, model, now],
    )?;
    Ok(AiAnalysisReport {
        id,
        project_id: project_id.to_string(),
        analysis_type: analysis_type.to_string(),
        target_summary: target_summary.map(|s| s.to_string()),
        context_summary: context_summary.map(|s| s.to_string()),
        result: result.to_string(),
        provider: provider.to_string(),
        model: model.to_string(),
        created_at: now,
    })
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM ai_analysis_reports WHERE id = ?1", params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::projects_repository;

    #[test]
    fn create_list_filter_and_delete_reports() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        let r1 = create(&conn, &project.id, "contradiction", None, Some("作品全文"), "矛盾は見つかりませんでした。", "anthropic", "claude-x").unwrap();
        create(&conn, &project.id, "style", None, Some("文体統計, 作品全文"), "文体は簡潔です。", "anthropic", "claude-x").unwrap();

        let all = list_by_project(&conn, &project.id, None).unwrap();
        assert_eq!(all.len(), 2);

        let contradiction_only = list_by_project(&conn, &project.id, Some("contradiction")).unwrap();
        assert_eq!(contradiction_only.len(), 1);
        assert_eq!(contradiction_only[0].id, r1.id);

        assert!(create(&conn, &project.id, "not_a_type", None, None, "x", "anthropic", "claude-x").is_err());

        delete(&conn, &r1.id).unwrap();
        assert_eq!(list_by_project(&conn, &project.id, None).unwrap().len(), 1);
    }
}
