//! Context Builder (仕様#41, docs/AI.md 2章)。
//!
//! AIに毎回作品全文を丸ごと渡さないよう、送信候補となる情報を「ブロック」
//! 単位で組み立てる。実際に送るかどうかはフロント側のContext Inspector
//! (仕様#42)でユーザーが確認・取捨選択してから初めて送信される -- この
//! モジュールは候補を集めるだけで、AI Gatewayへの送信は一切行わない。

use crate::error::AppResult;
use crate::models::ContextBlock;
use crate::repositories::{
    characters_repository, chapters_repository, documents_repository, foreshadowing_repository,
    projects_repository, scenes_repository, timeline_repository, world_repository,
};
use rusqlite::Connection;

const PRECEDING_TEXT_CHARS: usize = 1200;

fn char_count(s: &str) -> i64 {
    s.chars().count() as i64
}

/// 章/シーンいずれかの選択に応じて候補ブロック群を組み立てる。
/// `chapter_id`/`scene_id` はどちらか一方、または両方省略可(その場合は
/// 作品全体の設定情報のみを候補として返す)。
pub fn build_context(
    conn: &Connection,
    project_id: &str,
    chapter_id: Option<&str>,
    scene_id: Option<&str>,
    user_question: Option<&str>,
) -> AppResult<Vec<ContextBlock>> {
    let mut blocks = Vec::new();

    if let Some(project) = projects_repository::get(conn, project_id)? {
        let mut lines = Vec::new();
        if let Some(v) = &project.synopsis {
            if !v.is_empty() {
                lines.push(format!("あらすじ: {v}"));
            }
        }
        if let Some(v) = &project.theme {
            if !v.is_empty() {
                lines.push(format!("テーマ: {v}"));
            }
        }
        if let Some(v) = &project.concept {
            if !v.is_empty() {
                lines.push(format!("コンセプト: {v}"));
            }
        }
        if let Some(v) = &project.pov_policy {
            if !v.is_empty() {
                lines.push(format!("視点方針: {v}"));
            }
        }
        if !lines.is_empty() {
            let content = lines.join("\n");
            blocks.push(ContextBlock {
                label: "作品概要".into(),
                char_count: char_count(&content),
                content,
                included_by_default: true,
            });
        }
    }

    let mut resolved_chapter_id = chapter_id.map(|s| s.to_string());

    if let Some(cid) = chapter_id {
        if let Some(chapter) = chapters_repository::get(conn, cid)? {
            let mut lines = vec![format!("タイトル: {}", chapter.title)];
            if let Some(v) = &chapter.synopsis {
                if !v.is_empty() {
                    lines.push(format!("概要: {v}"));
                }
            }
            if let Some(v) = &chapter.memo {
                if !v.is_empty() {
                    lines.push(format!("メモ: {v}"));
                }
            }
            let content = lines.join("\n");
            blocks.push(ContextBlock {
                label: "現在の章".into(),
                char_count: char_count(&content),
                content,
                included_by_default: true,
            });
        }
    }

    if let Some(sid) = scene_id {
        if let Some(scene) = scenes_repository::get(conn, sid)? {
            resolved_chapter_id.get_or_insert(scene.chapter_id.clone());
            let mut lines = vec![format!("タイトル: {}", scene.title)];
            if let Some(v) = &scene.summary {
                if !v.is_empty() {
                    lines.push(format!("概要: {v}"));
                }
            }
            if let Some(v) = &scene.purpose {
                if !v.is_empty() {
                    lines.push(format!("目的: {v}"));
                }
            }
            if let Some(v) = &scene.conflict {
                if !v.is_empty() {
                    lines.push(format!("葛藤: {v}"));
                }
            }
            let content = lines.join("\n");
            blocks.push(ContextBlock {
                label: "現在のシーン".into(),
                char_count: char_count(&content),
                content,
                included_by_default: true,
            });
        }
    }

    // 直前本文: シーンがあればシーンの本文、なければ章の本文の末尾。
    let doc_owner: Option<(&str, String)> = if let Some(sid) = scene_id {
        Some(("scene", sid.to_string()))
    } else {
        resolved_chapter_id.clone().map(|cid| ("chapter", cid))
    };
    if let Some((owner_type, owner_id)) = doc_owner {
        if let Some(doc) = documents_repository::get(conn, owner_type, &owner_id)? {
            if !doc.body.is_empty() {
                let chars: Vec<char> = doc.body.chars().collect();
                let start = chars.len().saturating_sub(PRECEDING_TEXT_CHARS);
                let excerpt: String = chars[start..].iter().collect();
                blocks.push(ContextBlock {
                    label: "直前本文".into(),
                    char_count: char_count(&excerpt),
                    content: excerpt,
                    included_by_default: true,
                });
            }
        }
    }

    let characters = characters_repository::list_by_project(conn, project_id)?;
    if !characters.is_empty() {
        let content = characters
            .iter()
            .map(|c| {
                let role = c.role.clone().unwrap_or_default();
                let personality = c.personality.clone().unwrap_or_default();
                format!("- {}({}): {}", c.name, role, personality)
            })
            .collect::<Vec<_>>()
            .join("\n");
        blocks.push(ContextBlock {
            label: "登場人物".into(),
            char_count: char_count(&content),
            content,
            included_by_default: false,
        });
    }

    let world_entries = world_repository::list_entries(conn, project_id)?;
    if !world_entries.is_empty() {
        let content = world_entries
            .iter()
            .map(|e| format!("- {}: {}", e.name, e.summary.clone().unwrap_or_default()))
            .collect::<Vec<_>>()
            .join("\n");
        blocks.push(ContextBlock {
            label: "世界観設定".into(),
            char_count: char_count(&content),
            content,
            included_by_default: false,
        });
    }

    let foreshadowings = foreshadowing_repository::list_by_project(conn, project_id)?
        .into_iter()
        .filter(|f| f.status != "resolved" && f.status != "discarded")
        .collect::<Vec<_>>();
    if !foreshadowings.is_empty() {
        let content = foreshadowings
            .iter()
            .map(|f| format!("- {}({}): {}", f.title, f.status, f.detail.clone().unwrap_or_default()))
            .collect::<Vec<_>>()
            .join("\n");
        blocks.push(ContextBlock {
            label: "未回収の伏線".into(),
            char_count: char_count(&content),
            content,
            included_by_default: false,
        });
    }

    if let Some(q) = user_question {
        if !q.is_empty() {
            blocks.push(ContextBlock {
                label: "ユーザーの質問".into(),
                char_count: char_count(q),
                content: q.to_string(),
                included_by_default: true,
            });
        }
    }

    Ok(blocks)
}

