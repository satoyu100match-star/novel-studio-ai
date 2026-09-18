//! Phase 9: Export(TXT/Markdown/HTML/DOCX/PDF/EPUB)。
//!
//! すべてのフォーマットは同じ「作品構造の取得(`gather`)」から出発し、
//! フォーマットごとのビルダー(`txt`/`markdown`/`html`/`docx`/`pdf`/
//! `epub`)が同じ入力から出力バイト列を組み立てる。バイナリ・テキストを
//! 問わず常に`Vec<u8>`を返す統一インターフェースにすることで、フロント側
//! は常に同じ経路(Blobを作ってダウンロード)で扱える(`commands::export`
//! 参照)。
//!
//! 「原稿用紙PDF」は独立したPDF生成パイプラインを新設せず、Phase2から
//! 既にある原稿用紙ビューの印刷プレビュー(`window.print()` →
//! OS/ブラウザの「PDFとして保存」)を使う方針にしている。既に禁則処理・
//! ルビ・傍点・縦書きを正しく組版できているビューを再実装するコストと
//! リスクに見合わないと判断したため(CLAUDE.md「判断に迷った場合の
//! 優先順位」: 安定性を優先)。詳細はdocs/EXPORT.md。

pub mod docx;
pub mod epub;
pub mod html;
pub mod markdown;
pub mod pdf;
pub mod txt;

use crate::error::AppResult;
use crate::models::Project;
use crate::repositories::{chapters_repository, documents_repository, projects_repository, scenes_repository};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub const EXPORT_FORMATS: &[&str] = &["txt", "markdown", "html", "docx", "pdf", "epub"];

pub struct ExportScene {
    pub title: String,
    pub body: String,
}

pub struct ExportChapter {
    pub title: String,
    /// 章に直接ついた本文(シーン未使用の場合)。空なら`None`。
    pub chapter_body: Option<String>,
    pub scenes: Vec<ExportScene>,
}

/// フロントへそのままJSONで返す(`bytes`は数値配列としてシリアライズされる
/// -- base64エンコーダを自前実装せずに済むようにするための意図的な選択。
/// フロント側は`new Blob([new Uint8Array(bytes)], {type: mimeType})`で
/// 扱う)。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportOutput {
    pub filename: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

/// DOCX/EPUBはどちらもZIPコンテナの中にXML/XHTMLを詰めたものなので、
/// 共通のZIP組み立てヘルパーをここに置く。圧縮は`Stored`(無圧縮)にして
/// いる -- テキストが主体のため圧縮の恩恵は小さく、圧縮コーデックの
/// 選択肢が増えることによる複雑さを避けた。
pub(crate) fn build_zip(entries: Vec<(&str, Vec<u8>)>) -> AppResult<Vec<u8>> {
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, data) in entries {
        writer
            .start_file(name, options)
            .map_err(|e| crate::error::AppError::Other(format!("ZIP作成に失敗しました({name}): {e}")))?;
        writer.write_all(&data)?;
    }
    let cursor = writer.finish().map_err(|e| crate::error::AppError::Other(format!("ZIP作成に失敗しました: {e}")))?;
    Ok(cursor.into_inner())
}

fn sanitize_filename(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| if r#"\/:*?"<>|"#.contains(c) { '_' } else { c })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() { "無題の作品".to_string() } else { trimmed.to_string() }
}

/// 作品全体をExport用の構造(パート情報は畳み込み、章→シーンの単純な
/// 木構造のみ)に組み立てる。空の章/シーン(本文未入力)はそのまま
/// `None`/空`Vec`として残し、各フォーマットのビルダー側で読み飛ばす
/// かどうかを判断する(章立てだけのプロットも構成を確認できるよう、
/// 空の章タイトル自体は出力に残す)。
pub fn gather(conn: &Connection, project_id: &str) -> AppResult<(Project, Vec<ExportChapter>)> {
    let project = projects_repository::get(conn, project_id)?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("project {project_id}")))?;

    let mut chapters = Vec::new();
    for c in chapters_repository::list_by_project(conn, project_id)? {
        let chapter_body = documents_repository::get(conn, "chapter", &c.id)?
            .map(|d| d.body)
            .filter(|b| !b.is_empty());

        let mut scenes = Vec::new();
        for s in scenes_repository::list_by_chapter(conn, &c.id)? {
            if let Some(doc) = documents_repository::get(conn, "scene", &s.id)? {
                if !doc.body.is_empty() {
                    scenes.push(ExportScene { title: s.title, body: doc.body });
                }
            }
        }
        chapters.push(ExportChapter { title: c.title, chapter_body, scenes });
    }

    Ok((project, chapters))
}

pub fn export(conn: &Connection, project_id: &str, format: &str) -> AppResult<ExportOutput> {
    if !EXPORT_FORMATS.contains(&format) {
        return Err(crate::error::AppError::Other(format!("未対応のExport形式です: {format}")));
    }
    let (project, chapters) = gather(conn, project_id)?;
    let base_name = sanitize_filename(&project.title);

    let (extension, mime_type, bytes) = match format {
        "txt" => ("txt", "text/plain", txt::build(&project, &chapters).into_bytes()),
        "markdown" => ("md", "text/markdown", markdown::build(&project, &chapters).into_bytes()),
        "html" => ("html", "text/html", html::build(&project, &chapters).into_bytes()),
        "docx" => (
            "docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            docx::build(&project, &chapters)?,
        ),
        "pdf" => ("pdf", "application/pdf", pdf::build(&project, &chapters)),
        "epub" => ("epub", "application/epub+zip", epub::build(&project, &chapters)?),
        _ => unreachable!(),
    };

    Ok(ExportOutput { filename: format!("{base_name}.{extension}"), mime_type: mime_type.to_string(), bytes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::{chapters_repository, documents_repository, projects_repository, scenes_repository};

    fn sample_project(conn: &Connection) -> String {
        let project = projects_repository::create(
            conn,
            &ProjectInput { title: "テスト小説".into(), author_name: Some("著者名".into()), ..Default::default() },
            false,
        )
        .unwrap();
        let chapter = chapters_repository::create(conn, &project.id, None, "第一章 出会い").unwrap();
        let scene = scenes_repository::create(conn, &chapter.id, "冒頭").unwrap();
        documents_repository::save_body(conn, "scene", &scene.id, "その日、空はとても青かった。\n\n彼女はゆっくりと歩き出した。").unwrap();
        project.id
    }

    #[test]
    fn gather_builds_chapter_scene_tree() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project_id = sample_project(&conn);

        let (project, chapters) = gather(&conn, &project_id).unwrap();
        assert_eq!(project.title, "テスト小説");
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].scenes.len(), 1);
        assert!(chapters[0].scenes[0].body.contains("空はとても青かった"));
    }

    #[test]
    fn export_rejects_unknown_format() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project_id = sample_project(&conn);
        assert!(export(&conn, &project_id, "not_a_format").is_err());
    }

    #[test]
    fn export_produces_nonempty_bytes_for_every_format() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project_id = sample_project(&conn);

        for format in EXPORT_FORMATS {
            let output = export(&conn, &project_id, format).unwrap();
            assert!(!output.bytes.is_empty(), "{format} produced empty output");
            assert!(output.filename.ends_with(&format!(".{}", if *format == "markdown" { "md" } else { format })));
        }
    }
}
