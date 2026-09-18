# Changelog

All notable changes to this project are documented in this file.

## [Phase 16] - 2026-09-19

### Fixed

- 単一ペイン構成のセクション(エクスポート/ゴミ箱/人物相関タブ等)で、
  画面の高さに収まらない量の文章・項目があると、スクロールバーが
  出ないまま下端で文字が見切れてしまう不具合を修正
  (`src/styles/global.css`の`.workspace__body--single`)。原因は、
  Character/World/Story/AI/分析/資料・メモ/ゴミ箱/エクスポートの
  全セクションの共通の親であるこのコンテナに`overflow-y: auto`が
  指定されておらず、内部で独自にスクロール領域を持たないパネル
  (`.detail-form`や`.card`を直接置いているエクスポート・ゴミ箱・
  人物相関タブなど)では、はみ出した内容が見た目上切れるだけで
  スクロールする手段が無かったこと(ユーザー報告により発覚)。共通の
  親コンテナ側でスクロールを保証するようにしたため、個々のパネルを
  1つずつ直す必要は無かった。既にパネル内部で`overflow-y: auto`を
  持つ構成(AI/分析セクションの内側ラッパー、各種`.entity-list`/
  `.timeline-list`/`.todo-list`等)には影響しない

### 動作確認済み

- `cargo build`(Rust側の変更なしのため`cargo test`は前回の53件のまま
  グリーン)
- `pnpm typecheck` / `pnpm lint` / `pnpm test`(40件、CSSのみの修正の
  ため件数据え置き) / `pnpm build` すべて成功
- Xvfb実機起動でアプリが正常に立ち上がることを確認(スクロール自体の
  目視確認はブラウザ/GUIビューアが無い本サンドボックスでは行えず、
  Windows実機での確認が必要)

## [Phase 15] - 2026-09-19

### Added

- Auto Update機能(`docs/UPDATE_RELEASE.md`参照)。ユーザーから
  「アップデートのたびに手動で入れ直すのが面倒」という要望を受け、
  `tauri-plugin-updater` + `tauri-plugin-process`(再起動用)を導入
  (`src-tauri/src/lib.rs`、デスクトップのみ・`cfg(desktop)`ガード)
- 起動のたびに一度だけバックグラウンドで新バージョンの有無を確認し、
  見つかった場合のみ画面上部にCrash Recoveryバナー(Phase8)と同じ
  見た目の案内を表示する`UpdateBanner`(`src/features/update/
  UpdateBanner.tsx`)。確認自体が失敗しても(オフライン等)通常の執筆
  機能には一切影響しない
- ダウンロード・適用・再起動は必ずバナーの「今すぐ更新」ボタンを押した
  時だけ実行し、バックグラウンドで勝手に更新が当たることはない
  (CLAUDE.md優先順位#1「データ安全性」を優先し、執筆中に予告なく
  再起動されて内容を見失う不安を避けるための設計判断)
- 「このアプリについて」→「概要」タブに、いつでも手動で確認できる
  「アップデートを確認」ボタンを追加(`src/features/about/
  AboutModal.tsx`)
- 呼び出しは`src/services/updateService.ts`の1箇所に閉じ込め、feature/
  コンポーネントから`@tauri-apps/plugin-updater`を直接importしない
  (既存の「featureやコンポーネントから`@tauri-apps/api`を直接
  importしない」方針を踏襲)
- アップデート用の署名鍵ペア(Ed25519)を生成。公開鍵は
  `src-tauri/tauri.conf.json`の`plugins.updater.pubkey`に埋め込み、
  秘密鍵は本人にのみ別途安全な方法で受け渡し、リポジトリには含めない
  (`.gitignore`に`*.key`を追加し誤コミット経路を遮断)
- GitHub Actionsによるリリース自動化(`.github/workflows/release.yml`):
  `tauri-apps/tauri-action`を使い、`v*`タグのpushをトリガーに
  Windowsランナー上で署名付き成果物と`latest.json`を生成し、GitHub
  Releaseの下書き(Draft)として公開する。開発者側での毎回の再ビルド・
  再インストールが不要になる
- 詳細な設計・署名鍵の取り扱い・リリース手順を`docs/UPDATE_RELEASE.md`
  にまとめた

### 動作確認済み

