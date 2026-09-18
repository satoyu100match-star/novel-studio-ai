-- Phase 4: ストーリー管理 (Plot Board / Timeline / Foreshadowing / TODO / Comments)
-- 仕様 #30-#36 相当。既存テーブルへの破壊的変更は行わない(追記専用の方針)。

-- プロットボード: レーン(構想/執筆予定/執筆中/完了 等)とその中のカード。
-- カードの「状態」はレーンへの所属そのもので表現する(別途statusカラムは
-- 持たない -- カンバン方式)。
CREATE TABLE plot_lanes (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    name TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE plot_cards (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    lane_id TEXT NOT NULL REFERENCES plot_lanes(id),
    title TEXT NOT NULL,
    summary TEXT,
    chapter_id TEXT REFERENCES chapters(id),
    color TEXT,
    order_index INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

-- 時系列(作中の出来事を実世界時間とは独立に並べる年表)
CREATE TABLE timeline_events (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    title TEXT NOT NULL,
    event_date TEXT,
    description TEXT,
    chapter_id TEXT REFERENCES chapters(id),
    scene_id TEXT REFERENCES scenes(id),
    order_index INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

-- 伏線トラッカー(仕様#35: 構想/設置予定/設置済/ヒント提示済/回収予定/回収済/破棄)
CREATE TABLE foreshadowings (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    title TEXT NOT NULL,
    detail TEXT,
    status TEXT NOT NULL DEFAULT 'idea',
    planted_chapter_id TEXT REFERENCES chapters(id),
    payoff_chapter_id TEXT REFERENCES chapters(id),
    memo TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE todos (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    title TEXT NOT NULL,
    done INTEGER NOT NULL DEFAULT 0,
    due_date TEXT,
    memo TEXT,
    order_index INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

-- 本文内コメント。IME安全性優先のためcontentEditable化はせず、textareaの
-- selectionStart/Endで取得した位置と引用テキストのスナップショットを
-- 保存する方式(docs/EDITOR.md方針)。本文編集で前後の文字が増減しても
-- quoteテキストで位置のズレをユーザーが目視確認できるようにする。
CREATE TABLE comments (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    owner_type TEXT NOT NULL CHECK(owner_type IN ('chapter', 'scene')),
    owner_id TEXT NOT NULL,
    anchor_start INTEGER,
    anchor_end INTEGER,
    quote TEXT,
    body TEXT NOT NULL,
    resolved INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE INDEX idx_plot_cards_project ON plot_cards(project_id);
CREATE INDEX idx_plot_cards_lane ON plot_cards(lane_id);
CREATE INDEX idx_timeline_events_project ON timeline_events(project_id);
CREATE INDEX idx_foreshadowings_project ON foreshadowings(project_id);
CREATE INDEX idx_todos_project ON todos(project_id);
CREATE INDEX idx_comments_owner ON comments(owner_type, owner_id);
