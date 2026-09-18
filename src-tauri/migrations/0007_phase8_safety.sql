-- Phase 8: Revision履歴 / Crash Recovery / Auto Backup / ゴミ箱。
-- データ安全性を最優先する方針(CLAUDE.md優先順位#1)に対応するPhase。
--
-- Auto Backup(DBファイル全体のスナップショット)とゴミ箱(既存の
-- deleted_at列を使ったSoft Delete済みレコードの一覧)はテーブル追加を
-- 必要としない。このマイグレーションで追加するのはRevision履歴のみ。

-- 章/シーン本文のバージョン履歴。「上書き保存ですべて消える」ことへの
-- 不安に対応する(仕様の主要不安要素)。全編集を残すのではなく、
-- 一定間隔(自動)または明示操作(手動)でのスナップショット方式にして、
-- テーブルが際限なく肥大化しないようにする(`revisions_repository`参照)。
CREATE TABLE revisions (
    id TEXT PRIMARY KEY,
    owner_type TEXT NOT NULL CHECK(owner_type IN ('chapter', 'scene')),
    owner_id TEXT NOT NULL,
    body TEXT NOT NULL,
    char_count INTEGER NOT NULL,
    trigger TEXT NOT NULL CHECK(trigger IN ('auto', 'manual')),
    label TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_revisions_owner ON revisions(owner_type, owner_id, created_at);
