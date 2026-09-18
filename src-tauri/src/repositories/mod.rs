//! Repositories: the only layer allowed to write raw SQL. Application
//! services and Tauri commands call into repositories; they never touch
//! `rusqlite` directly. This keeps SQL in one place per table family so a
//! future schema change (via migration) has one call site to update.

pub mod ai_chat_repository;
pub mod ai_usage_repository;
pub mod analysis_repository;
pub mod chapters_repository;
pub mod character_relations_repository;
pub mod characters_repository;
pub mod comments_repository;
pub mod documents_repository;
pub mod foreshadowing_repository;
pub mod glossary_repository;
pub mod locations_repository;
pub mod notes_repository;
pub mod parts_repository;
pub mod plot_repository;
pub mod projects_repository;
pub mod revisions_repository;
pub mod scenes_repository;
pub mod search_repository;
pub mod settings_repository;
pub mod tags_repository;
pub mod timeline_repository;
pub mod todos_repository;
pub mod trash_repository;
pub mod world_repository;
