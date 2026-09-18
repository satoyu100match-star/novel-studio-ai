//! Phase 1 search: plain `LIKE` across chapter/scene titles and manuscript
//! bodies within one project. Good enough for a first pass; a proper
//! full-text index (SQLite FTS5) and the "高度検索" filters (spec #65) are
//! deferred to a later phase once there's enough real content to justify
//! it (see docs/ROADMAP.md -- don't add complexity ahead of measured need).

use crate::error::AppResult;
use crate::models::SearchHit;
use rusqlite::{params, Connection};

fn snippet(body: &str, query: &str) -> String {
    let lower_body = body.to_lowercase();
    let lower_query = query.to_lowercase();
    match lower_body.find(&lower_query) {
        Some(byte_idx) => {
            let start = body[..byte_idx].char_indices().rev().nth(20).map(|(i, _)| i).unwrap_or(0);
            let end_from = byte_idx + query.len();
            let end = body[end_from..]
                .char_indices()
                .nth(20)
                .map(|(i, _)| end_from + i)
                .unwrap_or(body.len());
            format!("...{}...", &body[start..end])
        }
        None => body.chars().take(40).collect(),
    }
}

pub fn search_project(conn: &Connection, project_id: &str, query: &str) -> AppResult<Vec<SearchHit>> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    let like = format!("%{query}%");
    let mut hits = Vec::new();

    let mut chapter_stmt = conn.prepare(
        "SELECT c.id, c.title, COALESCE(d.body, '') FROM chapters c
         LEFT JOIN documents d ON d.owner_type = 'chapter' AND d.owner_id = c.id
         WHERE c.project_id = ?1 AND c.deleted_at IS NULL
           AND (c.title LIKE ?2 OR d.body LIKE ?2)",
    )?;
    let chapter_rows = chapter_stmt.query_map(params![project_id, like], |row| {
        let title: String = row.get(1)?;
        let body: String = row.get(2)?;
        Ok((row.get::<_, String>(0)?, title, body))
    })?;
    for row in chapter_rows {
        let (id, title, body) = row?;
        hits.push(SearchHit { owner_type: "chapter".into(), owner_id: id, title, snippet: snippet(&body, query) });
    }

    let mut scene_stmt = conn.prepare(
        "SELECT s.id, s.title, COALESCE(d.body, '') FROM scenes s
         JOIN chapters c ON s.chapter_id = c.id AND c.deleted_at IS NULL
         LEFT JOIN documents d ON d.owner_type = 'scene' AND d.owner_id = s.id
         WHERE c.project_id = ?1 AND s.deleted_at IS NULL
           AND (s.title LIKE ?2 OR d.body LIKE ?2)",
    )?;
    let scene_rows = scene_stmt.query_map(params![project_id, like], |row| {
        let title: String = row.get(1)?;
        let body: String = row.get(2)?;
        Ok((row.get::<_, String>(0)?, title, body))
    })?;
    for row in scene_rows {
        let (id, title, body) = row?;
        hits.push(SearchHit { owner_type: "scene".into(), owner_id: id, title, snippet: snippet(&body, query) });
    }

    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::{chapters_repository, documents_repository, projects_repository};

    #[test]
    fn finds_matches_in_title_and_body() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();
        let chapter = chapters_repository::create(&conn, &project.id, None, "王都の陰謀").unwrap();
        documents_repository::save_body(&conn, "chapter", &chapter.id, "アリスは王都で懐中時計を見つけた。").unwrap();

        let by_title = search_project(&conn, &project.id, "陰謀").unwrap();
        assert_eq!(by_title.len(), 1);

        let by_body = search_project(&conn, &project.id, "懐中時計").unwrap();
        assert_eq!(by_body.len(), 1);
        assert!(by_body[0].snippet.contains("懐中時計"));

        let none = search_project(&conn, &project.id, "存在しない語句").unwrap();
        assert_eq!(none.len(), 0);
    }
}