- `cargo build`(新規プラグイン組み込み後)、`cargo test`(53件、
  Rustユニットテスト自体への変更は無いため件数据え置きで全件グリーン
  を再確認)
- `pnpm typecheck` / `pnpm lint` / `pnpm test`(40件、UI配線が中心の
  追加のため件数据え置き) / `pnpm build` すべて成功(updater/process
  向けの新しい遅延読み込みチャンクを確認)
- Xvfb実機起動でアプリが正常に立ち上がることを確認

### 既知の制約

- このサンドボックス環境では、実際にGitHub Releaseを介した更新の
  ダウンロード・適用・再起動までは検証できていない(署名鍵の生成、
  プラグインの組み込み、`cargo build`、Xvfb起動確認までは実施済み)。
  実際にリリースを1つ公開し、旧バージョンのアプリから更新できることを
  確認するまでは既知の制約として扱う
- `.github/workflows/release.yml`自体も実際に一度動かして確認するまで
  未検証
- `tauri.conf.json`の`plugins.updater.endpoints`は
  `https://github.com/satoyu100match-star/novel-studio-ai/releases/
  latest/download/latest.json`に確定した。ただしこのリポジトリへの
  最初のリリース公開が済むまではアップデート確認は「更新なし」と
  同じ挙動になる(通常の起動・執筆機能には影響しない)

## [Phase 14] - 2026-09-19

### Fixed

- 作品一覧から作品を削除できない不具合を修正(`src/features/projects/
  ProjectListPage.tsx`)。バックエンドの`delete_project`コマンド
  (Soft Delete)とストアの`useProjectListStore.remove`はPhase1時点で
  実装済みだったが、作品一覧画面にそれを呼び出すボタンが実装されて
  おらず、UIからは作品を削除する手段が無かった(ユーザー報告により発覚)。
  各作品の行に確認ダイアログ付きの「削除」ボタンを追加した。バック
  エンド・DB側の変更は不要(既存の未配線の経路をUIに繋いだのみ)

### 動作確認済み

- `cargo build`(Rust側の変更なしのため`cargo test`は前回の53件のまま
  グリーン)
- `pnpm typecheck` / `pnpm lint` / `pnpm test`(40件、UI配線のみの修正
  のため件数据え置き) / `pnpm build` すべて成功
- Xvfb実機起動でアプリが正常に立ち上がることを確認

## [Phase 13] - 2026-09-19

### Added

- テーマを5種類に拡張(`src/stores/themeStore.ts`): 従来のライト/ダークに
  加え、「かわいい系」「かっこいい系」「パンク系」を追加。トップバーの
  テーマ切替ボタン(☀/☾のトグル)を、全テーマから選べるセレクト
  (`.theme-select`)に置き換えた。色(`--color-*`)のみをテーマごとに
  変え、フォント(`--font-ui`/`--font-manuscript`)は全テーマで共通のまま
  据え置いている(原稿用紙の組版・日本語IME入力の見え方に影響する変更は
  避け、「エディタ領域の視認性を最優先する」方針を保つため)。既存の
  CSS変数ベースの設計(`src/styles/theme.css`)をそのまま踏襲しており、
  `data-theme`属性の値を増やしただけでコンポーネント側の変更は不要
  だった
- AI執筆支援(`AiWritingAssistPanel`)の分かりやすさを改善:
  - 機能選択プルダウンを「本文編集(本文の選択が必要)」/「アイデア生成
    (本文選択は不要)」の2グループ(`optgroup`)に分け、選択中の機能が
    何をするかを1行で説明する`AI_WRITING_FEATURE_DESCRIPTIONS`
    (`src/types/tauriCommands.ts`)を追加
  - 新しい共通コンポーネント`AiFlowSteps`(`src/features/ai/
    AiFlowSteps.tsx`)を追加し、「① 入力 → ② 送信内容の確認 → ③ 結果の
    確認・適用」という操作の流れを常に画面上に表示するようにした
    (`AiWritingAssistPanel`と、Phase12の`ConceptChatPanel`の両方で使用)

### 動作確認済み

- `cargo build`(Rust側の変更なしのため`cargo test`は前回の53件のまま
  グリーン)
