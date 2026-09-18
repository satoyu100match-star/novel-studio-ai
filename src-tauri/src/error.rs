//! Application-wide error type.
//!
//! Tauri commands must return errors that implement `serde::Serialize` so
//! they can cross the IPC boundary to the frontend. We keep a single
//! `AppError` type for the whole backend and serialize it as a plain
//! `{ "message": "..." }` object -- never leaking raw file paths or
//! internal details that aren't useful to the user, and never logging or
//! surfacing secrets (API keys) through this path (see docs/AI.md).

use serde::Serialize;
use std::fmt;

#[derive(Debug)]
#[allow(dead_code)] // NotFound / Other are used from later phases onward.
pub enum AppError {
    Database(String),
    Io(String),
    NotFound(String),
    Migration(String),
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(msg) => write!(f, "データベースエラー: {msg}"),
            AppError::Io(msg) => write!(f, "ファイル入出力エラー: {msg}"),
            AppError::NotFound(msg) => write!(f, "見つかりません: {msg}"),
            AppError::Migration(msg) => write!(f, "マイグレーションエラー: {msg}"),
            AppError::Other(msg) => write!(f, "エラー: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
