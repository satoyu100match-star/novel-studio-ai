//! EPUB3出力。XHTML+CSSベースなので(DOCXと同様)日本語フォントの埋め込み
//! は不要 -- 読者側のリーダーアプリが持つフォントで表示される。
//! 章ごとに1つのXHTMLファイルを作り、シーンはその中の`<section>`にする
//! (ファイル数を抑え、目次(nav)をシンプルに保つための判断)。

use super::{build_zip, ExportChapter};
use crate::error::AppResult;
use crate::models::Project;

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn body_to_xhtml(body: &str) -> String {
    body.split("\n\n")
        .map(|para| {
            let escaped = escape_xml(para.trim());
            if escaped.is_empty() {
                String::new()
            } else {
                format!("<p>{}</p>", escaped.replace('\n', "<br/>"))
            }
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn chapter_xhtml(title: &str, chapter: &ExportChapter) -> String {
    let mut content = String::new();
    content.push_str(&format!("<h1>{}</h1>\n", escape_xml(title)));
    if let Some(body) = &chapter.chapter_body {
        content.push_str(&body_to_xhtml(body));
        content.push('\n');
    }
    for scene in &chapter.scenes {
        content.push_str("<section>\n");
        if !scene.title.is_empty() {
            content.push_str(&format!("<h2>{}</h2>\n", escape_xml(&scene.title)));
        }
        content.push_str(&body_to_xhtml(&scene.body));
        content.push_str("\n</section>\n");
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="ja">
<head><meta charset="UTF-8"/><title>{title}</title><link rel="stylesheet" type="text/css" href="style.css"/></head>
<body>
{content}
</body>
</html>"#,
        title = escape_xml(title)
    )
}

const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
<rootfiles>
<rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
</rootfiles>
</container>"#;

const STYLE_CSS: &str = r#"body { font-family: serif; line-height: 1.9; margin: 1em; }
h1 { font-size: 1.6em; border-bottom: 1px solid #999; padding-bottom: 0.3em; }
h2 { font-size: 1.2em; color: #444; }
p { margin: 0 0 1em; text-indent: 1em; }"#;

pub fn build(project: &Project, chapters: &[ExportChapter]) -> AppResult<Vec<u8>> {
    let book_id = format!("urn:uuid:{}", uuid::Uuid::new_v4());
    let author = project.pen_name.as_deref().or(project.author_name.as_deref()).unwrap_or("");

    let mut manifest_items = String::new();
    let mut spine_items = String::new();
    let mut nav_items = String::new();
    let mut chapter_files: Vec<(String, Vec<u8>)> = Vec::new();

    for (i, chapter) in chapters.iter().enumerate() {
        let file_name = format!("chapter-{}.xhtml", i + 1);
        let id = format!("ch{}", i + 1);
        manifest_items.push_str(&format!(
            r#"<item id="{id}" href="{file_name}" media-type="application/xhtml+xml"/>"#
        ));
        spine_items.push_str(&format!(r#"<itemref idref="{id}"/>"#));
        nav_items.push_str(&format!(
            r#"<li><a href="{file_name}">{title}</a></li>"#,
            title = escape_xml(&chapter.title)
        ));
        chapter_files.push((format!("OEBPS/{file_name}"), chapter_xhtml(&chapter.title, chapter).into_bytes()));
    }

    let content_opf = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="bookid" xml:lang="ja">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:identifier id="bookid">{book_id}</dc:identifier>
<dc:title>{title}</dc:title>
<dc:language>ja</dc:language>
<dc:creator>{author}</dc:creator>
</metadata>
<manifest>
<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
<item id="css" href="style.css" media-type="text/css"/>
{manifest_items}
</manifest>
<spine>
{spine_items}
</spine>
</package>"#,
        title = escape_xml(&project.title),
        author = escape_xml(author),
    );

    let nav_xhtml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="ja">
<head><meta charset="UTF-8"/><title>目次</title></head>
<body>
<nav epub:type="toc" id="toc"><h1>目次</h1><ol>
{nav_items}
</ol></nav>
</body>
</html>"#
    );

    let mut entries: Vec<(&str, Vec<u8>)> = vec![
        ("mimetype", b"application/epub+zip".to_vec()),
        ("META-INF/container.xml", CONTAINER_XML.as_bytes().to_vec()),
        ("OEBPS/content.opf", content_opf.into_bytes()),
        ("OEBPS/nav.xhtml", nav_xhtml.into_bytes()),
        ("OEBPS/style.css", STYLE_CSS.as_bytes().to_vec()),
    ];
    for (name, data) in &chapter_files {
        entries.push((name.as_str(), data.clone()));
    }

    build_zip(entries)
}