- `pnpm typecheck` / `pnpm lint` / `pnpm test`(40件、新規4件:
  テーマ拡張後の`nextTheme`のフォールバック挙動・`THEME_OPTIONS`/
  `THEME_LABELS`の整合性・`setTheme`が新テーマを受け付けることの確認) /
  `pnpm build` すべて成功
- Xvfb実機起動でアプリが正常に立ち上がることを確認

## [Phase 12] - 2026-09-19

### Added

- 作品設計チャット(`src/features/ai/ConceptChatPanel.tsx`、AIセクションの
  新サブタブ「作品設計チャット」): 作りたい作品の自由記述のアイデアから、
  タイトル案・あらすじ・キャラクター(複数)・世界観項目(複数)・プロット
  カード(複数)を一括でAIに提案させる機能。既存のAI小説支援機能
  (Phase6)と同じく「入力 → Context Inspectorで送信内容確認 → 送信 →
  結果確認 → 適用」の順を必ず踏み、提案を受け取った時点では何も確定
  しない。結果はタイトル/あらすじ/キャラクター/世界観/プロットの
  セクションごとにチェックボックス付きで一覧表示され、選んだ項目だけが
  既存の`create_character`/`create_world_entry`/`create_plot_card`等の
  コマンドを通じて実際に作成される(新しいテーブル・新しい書き込み経路は
  追加していない、既存の作成経路をAIの一括提案から呼び出せるようにした
  だけ)
- バックエンド: `src-tauri/src/ai/features.rs`に新しい機能ID
  `generate_concept`を追加。既存の`run_ai_writing_feature`コマンドを
  そのまま流用できるよう、応答を`==TITLE==`/`==SYNOPSIS==`/
  `==CHARACTERS==`/`==WORLD==`/`==PLOT==`という見出し区切りの構造化
  テキストで返すようプロンプトを設計。新しいTauriコマンドの追加は不要
  だった
- フロントエンド: `src/features/ai/parseGenerated.ts`に
  `parseCharacterDrafts`/`parseWorldEntryDrafts`(複数件版)と
  `parseConceptDraft`(見出しごとにセクションを正規表現で独立して
  切り出し、各セクションを上記のパーサへ委譲)を追加。あるセクションの
  見出しが欠けていたり崩れていても、他のセクションの読み取りには
  影響しない設計

### 動作確認済み

- `cargo build` / `cargo test`(53件、新規1件: プロンプトが全見出しを
  含むことの確認)すべて成功
- `pnpm typecheck` / `pnpm lint` / `pnpm test`(36件、新規5件:
  `parseConceptDraft`の正常系・セクション欠落時・完全に読み取れない
  場合の3パターンと、複数件パーサの2パターン) / `pnpm build` すべて成功
- Xvfb実機起動でアプリが正常に立ち上がることを確認

### 既知の制約

- 他のAI機能と同じ理由(本サンドボックスのネットワーク制約)で、実際の
  AIプロバイダーからの応答での動作(見出し形式からの崩れへの頑健性)は
  未検証。長編作品向けの要約送信等は行わず、Context Builderの候補
  ブロックをそのまま送る点も他機能と同様

## [Phase 11] - 2026-09-18

### Added

- Command Palette(Ctrl+K / Cmd+K、またはトップバーの「コマンド(Ctrl+K)」
  ボタン、`src/features/commandPalette/CommandPalette.tsx`): 作品一覧へ
  移動・サンプル作品を開く・テーマ切替・「このアプリについて」を開く、
  といったグローバル操作と、作品を開いている場合はワークスペースの
  各セクションへのジャンプ・章タイトル検索での原稿ジャンプに対応
- ワークスペースのセクション状態を`ProjectWorkspacePage`のローカル
  useStateから`src/stores/workspaceUiStore.ts`(Zustand)へ引き上げ
- `AppRouter`をルート定義のみへリファクタリングし、`HashRouter`を
  アプリシェル全体の外側で1つだけラップするよう変更(Command Paletteが
  Route要素の外から`useNavigate`を使えるようにするため)

### 意図的にスコープ外とした項目

仕様書自身が「初期販売版には必須ではない」と明示する高度機能(Semantic
Search / Map / Custom templates / Plugin architecture / Local AI /
Cloud sync / Multi-device / Collaboration)は、いずれも現在のローカル
SQLite単体構成からの大きなアーキテクチャ変更を要するため、今回は
実装しない。理由の詳細: `docs/ROADMAP.md` Phase11。

