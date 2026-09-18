-- Phase 5: AI基盤 (Provider abstraction / Context Builder / Chat / Usage)
-- 仕様 #40-#46 相当。APIキー自体はここには保存しない
-- (OSの資格情報ストアのみで管理する方針。docs/AI.md 3章)。

-- 作品ごとのAIチャット(仕様#44)。MVPとしてプロジェクトにつき1スレッド。
CREATE TABLE ai_chat_messages (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
    content TEXT NOT NULL,
    -- 送信時にContext Inspectorでユーザーが確認・選択したコンテキストの
    -- 要約(ブロックのラベル一覧)。何を送ったかを後から追跡できるようにする。
    context_summary TEXT,
    provider TEXT,
    model TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_ai_chat_messages_project ON ai_chat_messages(project_id, created_at);

-- AI使用量ログ(仕様#46)。日次・月次の集計に使う。コスト計算は行わない
-- (プロバイダーによって単価体系が異なり、不正確な金額を断定的に出さない
-- 方針のため -- docs/AI.md 6章)。
CREATE TABLE ai_usage_log (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id),
    feature TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    input_tokens INTEGER,
    output_tokens INTEGER,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_ai_usage_log_created ON ai_usage_log(created_at);
