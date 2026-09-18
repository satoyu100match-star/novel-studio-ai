# DATABASE.md — Novel Studio AI

## 1. 基本方針

- SQLite1ファイル、アプリのローカルデータディレクトリに保存する
  （`tauri::path::app_data_dir()`。Windowsでは `%APPDATA%\com.novelstudioai.app\`
  相当、Linuxでは `~/.local/share/com.novelstudioai.app/` 相当）。
- 何でもJSON1列に押し込まない。正規化されたテーブルを基本とし、
  柔軟な拡張が必要な項目（キャラクターの自由メモ欄など）のみJSON列を許容する。
- 内部IDは UUID (v4) を使用する。タイトル等の人間可読文字列をIDにしない。
- 重要データは即時完全削除ではなく Soft Delete（`deleted_at` 列 + ゴミ箱UI）を
  基本方針とする。具体的なテーブルはPhase 4以降、該当機能と一緒に導入する。
- スキーマ変更は必ずマイグレーションを使う。既存ユーザーの作品データを破壊しない。

## 2. マイグレーションの運用ルール

- 実体: `src-tauri/migrations/NNNN_name.sql`（4桁連番、`include_str!` でバイナリに
  埋め込み）。
- 適用状況は `schema_migrations (version INTEGER PK, name TEXT, applied_at TEXT)`
  で管理する。起動時に `src-tauri/src/db/migrations.rs` のランナーが未適用分のみ
  トランザクション内で実行する。
- **一度リリースしたマイグレーションファイルは絶対に書き換えない。** 変更が
  必要になったら新しい番号のファイルを追加する。
- 1マイグレーション = 1トランザクション。失敗したら起動を止める（中途半端な
  状態のDBのままユーザーに使わせない）。

## 3. Phase 0 時点のスキーマ

`0001_init.sql`:

```sql
CREATE TABLE app_settings (
    key         TEXT PRIMARY KEY NOT NULL,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
```

アプリ全体設定（テーマ等）のみ。作品データ用テーブルはまだ存在しない
（Phase 1で `projects` 等を追加する）。

## 4. 将来の全体スキーマ計画（仕様書 セクション76 準拠、導入Phaseの目安）

実装前にこの一覧を確定させすぎない（要件は各Phase着手時に見直す）。あくまで
「後から大規模作り直しにならないための」設計メモ。

| テーブル | 概要 | 想定導入Phase |
|---|---|---|
| projects | 作品（タイトル・作者・ジャンル等） | 1 |
| project_settings | 作品単位の詳細設定（POV方針・文体メモ等） | 1 |
| parts | 部（任意） | 1 |
| chapters | 章 | 1 |
| scenes | シーン | 1 |
| documents | 本文データ本体（章/シーンに紐づく通常テキスト） | 1 |
| document_revisions | リビジョン（スナップショット） | 8 |
| characters | キャラクター | 3 |
| character_relations | 人物相関 | 3 |
| world_categories | 世界観カテゴリ（ユーザー拡張可） | 3 |
| world_entries | 世界観項目 | 3 |
| locations | 場所（階層構造） | 3 |
| plot_cards | プロットカード | 4 |
| plot_lanes | ストーリーライン（複数プロットレーン） | 4 |
| timeline_events | タイムラインイベント | 4 |
| foreshadowings | 伏線 | 4 |
| glossary_entries | 用語辞典 | 3 |
| notes | 自由メモ | 3 |
| attachments | 資料・添付ファイル参照 | 3 |
| tags | 共通タグ定義 | 3 |
| entity_tags | タグの多対多紐付け（対象種別+対象ID） | 3 |
| comments | 本文コメント | 4 |
| todos | 執筆TODO | 4 |
| writing_sessions | 執筆セッション（タイマー） | 8 |
| writing_daily_stats | 日別執筆統計 | 8 |
| writing_goals | 執筆目標 | 8 |
| ai_conversations | AIチャットスレッド | 5 |
| ai_messages | AIチャットメッセージ | 5 |
| ai_usage | AI使用量（リクエスト数・推定トークン） | 5 |
| app_settings | アプリ全体設定 | 0 (実装済み) |

`documents` は本文を「通常の文章」として保持し、原稿用紙・縦書き・横書き・PDF等は
すべてこの本文データからビューとして生成する（仕様#18 = 本文とレイアウトの分離）。
本文を400個のinputやセル単位のレコードで保存することはしない。

## 5. 命名・型の規約

- テーブル名は複数形スネークケース。
- 主キーは `id TEXT PRIMARY KEY`（UUID文字列）。
- タイムスタンプは `TEXT`（RFC3339, UTC, 例 `2026-09-18T01:42:17Z`）で統一。
- 外部キーは `<参照先単数形>_id`。`PRAGMA foreign_keys = ON` を接続時に必ず設定する
  （`src-tauri/src/db/mod.rs` で設定済み）。
- 削除予定フラグは `deleted_at TEXT NULL`（NULL = 生存、値あり = ゴミ箱行き）。

## 6. パフォーマンス

- WAL (`journal_mode = WAL`) を起動時に設定済み。書き込み中でも読み取りをブロック
  しない。自動保存とAI機能が同時に動く場面（仕様#8, #90）を想定した設定。
- 長編作品（100万文字/100章/数百シーン/数百設定項目、仕様#89）でも一覧系クエリが
  重くならないよう、章一覧・シーン一覧等は本文列を含めないSELECTを基本にする
  （本文が必要な画面でのみ `documents` を個別取得）。