これでPhase 0〜11、計画していた全Phaseが完了した。

### 動作確認済み

- `pnpm typecheck` / `pnpm lint` / `pnpm test`(31件) / `pnpm build`
  すべて成功
- `cargo build` / `cargo test`(52件、Rust側の変更なしのため件数据え置き)
  すべて成功
- Xvfb実機起動で`HashRouter`の位置変更後もアプリが正常に立ち上がる
  ことを確認

## [Phase 10] - 2026-09-18

### Added

- Onboarding: 初回起動時のみ表示する4ステップの案内モーダル
  （`src/features/onboarding/OnboardingModal.tsx`）。`app_settings`の
  "app.onboarding_seen"で表示済みを記録し、いつでもスキップ可能
- Sample Project: 作品一覧画面の「サンプル作品を開く」ボタンから、
  人物2名・世界観2件・場所1件・用語1件・章1(実際の日本語の文章入りの
  シーン2つ)・プロットカード2件・時系列イベント1件・伏線1件・TODO2件が
  ひと通り揃ったサンプル作品を作成できる（`src-tauri/src/
  sample_project.rs`、`create_sample_project`コマンド）。`is_sample`
  フラグ自体はPhase1から存在していたが、実際に生成する経路が無かった
- 「このアプリについて」画面（`src/features/about/AboutModal.tsx`）:
  概要(バージョン情報)・更新履歴(CHANGELOG.md)・ライセンス
  (THIRD_PARTY_NOTICES.md新設)・プライバシー(PRIVACY.md新設)の4タブ。
  いずれもリポジトリ直下のMarkdownを`include_str!`でそのまま埋め込み、
  アプリ内に別コピーを持たない
- 診断ログの書き出し(`export_diagnostics`コマンド): ログフォルダの
  中身をZIPにまとめてダウンロードさせる。自動送信は一切行わない
  （テレメトリ非搭載の方針。詳細はPRIVACY.md）
- `cargo test` 1件追加（計52件: sample_project::create が人物・世界観・
  原稿・プロット・時系列・伏線・TODOをすべて正しく生成することの確認）
- `cargo tauri build --bundles deb`でLinux `.deb`バンドルの生成に成功
  （パッケージング設定`tauri.conf.json`が機能することの実機確認）

### 意図的な設計判断・見送った項目

- **Auto Updateは実装しない**: 署名鍵・更新サーバー・実際のWindows/
  macOS配布環境が必要で、このサンドボックスでは検証も安全な実装も
  できないため見送った。リリース運用開始前に必ず設計・実装が必要な
  既知の制約として記録する
- Windows向けインストーラ(`.msi`等)の生成・署名は未検証（Windows環境が
  必要なため）。Onboardingは固定ステップの静的案内のみで対話的
  チュートリアルではない。サンプル作品は1種類のみ。詳細:
  `CLAUDE.md`「未完成機能」

### 動作確認済み

- `pnpm typecheck` / `pnpm lint` / `pnpm test`(31件) / `pnpm build`
  すべて成功
- `cargo build` / `cargo test`(52件) すべて成功
- Xvfb実機起動でアプリが正常に立ち上がり、新設の`sample_project`/
  `about`モジュール・コマンド登録がアプリ初期化を壊していないことを
  確認（マイグレーション1〜7がすべて正しく適用されることも再確認）

## [Phase 9] - 2026-09-18

### Added

- Export機能（`src-tauri/src/export/`）: TXT / Markdown / HTML / DOCX /
  PDF(簡易・横書き) / EPUB の6形式。共通の`gather()`(作品→章→シーンの
  木構造取得)から出発し、フォーマット別builderが`ExportOutput{filename,
  mime_type, bytes}`という統一形式で返す設計
- DOCX(OOXML WordprocessingML)・EPUB(EPUB3)はいずれも専用クレートを
  使わず必要最小限のXML/XHTMLを直接組み立て、`zip`クレートでZIP
  コンテナに詰める
- PDF出力は`printpdf`クレートを使用。IPAゴシックフォント(IPAフォント
  ライセンスv1.0、`src-tauri/assets/fonts/ipag.ttf`)をコンパイル時に
  埋め込み、日本語を正しく表示する
