//! HTML出力。単一ファイルで完結する読みやすいHTML(縦書き・ルビ等は
//! 未対応 -- 原稿用紙ビューの印刷プレビューが担う領域のため、ここでは
//! 「どの端末でも開けるプレーンな横書き本文」に徹する)。

use super::ExportChapter;
use crate::models::Project;

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn body_to_html(body: &str) -> String {
    body.split("\n\n")
        .map(|para| {
            let escaped = escape_html(para.trim());
            if escaped.is_empty() {
                String::new()
            } else {
                format!("<p>{}</p>", escaped.replace('\n', "<br>"))
            }
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn build(project: &Project, chapters: &[ExportChapter]) -> String {
    let mut body_html = String::new();
    body_html.push_str(&format!("<h1>{}</h1>\n", escape_html(&project.title)));
    if let Some(author) = project.pen_name.as_ref().or(project.author_name.as_ref()) {
        if !author.is_empty() {
            body_html.push_str(&format!("<p class=\"author\">{}</p>\n", escape_html(author)));
        }
    }

    for chapter in chapters {
        body_html.push_str(&format!("<h2>{}</h2>\n", escape_html(&chapter.title)));
        if let Some(body) = &chapter.chapter_body {
            body_html.push_str(&body_to_html(body));
            body_html.push('\n');
        }
        for scene in &chapter.scenes {
            if !scene.title.is_empty() {
                body_html.push_str(&format!("<h3>{}</h3>\n", escape_html(&scene.title)));
            }
            body_html.push_str(&body_to_html(&scene.body));
            body_html.push('\n');
        }
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="UTF-8">
<title>{title}</title>
<style>
  body {{ font-family: "Noto Serif JP", "Hiragino Mincho ProN", serif; line-height: 2; max-width: 720px; margin: 40px auto; padding: 0 16px; color: #222; }}
  h1 {{ font-size: 1.8em; border-bottom: 2px solid #333; padding-bottom: 8px; }}
  h2 {{ font-size: 1.4em; margin-top: 2.5em; }}
  h3 {{ font-size: 1.1em; color: #555; }}
  p.author {{ color: #666; margin-top: -0.5em; }}
  p {{ margin: 0 0 1em; text-indent: 1em; }}
</style>
</head>
<body>
{body_html}
</body>
</html>
"#,
        title = escape_html(&project.title)
    )
}