/// 作品全体の本文を、章/シーンの見出し付きで連結する(Phase 7の高度AI分析用)。
/// 章に直接本文がある場合とシーン単位で本文がある場合の両方に対応する
/// (`documents_repository::total_char_count_for_project`と同様、両方の
/// 存在を許容する設計に合わせている)。
///
/// 長編になるほど文字数が膨らみ、AIプロバイダーの上限を超える恐れがある
/// (既知の制約。docs/AI.md参照)。現状は全文をそのまま候補ブロックとして
/// 提示し、送るかどうかの最終判断はContext Inspectorでのユーザー確認に
/// 委ねている。
pub fn full_manuscript_text(conn: &Connection, project_id: &str) -> AppResult<String> {
    let mut out = String::new();
    for chapter in chapters_repository::list_by_project(conn, project_id)? {
        let mut chapter_had_body = false;
        let mut chapter_section = format!("### {}\n", chapter.title);
        if let Some(doc) = documents_repository::get(conn, "chapter", &chapter.id)? {
            if !doc.body.is_empty() {
                chapter_section.push_str(&doc.body);
                chapter_section.push('\n');
                chapter_had_body = true;
            }
        }
        for scene in scenes_repository::list_by_chapter(conn, &chapter.id)? {
            if let Some(doc) = documents_repository::get(conn, "scene", &scene.id)? {
                if !doc.body.is_empty() {
                    chapter_section.push_str(&format!("#### {}\n", scene.title));
                    chapter_section.push_str(&doc.body);
                    chapter_section.push('\n');
                    chapter_had_body = true;
                }
            }
        }
        if chapter_had_body {
            out.push_str(&chapter_section);
        }
    }
    Ok(out)
}

