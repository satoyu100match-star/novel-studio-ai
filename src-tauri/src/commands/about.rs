//! Phase 10: 「このアプリについて」画面向けのコマンド群。CHANGELOG.md /
//! THIRD_PARTY_NOTICES.md / PRIVACY.mdはリポジトリ直下の単一の真実源
//! (人間が読む用のMarkdownそのもの)をビルド時に`include_str!`で埋め込み、
//! アプリ内表示用に文字列を複製・同期させる必要をなくしている。

use crate::error::AppResult;
use crate::export::ExportOutput;
use tauri::{AppHandle, Manager};

const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");
const THIRD_PARTY_NOTICES: &str = include_str!("../../../THIRD_PARTY_NOTICES.md");
const PRIVACY_NOTICE: &str = include_str!("../../../PRIVACY.md");

#[tauri::command]
pub fn get_changelog() -> &'static str {
    CHANGELOG
}

#[tauri::command]
pub fn get_third_party_notices() -> &'static str {
    THIRD_PARTY_NOTICES
}

#[tauri::command]
pub fn get_privacy_notice() -> &'static str {
    PRIVACY_NOTICE
}

/// 診断ログの書き出し。アプリは一切のテレメトリ(自動送信)を行わない方針
/// (`PRIVACY.md`参照)のため、「エラー報告」機能は自動送信ではなく、
/// ユーザーが自分の意思でログをZIPとして書き出し、任意の手段(メール等)で
/// 共有できるようにする形にしている。ログには本文やAPIキーは出力していない
/// (CLAUDE.md「禁止事項」)。
#[tauri::command]
pub fn export_diagnostics(app: AppHandle) -> AppResult<ExportOutput> {
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|e| crate::error::AppError::Other(format!("ログフォルダの取得に失敗しました: {e}")))?;

    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    if log_dir.is_dir() {
        for entry in std::fs::read_dir(&log_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let bytes = std::fs::read(&path)?;
                    entries.push((name.to_string(), bytes));
                }
            }
        }
    }

    if entries.is_empty() {
        return Err(crate::error::AppError::Other("書き出せるログファイルが見つかりませんでした。".into()));
    }

    let zip_entries: Vec<(&str, Vec<u8>)> = entries.iter().map(|(name, bytes)| (name.as_str(), bytes.clone())).collect();
    let bytes = crate::export::build_zip(zip_entries)?;

    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    Ok(ExportOutput {
        filename: format!("novel-studio-ai-diagnostics-{timestamp}.zip"),
        mime_type: "application/zip".to_string(),
        bytes,
    })
}
