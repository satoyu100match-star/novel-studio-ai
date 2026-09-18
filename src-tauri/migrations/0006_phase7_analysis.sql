-- Phase 7: 高度AI分析 (矛盾チェック/Character口調/Timeline矛盾/設定矛盾/
-- 未回収伏線/文体分析)。仕様の分析系機能に相当。
--
-- 可読性分析(文字数・平均文長・会話文比率等)はAI不要のローカル計算のため
-- 保存対象にしない(いつでも即時再計算できる)。AIによる分析結果のみ、
-- 「何を・いつ・どのプロバイダーで分析したか」を後から見返せるように
-- 保存する(ai_chat_messagesと同様、送信内容の要約も残す)。

CREATE TABLE ai_analysis_reports (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    analysis_type TEXT NOT NULL CHECK(analysis_type IN (
        'contradiction', 'character_tone', 'timeline', 'setting', 'foreshadowing', 'style'
    )),
    -- character_tone分析の対象キャラクター名など、レポート一覧に表示する
    -- ための短い補足(必須ではない)。
    target_summary TEXT,
    -- 送信時にContext Inspectorで選択されたブロックのラベル一覧。
    context_summary TEXT,
    result TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_ai_analysis_reports_project ON ai_analysis_reports(project_id, analysis_type, created_at);
