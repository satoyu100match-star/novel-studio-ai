//! プレーンテキスト出力。原稿の生データにいちばん近い形式で、ルビ
//! (`｜base《reading》`)や傍点(`[#傍点]...[#傍点終わり]`)の記法も
//! そのまま残す(青空文庫形式として認識できるテキストのため、変換せず
//! 保持するのが最も情報を失わない選択 -- docs/MANUSCRIPT.md参照)。

use super::ExportChapter;
use crate::models::Project;

pub fn build(project: &Project, chapters: &[ExportChapter]) -> String {
    let mut out = String::new();
    out.push_str(&project.title);
    out.push('\n');
    if let Some(author) = project.pen_name.as_ref().or(project.author_name.as_ref()) {
        if !author.is_empty() {
            out.push_str(author);
            out.push('\n');
        }
    }
    out.push('\n');

    for chapter in chapters {
        out.push_str(&chapter.title);
        out.push_str("\n\n");
        if let Some(body) = &chapter.chapter_body {
            out.push_str(body);
            out.push_str("\n\n");
        }
        for scene in &chapter.scenes {
            if !scene.title.is_empty() {
                out.push_str(&scene.title);
                out.push_str("\n\n");
            }
            out.push_str(&scene.body);
            out.push_str("\n\n");
        }
    }

    out
}
