-- Migration 0001: initial schema (Phase 0)
--
-- Phase 0 only needs to prove that the app can read and write local SQLite
-- data. The full domain schema (projects, chapters, scenes, characters,
-- world entries, plot cards, timeline events, foreshadowing, etc. -- see
-- docs/DATABASE.md) is introduced incrementally in later phases, each as
-- its own numbered migration file. Do not add unused tables ahead of the
-- phase that needs them.

-- Generic app-level key/value settings (theme, last opened project, window
-- state, etc.). Not for per-project data -- that gets its own normalized
-- tables from Phase 1 onward.
CREATE TABLE IF NOT EXISTS app_settings (
    key         TEXT PRIMARY KEY NOT NULL,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
