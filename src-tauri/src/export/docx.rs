//! DOCX(Word)出力。OOXML WordprocessingMLを直接組み立て、ZIPコンテナに
//! 詰める(`docx-rs`等の専用クレートは使わず、必要最小限のXMLを自前で
//! 生成する方が構造を完全に把握でき、依存を増やさずに済むため)。
//! 日本語フォントはWord側が持っているものをそのまま使うため、PDFとは
//! 異なりフォント埋め込みは不要(テキストはただのUTF-8として書き出す)。

use super::{build_zip, ExportChapter};
use crate::error::AppResult;
use crate::models::Project;

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}

fn heading_paragraph(style: &str, text: &str) -> String {
    format!(
        r#"<w:p><w:pPr><w:pStyle w:val="{style}"/></w:pPr><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>"#,
        style = style,
        text = escape_xml(text)
    )
}

fn body_paragraphs(body: &str) -> String {
    body.lines()
        .map(|line| {
            format!(r#"<w:p><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#, escape_xml(line))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn document_xml(project: &Project, chapters: &[ExportChapter]) -> String {
    let mut body = String::new();
    body.push_str(&heading_paragraph("Title", &project.title));
    if let Some(author) = project.pen_name.as_ref().or(project.author_name.as_ref()) {
        if !author.is_empty() {
            body.push_str(&format!(r#"<w:p><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#, escape_xml(author)));
        }
    }

    for chapter in chapters {
        body.push_str(&heading_paragraph("Heading1", &chapter.title));
        if let Some(chapter_body) = &chapter.chapter_body {
            body.push_str(&body_paragraphs(chapter_body));
        }
        for scene in &chapter.scenes {
            if !scene.title.is_empty() {
                body.push_str(&heading_paragraph("Heading2", &scene.title));
            }
            body.push_str(&body_paragraphs(&scene.body));
        }
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>
{body}
<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1417" w:right="1417" w:bottom="1417" w:left="1417"/></w:sectPr>
</w:body>
</w:document>"#
    )
}

const CONTENT_TYPES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#;

const ROOT_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

const DOCUMENT_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;

const STYLES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:rPr><w:sz w:val="22"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:rPr><w:b/><w:sz w:val="40"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="32"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:pPr><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/><w:sz w:val="26"/></w:rPr></w:style>
</w:styles>"#;

pub fn build(project: &Project, chapters: &[ExportChapter]) -> AppResult<Vec<u8>> {
    let document_xml_content = document_xml(project, chapters);
    build_zip(vec![
        ("[Content_Types].xml", CONTENT_TYPES_XML.as_bytes().to_vec()),
        ("_rels/.rels", ROOT_RELS_XML.as_bytes().to_vec()),
        ("word/document.xml", document_xml_content.into_bytes()),
        ("word/_rels/document.xml.rels", DOCUMENT_RELS_XML.as_bytes().to_vec()),
        ("word/styles.xml", STYLES_XML.as_bytes().to_vec()),
    ])
}
