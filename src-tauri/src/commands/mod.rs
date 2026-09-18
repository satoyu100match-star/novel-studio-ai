//! Tauri commands: the thin IPC boundary between the frontend and the
//! Rust backend. Commands validate input, call into repositories /
//! application services, and translate errors into `AppError`. No SQL and
//! no business logic belongs here.

pub mod about;
pub mod ai;
pub mod analysis;
pub mod app_info;
pub mod backup;
pub mod chapters;
pub mod characters;
pub mod comments;
pub mod documents;
pub mod export;
pub mod foreshadowing;
pub mod glossary;
pub mod locations;
pub mod notes;
pub mod plot;
pub mod projects;
pub mod revisions;
pub mod scenes;
pub mod search;
pub mod settings;
pub mod tags;
pub mod timeline;
pub mod todos;
pub mod trash;
pub mod world;
