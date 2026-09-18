# ARCHITECTURE.md — Novel Studio AI

このドキュメントはアプリ全体の設計方針を記録する。実装より先に読むべきドキュメント。

## 1. 全体レイヤー構成

通常機能（原稿・Character・世界観など）:

```
UI (React components / features/*)
  ↓
Application Services (src/services/*)  … ユースケース単位。Tauri commandを直接呼ぶのはここだけ。
  ↓
Domain (src/domain/*)                  … エンティティ・値オブジェクト・純粋なビジネスルール
  ↓
Repositories (src-tauri/src/repositories/*) … 生SQLを書けるのはここだけ
  ↓
SQLite / File System
```

AI関連は独立レイヤー（通常レイヤーに依存はするが、AIが無効でも通常機能は完全動作すること = 最重要方針#8）:

```
UI (features/ai/*)
  ↓
AI Feature (src/features/ai/*)         … 「推敲」「続きの案」等ユースケース単位
  ↓
Context Builder (src/services/ai/contextBuilder.ts) … 送信情報を明示的に選別
  ↓
AI Gateway (src-tauri 側 provider abstraction)
  ↓
Provider (Anthropic / OpenAI / Gemini / OpenAI-Compatible / 将来Local AI)
```

Providerを切り替えてもUI・Application Services・Domainは変更不要にする。Provider実装は
`AiProvider` トレイト（Rust）1本に対する差し替え可能な実装として設計する（Phase 5で導入）。

## 2. フロントエンド ⇔ Rustバックエンドの境界

- `src/types/tauriCommands.ts` が `invoke()` を呼ぶ唯一の場所。返り値は必ずZodで検証する。
- Featureやコンポーネントから直接 `@tauri-apps/api` を import しない。必ず `src/services/*`
  (Application Services) を経由する。
- Rust側では `src-tauri/src/commands/*` がIPCの薄い境界。バリデーションと
  `repositories`/`services` 呼び出しのみ行い、SQLやビジネスロジックを書かない。
- `src-tauri/src/repositories/*` が生SQLを書ける唯一の場所（テーブルごとに1ファイル）。

## 3. フォルダ構成

```
novel-studio-ai/
├─ src/
│  ├─ app/            … エントリポイント、ルーター、テーマ適用、ErrorBoundary
│  ├─ components/      … 汎用UIコンポーネント（特定featureに依存しない）
│  ├─ features/         … 機能単位（editor, manuscript, characters, world, plot, timeline,
│  │                       foreshadowing, ai, analysis, export, settings, ...）
│  ├─ domain/           … エンティティ・値オブジェクト・純粋ロジック（フレームワーク非依存）
│  ├─ services/         … Application Services（ユースケース）
│  ├─ repositories/     … フロント側の型定義/クエリキー等（実SQLはRust側）
│  ├─ stores/           … Zustandストア（UI状態。全文データを丸ごと持たない）
│  ├─ hooks/            … 汎用React hooks
│  ├─ utils/             … 汎用ユーティリティ（logger等）
│  ├─ styles/            … デザイントークン(CSS変数)・グローバルCSS
│  └─ types/             … 型定義、Tauri commandラッパー
├─ src-tauri/
│  ├─ src/
│  │  ├─ commands/       … IPCコマンド（薄い層）
│  │  ├─ repositories/   … 生SQLを書ける唯一の場所
│  │  ├─ db/              … 接続管理・マイグレーションランナー
│  │  └─ error.rs         … アプリ共通エラー型
│  ├─ capabilities/       … Tauri v2 permission定義
│  └─ migrations/         … 番号付きSQLマイグレーション（追記専用）
├─ tests/                 … クロスカット的なテスト設定・統合テスト
├─ docs/                  … このドキュメント群
└─ scripts/               … 開発補助スクリプト
```

巨大な1ファイル（何千行のApp.tsxやlib.rs）は禁止。featureごと、テーブルごと、
ユースケースごとにファイルを分割する。

## 4. 状態管理方針

- Zustandは「UI状態」と「現在表示中データのキャッシュ」のためのもの。
- **作品全文を常時React State / Zustandへ載せない**（最重要方針#14, 仕様#89, #90）。
  100万文字・100章超でも快適に動作させるため、エディタは「今開いている章/シーン」の
  本文のみをメモリに持ち、他はSQLiteから必要な範囲だけ取得する。
- 一文字入力ごとに作品全体を再計算しない。文字数カウント・原稿枚数換算・AI解析などは
  すべてdebounce（500〜1500ms目安）またはidle時に実行する。

## 5. アプリ名のハードコード禁止

アプリ名は以下の1箇所ずつが真実源（Single Source of Truth）:

- Rust/バンドル設定: `src-tauri/tauri.conf.json` の `productName`
- フロントエンドのフォールバック文字列: `src/app/config.ts` の `APP_NAME_FALLBACK`
  （Tauri起動前の初期描画や、Tauri外プレビュー時のみ使用。通常は起動時に
  `app_info` コマンド経由で `tauri.conf.json` の値を取得して表示する）

新しいUI文字列やコンポーネントにアプリ名の文字列リテラルを追加しない。

## 6. エラーハンドリング方針

「何か失敗しました」だけの表示は禁止（仕様#91）。エラーは最低限:

- 何が起きたか（可能な範囲で原因）
- データは失われていないか（原稿は一時領域に残っているか）
- 次にできること（再試行・別名保存 等）

を含める。`src-tauri/src/error.rs` の `AppError` が全コマンド共通のエラー型。
バックエンドは詳細ログをファイル (`tauri-plugin-log`) に残しつつ、フロントには
ユーザーに意味のあるメッセージだけを返す。ログにAPIキー・原稿全文・個人情報を
大量に書き込まない（仕様#92）。

## 7. テスト方針

- Rust: `cargo test`（`src-tauri/src/**` 内 `#[cfg(test)] mod tests`）。
  リポジトリ・マイグレーションランナーなど、SQL/ロジック層を中心に単体テスト。
- フロントエンド: Vitest + Testing Library。Zustandストアのロジック、
  ErrorBoundaryのようなクリティカルUI、Application Servicesのフォールバック
  挙動（Tauri未接続時にthrowしないこと）を検証する。
- 将来（Phase 5以降）AI機能はMock Providerでテストし、実APIを消費しない
  （仕様#96）。
- E2E（Playwright等）はPhase 1以降、実際に操作できる機能が揃ってから追加する。

## 8. 判断に迷ったときの優先順位（最重要方針#104を踏襲）

```
データ安全性 > 執筆体験 > 安定性 > 操作性 > パフォーマンス > AI機能 > 見た目
```
