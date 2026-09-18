//! Markdown出力。章を`##`、シーンを`###`の見出しにする(作品タイトルは
//! `#`)。本文中の`#`や`*`等Markdown上意味を持つ文字はエスケープしない
//! (小説本文でこれらの文字が見出し等として誤解釈される可能性は低く、
//! 過剰なエスケープでかえって読みにくくなることを避けた)。

use super::ExportChapter;
use crate::models::Project;

fn paragraphs_to_markdown(body: &str) -> String {
    // 空行区切りで段落と見なし、段落内の単一改行はMarkdown上そのままだと
    // 連結されてしまうため、行末に半角スペース2つ(改行の慣習)を足す。
    body.lines().map(|line| if line.trim().is_empty() { String::new() } else { format!("{line}  ") }).collect::<Vec<_>>().join("\n")
}

pub fn build(project: &Project, chapters: &[ExportChapter]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", project.title));
    if let Some(author) = project.pen_name.as_ref().or(project.author_name.as_ref()) {
        if !author.is_empty() {
            out.push_str(&format!("*{author}*\n\n"));
        }
    }

    for chapter in chapters {
        out.push_str(&format!("## {}\n\n", chapter.title));
        if let Some(body) = &chapter.chapter_body {
            out.push_str(&paragraphs_to_markdown(body));
            out.push_str("\n\n");
        }
        for scene in &chapter.scenes {
            if !scene.title.is_empty() {
                out.push_str(&format!("### {}\n\n", scene.title));
            }
            out.push_str(&paragraphs_to_markdown(&scene.body));
            out.push_str("\n\n");
        }
    }

    out
}