- ワークスペースに「エクスポート」セクションタブを追加
  （`src/features/export/ExportPanel.tsx`）。書き出しはBlobダウンロード
  経路のみで、原稿データへの書き込みは一切発生しない
- Tauriコマンド`list_export_formats` / `export_project`を追加
- `docs/EXPORT.md`新設（Export設計・フォーマット別方針・原稿用紙PDFを
  独立実装しない判断の詳細）
- `cargo test` 3件追加（計51件: export::gather/未対応形式拒否/全形式が
  非空バイト列を返すことの確認）

### 意図的な設計判断

- **原稿用紙PDF(縦書き・禁則処理・ルビ・傍点込みの本組版)は独立実装
  しない**: Phase2で既に完成している原稿用紙ビューの印刷プレビュー
  （`window.print()`→「PDFとして保存」）を引き続き使う方針。既に
  組版済みの動くビューを再実装するリスクに見合わないと判断
  （`CLAUDE.md`優先順位: 安定性を優先）。詳細: `docs/EXPORT.md`4章
- 【技術的環境の変化】crates.ioへのネットワークアクセスがPhase5当時の
  障害から回復していることを確認し、`zip`・`printpdf`クレートを新規
  追加した(実クレートを使う設計に戻せた)。ただしPhase5で暫定実装した
  `ai::http_client`(curlサブプロセス)・`ai::keystore`(メモリのみ)は
  今回のスコープ外として据え置き（技術的負債として引き続き記録）

### 動作確認済み

- `pnpm typecheck` / `pnpm lint` / `pnpm test`(31件) / `pnpm build`
  すべて成功
- `cargo build` / `cargo test`(51件) すべて成功
- Xvfb実機起動でアプリが正常に立ち上がり、DB接続・起動時自動バックアップ
  作成を含めエラーなく動作することを確認(新設の`export`モジュール・
  コマンド登録がアプリ初期化を壊していないことの確認)

### 見送った項目

原稿用紙PDFの独立実装、PDFの厳密な行送り(近似のみ)、DOCX/EPUBでの
ルビ・傍点記法の変換、Export履歴・再ダウンロード機能。詳細:
`CLAUDE.md`「未完成機能」/`docs/EXPORT.md`5章。

## [Phase 8] - 2026-09-18

### Added

- DB migration `0007_phase8_safety.sql`: revisions
- バージョン履歴（`src-tauri/src/repositories/revisions_repository.rs`）:
  章/シーン保存のたびに間隔・変化を判定して自動スナップショット
  (間引きあり、最新50件/章・シーン)、いつでも手動スナップショット
  (ラベル付き)、現在の内容との差分表示(Phase6の`DiffView`を再利用)、
  復元(復元前の状態も自動退避)。原稿ワークスペースのサイドパネルに
  「履歴」タブを追加
- Crash Recovery: `app_settings`にセッション中マーカーを持たせ、次回
  起動時に前回の異常終了を検出したら案内バナーを一度だけ表示
  （`tauri::RunEvent::Exit`でクリーン終了時にマーカーを下ろす）
- Auto Backup（`src-tauri/src/db/backup.rs`、`Db::backup_now`/
  `Db::restore_from_backup`）: `VACUUM INTO`によるDB全体の一貫した
  スナップショット。起動時に自動作成(最新10件保持)、作品一覧画面から
  手動バックアップ・バックアップからの復元に対応
- ゴミ箱（`src-tauri/src/repositories/trash_repository.rs`）: 既存の
  Soft Delete(`deleted_at`)を12種のエンティティ(章/シーン/人物/
  世界観項目/場所/用語/メモ/プロットカード/時系列イベント/伏線/TODO/
  パート)にわたって横断的に一覧・復元・完全削除。ワークスペースに
  「ゴミ箱」セクションタブを追加
- `cargo test` 9件追加（計48件: revisions_repository 3件、
  trash_repository 1件、db::backup 3件、db(バックアップ復元系) 2件）

### 動作確認済み

- `pnpm typecheck` / `pnpm lint` / `pnpm test`(31件) / `pnpm build`
  すべて成功
- `cargo build` / `cargo test`(48件) すべて成功
- 既存DB（Phase0〜7のデータ含む）へのマイグレーション0007追加適用・
  既存プロジェクトデータの保持をXvfb実機起動で確認。起動時の自動
  バックアップファイル生成、およびプロセス強制終了によるセッション
  マーカー(異常終了検出)の残存も確認