/// Phase 7: 高度AI分析用のコンテキスト候補を組み立てる。通常のチャット用
/// `build_context`とは異なり、分析対象は「今どこを開いているか」ではなく
/// 「分析種別ごとに必要な作品全体の情報」なので、専用の組み立てにしている。
pub fn build_analysis_context(
    conn: &Connection,
    project_id: &str,
    analysis_type: &str,
    character_id: Option<&str>,
) -> AppResult<Vec<ContextBlock>> {
    let mut blocks = Vec::new();

    if let Some(project) = projects_repository::get(conn, project_id)? {
        if let Some(v) = &project.synopsis {
            if !v.is_empty() {
                blocks.push(ContextBlock {
                    label: "作品概要".into(),
                    char_count: char_count(v),
                    content: v.clone(),
                    included_by_default: true,
                });
            }
        }
    }

    if analysis_type == "character_tone" {
        if let Some(cid) = character_id {
            if let Some(c) = characters_repository::get(conn, cid)? {
                let mut lines = vec![format!("名前: {}", c.name)];
                if let Some(v) = &c.personality {
                    if !v.is_empty() {
                        lines.push(format!("性格: {v}"));
                    }
                }
                if let Some(v) = &c.first_person {
                    if !v.is_empty() {
                        lines.push(format!("一人称: {v}"));
                    }
                }
                if let Some(v) = &c.second_person {
                    if !v.is_empty() {
                        lines.push(format!("二人称: {v}"));
                    }
                }
                if let Some(v) = &c.speech_suffix {
                    if !v.is_empty() {
                        lines.push(format!("語尾: {v}"));
                    }
                }
                if let Some(v) = &c.catchphrase {
                    if !v.is_empty() {
                        lines.push(format!("口癖: {v}"));
                    }
                }
                if let Some(v) = &c.honorific_level {
                    if !v.is_empty() {
                        lines.push(format!("敬語レベル: {v}"));
                    }
                }
                let content = lines.join("\n");
                blocks.push(ContextBlock {
                    label: "対象キャラクター設定".into(),
                    char_count: char_count(&content),
                    content,
                    included_by_default: true,
                });
            }
        }
    }

    if matches!(analysis_type, "contradiction" | "character_tone" | "setting") {
        let characters = characters_repository::list_by_project(conn, project_id)?;
        if !characters.is_empty() {
            let content = characters
                .iter()
                .map(|c| format!("- {}({}): {}", c.name, c.role.clone().unwrap_or_default(), c.personality.clone().unwrap_or_default()))
                .collect::<Vec<_>>()
                .join("\n");
            blocks.push(ContextBlock {
                label: "登場人物".into(),
                char_count: char_count(&content),
                content,
                included_by_default: analysis_type != "character_tone",
            });
        }
    }

    if matches!(analysis_type, "contradiction" | "setting") {
        let world_entries = world_repository::list_entries(conn, project_id)?;
        if !world_entries.is_empty() {
            let content = world_entries
                .iter()
                .map(|e| format!("- {}: {} / {}", e.name, e.summary.clone().unwrap_or_default(), e.detail.clone().unwrap_or_default()))
                .collect::<Vec<_>>()
                .join("\n");
            blocks.push(ContextBlock {
                label: "世界観設定".into(),
                char_count: char_count(&content),
                content,
                included_by_default: true,
            });
        }
    }

    if matches!(analysis_type, "contradiction" | "timeline") {
        let events = timeline_repository::list_by_project(conn, project_id)?;
        if !events.is_empty() {
            let content = events
                .iter()
                .map(|e| format!("- [{}] {}: {}", e.event_date.clone().unwrap_or_default(), e.title, e.description.clone().unwrap_or_default()))
                .collect::<Vec<_>>()
                .join("\n");
            blocks.push(ContextBlock {
                label: "時系列".into(),
                char_count: char_count(&content),
                content,
                included_by_default: true,
            });
        }
    }

    if matches!(analysis_type, "contradiction" | "foreshadowing") {
        let foreshadowings = foreshadowing_repository::list_by_project(conn, project_id)?;
        if !foreshadowings.is_empty() {
            let content = foreshadowings
                .iter()
                .map(|f| format!("- {}({}): {}", f.title, f.status, f.detail.clone().unwrap_or_default()))
                .collect::<Vec<_>>()
                .join("\n");
            blocks.push(ContextBlock {
                label: "伏線一覧".into(),
                char_count: char_count(&content),
                content,
                included_by_default: true,
            });
        }
    }

    if analysis_type == "style" {
        let text = full_manuscript_text(conn, project_id)?;
        let stats = crate::readability::compute(&text);
        let content = crate::readability::summarize_for_ai(&stats);
        blocks.push(ContextBlock {
            label: "文体統計".into(),
            char_count: char_count(&content),
            content,
            included_by_default: true,
        });
    }

    let manuscript = full_manuscript_text(conn, project_id)?;
    if !manuscript.is_empty() {
        blocks.push(ContextBlock {
            label: "作品全文".into(),
            char_count: char_count(&manuscript),
            content: manuscript,
            included_by_default: true,
        });
    }

    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::projects_repository;

    #[test]
    fn builds_blocks_from_project_and_chapter() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(
            &conn,
            &ProjectInput { title: "P".into(), synopsis: Some("主人公が謎を追う".into()), ..Default::default() },
            false,
        )
        .unwrap();
        let chapter = chapters_repository::create(&conn, &project.id, None, "第一章").unwrap();
        documents_repository::save_body(&conn, "chapter", &chapter.id, "その夜、時計が止まった。").unwrap();

        let blocks = build_context(&conn, &project.id, Some(&chapter.id), None, Some("この後どう展開すべき?")).unwrap();
        let labels: Vec<&str> = blocks.iter().map(|b| b.label.as_str()).collect();
        assert!(labels.contains(&"作品概要"));
        assert!(labels.contains(&"現在の章"));
        assert!(labels.contains(&"直前本文"));
        assert!(labels.contains(&"ユーザーの質問"));
    }

    #[test]
    fn full_manuscript_text_concatenates_chapter_and_scene_bodies() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();
        let chapter = chapters_repository::create(&conn, &project.id, None, "第一章").unwrap();
        documents_repository::save_body(&conn, "chapter", &chapter.id, "章直下の本文。").unwrap();
        let scene = crate::repositories::scenes_repository::create(&conn, &chapter.id, "シーン1").unwrap();
        documents_repository::save_body(&conn, "scene", &scene.id, "シーンの本文。").unwrap();

        let text = full_manuscript_text(&conn, &project.id).unwrap();
        assert!(text.contains("第一章"));
        assert!(text.contains("章直下の本文。"));
        assert!(text.contains("シーン1"));
        assert!(text.contains("シーンの本文。"));
    }

    #[test]
    fn build_analysis_context_includes_manuscript_and_type_specific_blocks() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();
        let chapter = chapters_repository::create(&conn, &project.id, None, "第一章").unwrap();
        documents_repository::save_body(&conn, "chapter", &chapter.id, "その夜、時計が止まった。").unwrap();

        let blocks = build_analysis_context(&conn, &project.id, "contradiction", None).unwrap();
        let labels: Vec<&str> = blocks.iter().map(|b| b.label.as_str()).collect();
        assert!(labels.contains(&"作品全文"));

        let style_blocks = build_analysis_context(&conn, &project.id, "style", None).unwrap();
        let style_labels: Vec<&str> = style_blocks.iter().map(|b| b.label.as_str()).collect();
        assert!(style_labels.contains(&"文体統計"));
    }
}
