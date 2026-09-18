//! PDF出力(標準の横書き本文レイアウト)。
//!
//! 「原稿用紙PDF」(縦書き・禁則処理・ルビ・傍点込みの本組版)は、
//! 既に完成している原稿用紙ビュー(`docs/MANUSCRIPT.md`、Phase2)の
//! 印刷プレビュー(`window.print()` → OS/ブラウザの「PDFとして保存」)
//! を使う方針とし、ここでは実装しない(export/mod.rsのモジュール
//! コメント参照)。ここで作るのは「本文をシンプルな横書きPDFとして
//! さっと書き出したい」というニーズに応える簡易版。
//!
//! 日本語を正しく表示するため、PDF標準14フォント(Latin専用)ではなく
//! IPAゴシック(IPAフォントライセンスv1.0、`assets/fonts/`に同梱)を
//! 埋め込む。改行位置は正確なグリフ幅測定ではなく「1文字=1em」という
//! 近似(和文フォントはほぼ正方形のため実用上十分)で、ページ幅から
//! 1行あたりの文字数を概算している。

use super::ExportChapter;
use crate::models::Project;
use printpdf::*;

static IPA_GOTHIC_TTF: &[u8] = include_bytes!("../../assets/fonts/ipag.ttf");

const PAGE_WIDTH_MM: f32 = 210.0; // A4
const PAGE_HEIGHT_MM: f32 = 297.0;
const MARGIN_MM: f32 = 20.0;
const BODY_FONT_SIZE_PT: f32 = 10.5;
const HEADING1_FONT_SIZE_PT: f32 = 16.0;
const HEADING2_FONT_SIZE_PT: f32 = 12.5;
const LINE_HEIGHT_FACTOR: f32 = 1.9;
/// 1pt = 1/72インチ = 25.4/72 mm。
const MM_PER_PT: f32 = 25.4 / 72.0;

struct PageCursor {
    font_id: FontId,
    pages: Vec<PdfPage>,
    ops: Vec<Op>,
    y_mm: f32,
}

impl PageCursor {
    fn new(font_id: FontId) -> Self {
        Self { font_id, pages: Vec::new(), ops: vec![Op::StartTextSection], y_mm: PAGE_HEIGHT_MM - MARGIN_MM }
    }

    fn flush_page(&mut self) {
        self.ops.push(Op::EndTextSection);
        let ops = std::mem::replace(&mut self.ops, vec![Op::StartTextSection]);
        self.pages.push(PdfPage::new(Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), ops));
        self.y_mm = PAGE_HEIGHT_MM - MARGIN_MM;
    }

    fn ensure_space(&mut self, needed_mm: f32) {
        if self.y_mm - needed_mm < MARGIN_MM {
            self.flush_page();
        }
    }

    fn line(&mut self, text: &str, size_pt: f32) {
        if text.is_empty() {
            return;
        }
        let line_h_mm = size_pt * MM_PER_PT * LINE_HEIGHT_FACTOR;
        self.ensure_space(line_h_mm);
        self.ops.push(Op::SetFont { font: PdfFontHandle::External(self.font_id.clone()), size: Pt(size_pt) });
        self.ops.push(Op::SetLineHeight { lh: Pt(size_pt * LINE_HEIGHT_FACTOR) });
        self.ops.push(Op::SetFillColor { col: Color::Rgb(Rgb { r: 0.1, g: 0.1, b: 0.1, icc_profile: None }) });
        self.ops.push(Op::SetTextCursor { pos: Point::new(Mm(MARGIN_MM), Mm(self.y_mm)) });
        self.ops.push(Op::ShowText { items: vec![TextItem::Text(text.to_string())] });
        self.y_mm -= line_h_mm;
    }

    fn blank(&mut self, size_pt: f32) {
        let h = size_pt * MM_PER_PT * LINE_HEIGHT_FACTOR * 0.5;
        if self.y_mm - h < MARGIN_MM {
            self.flush_page();
        } else {
            self.y_mm -= h;
        }
    }

    fn heading(&mut self, text: &str, size_pt: f32) {
        self.line(text, size_pt);
        self.blank(BODY_FONT_SIZE_PT);
    }

    fn paragraphs(&mut self, body: &str, size_pt: f32) {
        let usable_width_mm = PAGE_WIDTH_MM - 2.0 * MARGIN_MM;
        let char_width_mm = size_pt * MM_PER_PT;
        let chars_per_line = ((usable_width_mm / char_width_mm).floor() as usize).max(1);

        for para in body.split('\n') {
            if para.trim().is_empty() {
                self.blank(size_pt);
                continue;
            }
            let chars: Vec<char> = para.chars().collect();
            for chunk in chars.chunks(chars_per_line) {
                let line_text: String = chunk.iter().collect();
                self.line(&line_text, size_pt);
            }
        }
        self.blank(size_pt);
    }

    fn finish(mut self) -> Vec<PdfPage> {
        if self.ops.len() > 1 {
            self.flush_page();
        }
        self.pages
    }
}

pub fn build(project: &Project, chapters: &[ExportChapter]) -> Vec<u8> {
    let mut doc = PdfDocument::new(&project.title);
    let mut warnings = Vec::new();
    let parsed_font =
        ParsedFont::from_bytes(IPA_GOTHIC_TTF, 0, &mut warnings).expect("embedded IPA Gothic font must parse");
    let font_id = doc.add_font(&parsed_font);

    let mut cursor = PageCursor::new(font_id);

    cursor.heading(&project.title, HEADING1_FONT_SIZE_PT);
    if let Some(author) = project.pen_name.as_ref().or(project.author_name.as_ref()) {
        if !author.is_empty() {
            cursor.line(author, BODY_FONT_SIZE_PT);
        }
    }
    cursor.blank(BODY_FONT_SIZE_PT);

    for chapter in chapters {
        cursor.heading(&chapter.title, HEADING1_FONT_SIZE_PT);
        if let Some(body) = &chapter.chapter_body {
            cursor.paragraphs(body, BODY_FONT_SIZE_PT);
        }
        for scene in &chapter.scenes {
            if !scene.title.is_empty() {
                cursor.heading(&scene.title, HEADING2_FONT_SIZE_PT);
            }
            cursor.paragraphs(&scene.body, BODY_FONT_SIZE_PT);
        }
    }

    let pages = cursor.finish();
    let pages = if pages.is_empty() { vec![PdfPage::new(Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), vec![])] } else { pages };

    let mut save_warnings = Vec::new();
    doc.with_pages(pages).save(&PdfSaveOptions::default(), &mut save_warnings)
}