### 見送った項目

プロジェクト単位のゴミ箱UI、バックアップ世代数の設定変更、Revision
同士の相互比較、Crash Recoveryでの未保存差分マージ(オートセーブ設計
上そもそも概念が存在しない)。詳細は`CLAUDE.md`参照。

## [Phase 7] - 2026-09-18

### Added

- DB migration `0006_phase7_analysis.sql`: ai_analysis_reports
- 高度AI分析6種（`src-tauri/src/ai/analysis.rs`、`commands::analysis::
  run_ai_analysis`/`list_ai_analysis_types`）: 矛盾チェック/Character
  口調チェック/Timeline矛盾チェック/設定矛盾チェック/伏線チェック/
  文体講評。分析種別ごとに必要な情報(作品全文/登場人物/世界観設定/
  時系列/伏線一覧/対象キャラクター設定)を組み立てる専用のContext
  Builder(`context_builder::build_analysis_context`)を追加
- 可読性分析（`src-tauri/src/readability.rs`、非AI・独立モジュール）:
  総文字数/文数/平均文長/最長文/長文数/会話文比率/漢字・ひらがな・
  カタカナ比率/一文あたりの読点数を集計。AI設定なしでも常に利用可能
- フロントエンド`AnalysisPanel`(可読性/AI分析のサブタブ)、
  `ReadabilityPanel`、`AiAnalysisPanel`。ワークスペースに「分析」
  セクションタブを追加
- 分析結果は`ai_analysis_reports`に保存し、分析種別ごとの履歴一覧・
  削除に対応(結果はあくまで参考情報で、原稿等への書き込みは一切発生
  しない読み取り専用機能)
- `cargo test` 9件追加（計39件: ai::analysis 3件、readability 3件、
  context_builder(全文連結・分析コンテキスト) 2件、analysis_repository
  1件）

### 動作確認済み

- `pnpm typecheck` / `pnpm lint` / `pnpm test`(31件) / `pnpm build`
  すべて成功
- `cargo build` / `cargo test`(39件) すべて成功
- 既存DB（Phase0〜6のデータ含む）へのマイグレーション0006追加適用・
  既存プロジェクトデータの保持をXvfb実機起動で確認

### 見送った項目

長編作品での文字数超過対策(要約送信・分割送信は未実装)、分析結果
から本文該当箇所への自動ジャンプ、バックグラウンドでの自動/定期分析。
詳細は`CLAUDE.md`参照。

## [Phase 6] - 2026-09-18

### Added

- AI小説支援機能9種（`src-tauri/src/ai/features.rs`、
  `commands::ai::run_ai_writing_feature`/`list_ai_writing_features`）:
  推敲/続き候補/描写追加/会話改善/Character生成/World生成/Plot生成/
  あらすじ生成/タイトル案。各機能のシステムプロンプトを日本語で定義し、
  本文編集系3機能のみ入力本文を必須とするバリデーションを実装
- フロントエンド `AiWritingAssistPanel`（AIパネルの新規デフォルトタブ
  「執筆支援」）: 機能選択→対象本文プレビュー→追加指示→Context Inspector
  確認→送信→結果表示(差分表示付き)→適用/再生成/コピー/破棄の一連のUI
- 依存ライブラリなしの差分表示（`src/features/ai/diff.ts`: 文字数が
  少ない場合は文字単位、長文はLCSのDP計算量を抑えるため文単位にフォール
  バックするLCSベースの自前実装、`DiffView.tsx`で描画）
- 生成結果パーサー（`src/features/ai/parseGenerated.ts`）: 「ラベル: 値」
  形式のAI出力をCharacter/World/Plotの各作成フォーム入力へ変換
- 「適用」操作は機能ごとに既存の保存/作成コマンドを呼ぶのみ(AI応答受信
  時点では原稿やCharacter等のデータを一切変更しない、`docs/AI.md`4章の
  ルールをバックエンド・フロントエンド双方で維持)
- `cargo test` 3件追加（計30件: プロンプト網羅性/未知機能拒否/本文必須
  判定の各テスト）

### 動作確認済み

- `pnpm typecheck` / `pnpm lint` / `pnpm test`(31件) / `pnpm build`
  すべて成功
