# EXPORT.md — Export機能設計（Phase 9）

## 1. 対応フォーマット

TXT / Markdown / HTML / DOCX（Word） / PDF（簡易・横書き） / EPUB の6種。
`src-tauri/src/export/mod.rs`の`EXPORT_FORMATS`が真実源で、コマンド
`list_export_formats`経由でフロントの選択UIもここから動的に構築する
（選択肢のハードコードによるズレを防ぐため）。

## 2. 共通パイプライン

```
gather(project_id)
  → Project + Vec<ExportChapter>（章 → シーンの単純な木構造）
  → フォーマットごとのbuilder（txt/markdown/html/docx/pdf/epub）
  → ExportOutput { filename, mime_type, bytes }
```

`gather`は`documents`テーブルから章/シーンの本文を取得し、空の本文は
読み飛ばす（章立てだけのプロットも構成を確認できるよう、空の章タイトル
自体は出力に残す）。パート（Part）情報は現状Exportの構造には畳み込まず、
章の並び順のみを使う。

`ExportOutput`はテキスト・バイナリを問わず常に`bytes: Vec<u8>`で返す統一
インターフェース。Tauri IPC（serde）がこれをそのままJSON数値配列へ
シリアライズするため、base64エンコーダ等を自前実装する必要がない。フロント
側は`new Blob([new Uint8Array(bytes)], {type: mimeType})` → Blob URL →
`<a download>`クリックという1経路だけで、全フォーマットを扱える
（`src/services/exportService.ts`の`downloadExportOutput`）。

## 3. フォーマットごとの実装方針

- **TXT/Markdown/HTML**: 依存クレートなし。素朴な文字列組み立てのみ
  （`export/txt.rs` / `markdown.rs` / `html.rs`）。HTMLは単一ファイルで
  完結する自己完結型（インラインCSS）。
- **DOCX**: `docx-rs`等の専用クレートは使わず、OOXML WordprocessingML
  （`[Content_Types].xml` / `_rels/.rels` / `word/document.xml` /
  `word/styles.xml`）を直接組み立て、`zip`クレートでZIPコンテナに
  詰める。日本語フォントはWord側が持つフォントをそのまま使うため埋め込み
  不要。
- **EPUB**: `epub-builder`等は使わず、EPUB3の最小構成（`mimetype` /
  `META-INF/container.xml` / `OEBPS/content.opf` / `OEBPS/nav.xhtml` /
  章ごとの`OEBPS/chapter-N.xhtml`）を直接組み立て、`zip`で詰める。
  XHTML+CSSベースのためフォント埋め込みは不要（読者のリーダーアプリが
  持つフォントで表示される）。
- **PDF（簡易・横書き）**: `printpdf`クレートを使用。PDF標準14フォントは
  Latin専用で日本語を表示できないため、IPAゴシック
  （`src-tauri/assets/fonts/ipag.ttf`、IPAフォントライセンスv1.0、
  同梱の`IPA_Font_License.txt`参照）をコンパイル時に`include_bytes!`で
  埋め込み、実行時のフォント配置・パス解決の複雑さを避けている。
  改行位置は正確なグリフ幅測定ではなく「1文字=1em」という近似
  （和文フォントはほぼ正方形のため実用上十分）でページ幅から1行あたりの
  文字数を概算している。

## 4. 「原稿用紙PDF」を独立実装しない判断

仕様が想定する縦書き・禁則処理・ルビ・傍点込みの本格的な組版PDF
（いわゆる原稿用紙PDF）は、このPhaseでは**独立したPDF生成パイプラインを
新設しない**。代わりに、Phase 2で既に完成している原稿用紙ビュー
（`docs/MANUSCRIPT.md`）の印刷プレビュー（`window.print()` →
OS/ブラウザの「PDFとして保存」）を引き続き使う方針とする。

理由:

- 原稿用紙ビューは既に縦書き・禁則処理・ルビ・傍点・縦中横を正しく
  組版できており、テスト済み（Phase 2〜3で作り込み済み）。
- これをRust側のPDF生成ロジックとして再実装するのは、複雑さとリスクの
  わりに得られる価値が小さい（既存の動く機能を壊すリスクの方が大きい）。
- `CLAUDE.md`「判断に迷った場合の優先順位」（`データ安全性 > 執筆体験 >
  安定性 > ...`）に照らし、安定性を優先した。

Exportパネル（`src/features/export/ExportPanel.tsx`）には、原稿用紙PDFが
必要な場合は原稿用紙ビューの印刷プレビューを使うよう案内文を表示している。

## 5. 既知の制約

- PDFの改行判定は「1文字=1em」の近似であり、実際のグリフ幅とは厳密には
  一致しない（欧文・半角文字が混ざると特に誤差が出る）。長文で厳密な
  行送りが必要な場合は原稿用紙ビューの印刷プレビューを使うこと。
- DOCX/EPUBともに章・シーンの本文は改行のみを段落境界として扱う簡易的な
  変換であり、ルビ・傍点等の原稿用紙専用記法（`｜base《reading》`等）は
  そのままの文字列として出力される（変換・除去は行わない）。
- Export結果は毎回その場で生成するスナップショットであり、Export履歴や
  出力ファイルの管理機能（一覧・再ダウンロード等）はない。
