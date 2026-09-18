use crate::error::AppResult;
use crate::models::AiUsageSummary;
use rusqlite::{params, Connection};

#[allow(clippy::too_many_arguments)]
pub fn log(
    conn: &Connection,
    project_id: Option<&str>,
    feature: &str,
    provider: &str,
    model: &str,
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
) -> AppResult<()> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO ai_usage_log (id, project_id, feature, provider, model, input_tokens, output_tokens, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, project_id, feature, provider, model, input_tokens, output_tokens, now],
    )?;
    Ok(())
}

/// 本日・今月の集計を返す(仕様#46)。金額換算はプロバイダーごとに単価体系が
/// 異なり不正確な数値を断定的に出さない方針のため行わない(docs/AI.md 6章)。
/// `project_id` を渡すとその作品のみ、`None` ならアプリ全体を集計する。
pub fn summary(conn: &Connection, project_id: Option<&str>) -> AppResult<AiUsageSummary> {
    let now = chrono::Utc::now();
    let today_prefix = now.format("%Y-%m-%d").to_string();
    let month_prefix = now.format("%Y-%m").to_string();

    fn aggregate(
        conn: &Connection,
        project_id: Option<&str>,
        prefix: &str,
    ) -> AppResult<(i64, i64, i64)> {
        let sql = "SELECT COUNT(*), COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0)
                    FROM ai_usage_log
                    WHERE created_at LIKE ?1 || '%' AND (?2 IS NULL OR project_id = ?2)";
        let row = conn.query_row(sql, params![prefix, project_id], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?))
        })?;
        Ok(row)
    }

    let (today_requests, today_input_tokens, today_output_tokens) = aggregate(conn, project_id, &today_prefix)?;
    let (month_requests, month_input_tokens, month_output_tokens) = aggregate(conn, project_id, &month_prefix)?;

    Ok(AiUsageSummary {
        today_requests,
        today_input_tokens,
        today_output_tokens,
        month_requests,
        month_input_tokens,
        month_output_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::projects_repository;

    #[test]
    fn logs_and_summarizes_usage() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        log(&conn, Some(project.id.as_str()), "chat", "anthropic", "claude-x", Some(120), Some(340)).unwrap();
        log(&conn, Some(project.id.as_str()), "rewrite", "anthropic", "claude-x", Some(50), Some(60)).unwrap();
        log(&conn, None, "chat", "openai", "gpt-x", Some(10), Some(20)).unwrap();

        let project_summary = summary(&conn, Some(project.id.as_str())).unwrap();
        assert_eq!(project_summary.today_requests, 2);
        assert_eq!(project_summary.today_input_tokens, 170);
        assert_eq!(project_summary.today_output_tokens, 400);

        let app_summary = summary(&conn, None).unwrap();
        assert_eq!(app_summary.today_requests, 3);
    }
}