- `cargo build` / `cargo test`(30件) すべて成功
- Phase6は新規DBマイグレーションを追加していないため、Xvfb実機起動での
  マイグレーション確認は対象外（既存スキーマのまま）

### 見送った項目

Chapter構成提案機能、プロット生成時のレーン選択（常に先頭レーン固定）、
タイトル案の直接適用（コピーのみ）。詳細は`CLAUDE.md`参照。

## [Phase 5] - 2026-09-18

### Added

- DB migration `0005_phase5_ai.sql`: ai_chat_messages, ai_usage_log
- AI Provider Abstraction（Anthropic/OpenAI/Gemini/OpenAI互換の4種、
  `AiProvider`トレイトで抽象化）
- AI設定画面（Provider/Model/API Key/Base URL/Temperature/Max Output）
- Context Builder（作品概要/現在の章/現在のシーン/直前本文/登場人物/
  世界観設定/未回収の伏線/ユーザーの質問）とContext Inspector（送信前の
  確認・取捨選択UI、バイパス経路なし）
- 作品専用AIチャット(会話履歴の保存・表示・クリア)
- AI使用量表示（本日/今月のリクエスト数・入出力トークン、金額換算なし）
- ワークスペースに「AI」セクションタブを追加
- `cargo test` 5件追加（計27件）

### 動作確認済み

- `pnpm typecheck` / `pnpm lint` / `pnpm test` / `pnpm build` すべて成功
- `cargo build` / `cargo test` すべて成功
- 既存DB（Phase0〜4のデータ含む）へのマイグレーション0005追加適用・
  既存プロジェクトデータの保持をXvfb実機起動で確認

### 既知の制約(重要、詳細は`docs/AI.md` 8.1章)

この開発環境からcrates.ioへの新規クレート取得がネットワーク障害で
できなかったため、(1) HTTP通信は専用クレートではなくOS標準の`curl`
サブプロセス経由、(2) APIキーは`keyring`クレート未導入のためディスク
非永続化(メモリ上のみ、再起動で消える)、という2点の暫定実装にしている。
いずれも安全性(APIキーをargvに載せない、平文でディスクに残さない)は
確保した上での判断。実プロバイダーとの疎通は未検証。

### 見送った項目

Rewrite preview・Diff表示UI(Phase6の推敲機能等とセットで実装予定)。

## [Phase 4] - 2026-09-18

### Added

- DB migration `0004_phase4_story.sql`: plot_lanes, plot_cards,
  timeline_events, foreshadowings, todos, comments
- プロットボード（カンバン方式、レーン初回自動作成4種、カードD&D、
  レーン追加、カード概要編集）
- 時系列（自由記述日付イベントのCRUD）
- 伏線トラッカー（仕様#35の7ステータス管理）
- TODOリスト
- 本文内コメント（textarea選択範囲アンカー方式、引用スナップショット、
  解決済み切替）。原稿ワークスペースのサイドパネルに「検索/コメント」の
  タブ切り替えを追加
- ワークスペースに「ストーリー」セクションタブを追加
  （プロット/時系列/伏線/TODOのサブタブ）
- `cargo test` 5件追加（計22件）

### 動作確認済み

- `pnpm typecheck` / `pnpm lint` / `pnpm test` / `pnpm build` すべて成功
- `cargo build` / `cargo test` すべて成功
- 既存DB（Phase0〜3のデータ含む）へのマイグレーション0004追加適用・
  既存プロジェクトデータの保持をXvfb実機起動で確認

### 見送った項目

プロットボードのレーン並べ替え・リネームUI、時系列イベントのD&D並べ替え、
コメントの返信・スレッド化、コメントの本文内ハイライト表示。

## [Phase 3] - 2026-09-18

### Added

- DB migration `0003_phase3_worldbuilding.sql`: characters,
  character_relations, world_categories, world_entries, glossary_entries,
  locations, notes, tags, entity_tags
- Character CRUD（仕様#22の全項目）、タグ、簡易人物相関一覧
- World Bible（ビルトイン20カテゴリ自動シード）、用語辞典、場所階層のCRUD
- 自由メモ（フォルダ分類、オートセーブ）
- 共通タグ機構
- ワークスペースのセクション切り替えUI（原稿/人物/世界観/資料・メモ）
- `cargo test` 17件追加（計26件）

