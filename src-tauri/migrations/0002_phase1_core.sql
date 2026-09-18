-- Migration 0002 (Phase 1): projects / parts / chapters / scenes / documents
--
-- documents holds the actual manuscript body text, decoupled from
-- chapters/scenes so that Phase 2 (manuscript view) and Phase 9 (export)
-- can typeset the same underlying text differently, and so Phase 8
-- (revisions) has a stable id to snapshot. See docs/MANUSCRIPT.md.

CREATE TABLE projects (
    id              TEXT PRIMARY KEY NOT NULL,
    title           TEXT NOT NULL,
    subtitle        TEXT,
    author_name     TEXT,
    pen_name        TEXT,
    genre           TEXT,
    target_audience TEXT,
    target_length   INTEGER,
    deadline        TEXT,
    synopsis        TEXT,
    theme           TEXT,
    concept         TEXT,
    style_memo      TEXT,
    pov_policy      TEXT,
    tense           TEXT,
    ai_policy       TEXT,
    is_sample       INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT
);

CREATE TABLE parts (
    id          TEXT PRIMARY KEY NOT NULL,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    order_index INTEGER NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);
CREATE INDEX idx_parts_project ON parts(project_id);

CREATE TABLE chapters (
    id          TEXT PRIMARY KEY NOT NULL,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    part_id     TEXT REFERENCES parts(id) ON DELETE SET NULL,
    title       TEXT NOT NULL,
    subtitle    TEXT,
    synopsis    TEXT,
    memo        TEXT,
    pov         TEXT,
    status      TEXT NOT NULL DEFAULT 'not_started',
    start_at    TEXT,
    end_at      TEXT,
    location    TEXT,
    order_index INTEGER NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);
CREATE INDEX idx_chapters_project ON chapters(project_id);
CREATE INDEX idx_chapters_part ON chapters(part_id);

CREATE TABLE scenes (
    id              TEXT PRIMARY KEY NOT NULL,
    chapter_id      TEXT NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    summary         TEXT,
    pov_character   TEXT,
    location        TEXT,
    event_date      TEXT,
    start_time      TEXT,
    end_time        TEXT,
    purpose         TEXT,
    conflict        TEXT,
    result          TEXT,
    emotion         TEXT,
    memo            TEXT,
    order_index     INTEGER NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    deleted_at      TEXT
);
CREATE INDEX idx_scenes_chapter ON scenes(chapter_id);

-- owner_type is 'chapter' | 'scene'. Exactly one document per owner.
CREATE TABLE documents (
    id          TEXT PRIMARY KEY NOT NULL,
    owner_type  TEXT NOT NULL CHECK (owner_type IN ('chapter', 'scene')),
    owner_id    TEXT NOT NULL,
    body        TEXT NOT NULL DEFAULT '',
    char_count  INTEGER NOT NULL DEFAULT 0,
    updated_at  TEXT NOT NULL,
    UNIQUE (owner_type, owner_id)
);
CREATE INDEX idx_documents_owner ON documents(owner_type, owner_id);
