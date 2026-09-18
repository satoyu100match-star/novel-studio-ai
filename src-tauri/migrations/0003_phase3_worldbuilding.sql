-- Migration 0003 (Phase 3): Characters / World Bible / Glossary / Locations /
-- Notes / Tags.
--
-- Loosely-structured cross-references (e.g. a world entry's related
-- characters) are stored as JSON arrays of ids rather than join tables --
-- an intentional, documented use of the "flexible extension fields may be
-- JSON" allowance in docs/DATABASE.md, not a shortcut around normalization
-- for the entity data itself, which stays in real columns.

CREATE TABLE characters (
    id                  TEXT PRIMARY KEY NOT NULL,
    project_id          TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name                TEXT NOT NULL,
    reading             TEXT,
    aliases             TEXT,
    age                 TEXT,
    gender              TEXT,
    birthday            TEXT,
    height              TEXT,
    occupation          TEXT,
    affiliation         TEXT,
    role                TEXT,        -- 主人公/ヒロイン/敵/味方/脇役/モブ/故人 等、自由入力
    first_appearance    TEXT,
    hair                TEXT,
    eyes                TEXT,
    build               TEXT,
    clothing            TEXT,
    features             TEXT,
    scars               TEXT,
    equipment           TEXT,
    personality         TEXT,
    strengths           TEXT,
    weaknesses          TEXT,
    beliefs             TEXT,
    desires             TEXT,
    fears               TEXT,
    secret              TEXT,
    trauma              TEXT,
    first_person        TEXT,
    second_person       TEXT,
    speech_suffix       TEXT,
    catchphrase         TEXT,
    honorific_level     TEXT,
    calls_protagonist   TEXT,
    calls_others        TEXT,
    goal                TEXT,
    motivation          TEXT,
    past                TEXT,
    initial_state       TEXT,
    middle_state        TEXT,
    final_state         TEXT,
    character_arc       TEXT,
    memo                TEXT,
    reference_image_path TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    deleted_at          TEXT
);
CREATE INDEX idx_characters_project ON characters(project_id);

CREATE TABLE character_relations (
    id                  TEXT PRIMARY KEY NOT NULL,
    project_id          TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    from_character_id   TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    to_character_id     TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    label               TEXT NOT NULL, -- 親子/兄弟/恋人/friend/... 自由入力
    color               TEXT,
    direction           TEXT NOT NULL DEFAULT 'a_to_b' CHECK (direction IN ('a_to_b', 'bidirectional')),
    detail              TEXT,
    start_at            TEXT,
    end_at              TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    deleted_at          TEXT
);
CREATE INDEX idx_character_relations_project ON character_relations(project_id);

CREATE TABLE world_categories (
    id          TEXT PRIMARY KEY NOT NULL,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    is_builtin  INTEGER NOT NULL DEFAULT 0,
    order_index INTEGER NOT NULL,
    created_at  TEXT NOT NULL,
    deleted_at  TEXT
);
CREATE INDEX idx_world_categories_project ON world_categories(project_id);

CREATE TABLE world_entries (
    id                  TEXT PRIMARY KEY NOT NULL,
    project_id          TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    category_id         TEXT REFERENCES world_categories(id) ON DELETE SET NULL,
    name                TEXT NOT NULL,
    reading             TEXT,
    summary             TEXT,
    detail              TEXT,
    related_character_ids TEXT NOT NULL DEFAULT '[]',
    related_location_ids  TEXT NOT NULL DEFAULT '[]',
    related_entry_ids     TEXT NOT NULL DEFAULT '[]',
    image_path          TEXT,
    memo                TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    deleted_at          TEXT
);
CREATE INDEX idx_world_entries_project ON world_entries(project_id);
CREATE INDEX idx_world_entries_category ON world_entries(category_id);

CREATE TABLE glossary_entries (
    id          TEXT PRIMARY KEY NOT NULL,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    term        TEXT NOT NULL,
    reading     TEXT,
    definition  TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);
CREATE INDEX idx_glossary_project ON glossary_entries(project_id);

CREATE TABLE locations (
    id                  TEXT PRIMARY KEY NOT NULL,
    project_id          TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    parent_location_id  TEXT REFERENCES locations(id) ON DELETE SET NULL,
    name                TEXT NOT NULL,
    description         TEXT,
    coordinates         TEXT,
    related_character_ids TEXT NOT NULL DEFAULT '[]',
    image_path          TEXT,
    memo                TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL,
    deleted_at          TEXT
);
CREATE INDEX idx_locations_project ON locations(project_id);
CREATE INDEX idx_locations_parent ON locations(parent_location_id);

CREATE TABLE notes (
    id          TEXT PRIMARY KEY NOT NULL,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    folder      TEXT NOT NULL DEFAULT '',
    title       TEXT NOT NULL,
    body        TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);
CREATE INDEX idx_notes_project ON notes(project_id);

CREATE TABLE tags (
    id          TEXT PRIMARY KEY NOT NULL,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    color       TEXT,
    created_at  TEXT NOT NULL,
    UNIQUE (project_id, name)
);

CREATE TABLE entity_tags (
    id          TEXT PRIMARY KEY NOT NULL,
    tag_id      TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL, -- 'character' | 'world_entry' | 'location' | 'note' | ...
    entity_id   TEXT NOT NULL,
    UNIQUE (tag_id, entity_type, entity_id)
);
CREATE INDEX idx_entity_tags_entity ON entity_tags(entity_type, entity_id);