### 動作確認済み

- `pnpm typecheck` / `pnpm lint` / `pnpm test` / `pnpm build` すべて成功
- `cargo build`、既存DBへのマイグレーション0003追加適用・データ保持を
  Xvfb実機起動で確認

### 見送った項目

人物相関図のNode Graph表示、地図機能、画像添付の実ファイル保存フロー、
Phase3 UIコンポーネント自体のフロントエンドテスト。

## [Phase 2] - 2026-09-18

### Added

- `src/features/manuscript/typeset.ts`: 純粋関数の組版エンジン
  （禁則処理3段階、ルビ、傍点、縦中横、ページ分割、実レイアウト枚数）
- `GenkouYoushiView.tsx`: 原稿用紙ビュー（縦書き/横書き切替、サイズ
  プリセット20×20/20×40/30×40、ズーム、ページ送り、印刷プレビュー）
- 標準エディタから原稿用紙プレビューへのトグルボタン
- Vitestテスト17件追加（計31件）。仕様#95記載の必須テスト文字列
  （「」『』……――！？123ABC漢字（かんじ）)を含む

### 動作確認済み

- `pnpm typecheck` / `pnpm lint` / `pnpm test` / `pnpm build` すべて成功
- `cargo build`、Xvfb実機起動を再確認（Phase2はRust側変更なし）

### 見送った項目

原稿用紙PDF出力、標準エディタ上でのルビ/傍点のボタン挿入UI（現状は記法を
直接入力）、原稿用紙ビュー上での直接編集（IME安全性を優先し表示専用）。

## [Phase 1] - 2026-09-18

### Added

- DB migration `0002_phase1_core.sql`: `projects` / `parts` / `chapters` /
  `scenes` / `documents`
- Project CRUD（Soft Delete対応）、Part/Chapter/Scene階層、Navigatorツリー
- 標準エディタ（textarea、ネイティブUndo/Redo/IME、オートセーブ800ms、
  Ctrl+S、Focus Mode）
- 文字数・400字詰め換算枚数表示（章/シーン単位・作品合計）
- 作品全体検索（章/シーンのタイトル・本文）
- 作品設定フォーム
- Vitestテスト4件追加（計14件）、`cargo test` 6件追加（計9件）

### 動作確認済み

- `pnpm typecheck` / `pnpm lint` / `pnpm test` / `pnpm build` すべて成功
- `cargo check` / `cargo test` / `cargo build` 成功
- 既存(Phase0)のSQLiteファイルに対してマイグレーション0002が正しく追加適用
  され、既存データ（app_settings含む）が保持されることをXvfb実機起動で確認

### 見送った項目

リッチテキスト整形・ルビ・傍点・本文内コメント、Navigatorのドラッグ&ドロップ
並べ替え、実レイアウト原稿枚数（Phase 2で対応）。

## [Phase 0] - 2026-09-18

### Added

- Tauri 2 + React + TypeScript + Vite プロジェクト基盤
- SQLite接続・WALモード・マイグレーションランナー、`app_settings` テーブル
  (`0001_init.sql`)
- レイヤードアーキテクチャの骨格 (`services` / `repositories` / `commands`)
- Zustandによるテーマ (light/dark) 管理 + SQLite永続化
- ErrorBoundary（クラッシュ時の非汎用フォールバックUI）
- ファイル+stdoutへの構造化ログ (`tauri-plugin-log`)
- Vitestユニットテスト10件、`cargo test`ユニットテスト3件
- `docs/{ARCHITECTURE,DATABASE,EDITOR,MANUSCRIPT,AI,ROADMAP}.md`
- `CLAUDE.md`, `README.md`

### 動作確認済み

- `pnpm typecheck` / `pnpm lint` / `pnpm test` / `pnpm build` すべて成功
- `cargo check` / `cargo test` / `cargo build`（Linux向けデバッグビルド）成功
- Xvfb上でのアプリ起動、ウィンドウ生成、SQLiteファイル作成・マイグレーション
  適用・再起動後のデータ永続化を確認

### 既知の制約

- Windows実機でのビルド・起動・IME検証は未実施（開発環境がLinuxサンドボックス
  のため）。詳細は `CLAUDE.md` 参照。
