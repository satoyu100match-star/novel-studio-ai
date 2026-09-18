//! Shared data-transfer shapes for Phase 1 domain entities.
//!
//! These mirror the `projects` / `parts` / `chapters` / `scenes` /
//! `documents` tables from `0002_phase1_core.sql`. Kept in one file
//! because they are plain shapes with no behavior; real business logic
//! belongs in `services`/`repositories`, not here.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub author_name: Option<String>,
    pub pen_name: Option<String>,
    pub genre: Option<String>,
    pub target_audience: Option<String>,
    pub target_length: Option<i64>,
    pub deadline: Option<String>,
    pub synopsis: Option<String>,
    pub theme: Option<String>,
    pub concept: Option<String>,
    pub style_memo: Option<String>,
    pub pov_policy: Option<String>,
    pub tense: Option<String>,
    pub ai_policy: Option<String>,
    pub is_sample: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ProjectInput {
    pub title: String,
    pub subtitle: Option<String>,
    pub author_name: Option<String>,
    pub pen_name: Option<String>,
    pub genre: Option<String>,
    pub target_audience: Option<String>,
    pub target_length: Option<i64>,
    pub deadline: Option<String>,
    pub synopsis: Option<String>,
    pub theme: Option<String>,
    pub concept: Option<String>,
    pub style_memo: Option<String>,
    pub pov_policy: Option<String>,
    pub tense: Option<String>,
    pub ai_policy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Part {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub order_index: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub id: String,
    pub project_id: String,
    pub part_id: Option<String>,
    pub title: String,
    pub subtitle: Option<String>,
    pub synopsis: Option<String>,
    pub memo: Option<String>,
    pub pov: Option<String>,
    pub status: String,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub location: Option<String>,
    pub order_index: i64,
    pub char_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scene {
    pub id: String,
    pub chapter_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub pov_character: Option<String>,
    pub location: Option<String>,
    pub event_date: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub purpose: Option<String>,
    pub conflict: Option<String>,
    pub result: Option<String>,
    pub emotion: Option<String>,
    pub memo: Option<String>,
    pub order_index: i64,
    pub char_count: i64,
}

/// Valid chapter workflow statuses (spec #13). Kept as plain strings in
/// the DB (simpler migrations across renames) but validated at the
/// command boundary against this list.
pub const CHAPTER_STATUSES: &[&str] = &[
    "not_started",  // 未着手
    "planning",     // 構想中
    "writing",      // 執筆中
    "first_draft",  // 初稿完成
    "revising",     // 推敲中
    "done",         // 完成
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub id: String,
    pub owner_type: String,
    pub owner_id: String,
    pub body: String,
    pub char_count: i64,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SearchHit {
    pub owner_type: String,
    pub owner_id: String,
    pub title: String,
    pub snippet: String,
}

// ---------------------------------------------------------------------------
// Phase 3: Characters / World Bible / Glossary / Locations / Notes / Tags
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Character {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub reading: Option<String>,
    pub aliases: Option<String>,
    pub age: Option<String>,
    pub gender: Option<String>,
    pub birthday: Option<String>,
    pub height: Option<String>,
    pub occupation: Option<String>,
    pub affiliation: Option<String>,
    pub role: Option<String>,
    pub first_appearance: Option<String>,
    pub hair: Option<String>,
    pub eyes: Option<String>,
    pub build: Option<String>,
    pub clothing: Option<String>,
    pub features: Option<String>,
    pub scars: Option<String>,
    pub equipment: Option<String>,
    pub personality: Option<String>,
    pub strengths: Option<String>,
    pub weaknesses: Option<String>,
    pub beliefs: Option<String>,
    pub desires: Option<String>,
    pub fears: Option<String>,
    pub secret: Option<String>,
    pub trauma: Option<String>,
    pub first_person: Option<String>,
    pub second_person: Option<String>,
    pub speech_suffix: Option<String>,
    pub catchphrase: Option<String>,
    pub honorific_level: Option<String>,
    pub calls_protagonist: Option<String>,
    pub calls_others: Option<String>,
    pub goal: Option<String>,
    pub motivation: Option<String>,
    pub past: Option<String>,
    pub initial_state: Option<String>,
    pub middle_state: Option<String>,
    pub final_state: Option<String>,
    pub character_arc: Option<String>,
    pub memo: Option<String>,
    pub reference_image_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharacterRelation {
    pub id: String,
    pub project_id: String,
    pub from_character_id: String,
    pub to_character_id: String,
    pub label: String,
    pub color: Option<String>,
    pub direction: String,
    pub detail: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorldCategory {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub is_builtin: bool,
    pub order_index: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WorldEntry {
    pub id: String,
    pub project_id: String,
    pub category_id: Option<String>,
    pub name: String,
    pub reading: Option<String>,
    pub summary: Option<String>,
    pub detail: Option<String>,
    pub related_character_ids: Vec<String>,
    pub related_location_ids: Vec<String>,
    pub related_entry_ids: Vec<String>,
    pub image_path: Option<String>,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GlossaryEntry {
    pub id: String,
    pub project_id: String,
    pub term: String,
    pub reading: Option<String>,
    pub definition: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Location {
    pub id: String,
    pub project_id: String,
    pub parent_location_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub coordinates: Option<String>,
    pub related_character_ids: Vec<String>,
    pub image_path: Option<String>,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Note {
    pub id: String,
    pub project_id: String,
    pub folder: String,
    pub title: String,
    pub body: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub color: Option<String>,
}

// ---------------------------------------------------------------------------
// Phase 4: Plot Board / Timeline / Foreshadowing / TODO / Comments
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlotLane {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub order_index: i64,
}

/// Default lanes seeded for a project's plot board on first use, mirroring
/// `world_repository::BUILTIN_CATEGORY_NAMES`'s seed-on-first-read pattern.
/// Users can rename/add/remove lanes afterwards.
pub const DEFAULT_PLOT_LANE_NAMES: &[&str] = &["構想", "執筆予定", "執筆中", "完了"];

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PlotCard {
    pub id: String,
    pub project_id: String,
    pub lane_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub chapter_id: Option<String>,
    pub color: Option<String>,
    pub order_index: i64,
}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TimelineEvent {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub event_date: Option<String>,
    pub description: Option<String>,
    pub chapter_id: Option<String>,
    pub scene_id: Option<String>,
    pub order_index: i64,
}


/// 仕様#35の伏線ステータス: 構想/設置予定/設置済/ヒント提示済/回収予定/回収済/破棄
pub const FORESHADOWING_STATUSES: &[&str] = &[
    "idea",           // 構想
    "planned",        // 設置予定
    "planted",        // 設置済
    "hinted",         // ヒント提示済
    "payoff_planned", // 回収予定
    "resolved",       // 回収済
    "discarded",      // 破棄
];

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Foreshadowing {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub detail: Option<String>,
    pub status: String,
    pub planted_chapter_id: Option<String>,
    pub payoff_chapter_id: Option<String>,
    pub memo: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Todo {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub done: bool,
    pub due_date: Option<String>,
    pub memo: Option<String>,
    pub order_index: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Comment {
    pub id: String,
    pub project_id: String,
    pub owner_type: String,
    pub owner_id: String,
    pub anchor_start: Option<i64>,
    pub anchor_end: Option<i64>,
    pub quote: Option<String>,
    pub body: String,
    pub resolved: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Phase 5: AI基盤 (Provider abstraction / Context Builder / Chat / Usage)
// ---------------------------------------------------------------------------

/// 対応プロバイダー種別。追加時はここと `ai::provider::build_provider` の
/// 両方を更新する(1箇所に依存を閉じ込めるための唯一の分岐点)。
pub const AI_PROVIDER_KINDS: &[&str] = &["anthropic", "openai", "gemini", "openai_compatible"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiChatMessage {
    pub id: String,
    pub project_id: String,
    pub role: String,
    pub content: String,
    pub context_summary: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AiUsageSummary {
    pub today_requests: i64,
    pub today_input_tokens: i64,
    pub today_output_tokens: i64,
    pub month_requests: i64,
    pub month_input_tokens: i64,
    pub month_output_tokens: i64,
}

/// Context Inspector(仕様#42)がユーザーへ提示する、送信候補となる情報の
/// 1ブロック。`included` はデフォルトのチェック状態で、実際に送るかどうかは
/// フロント側でユーザーがトグルしてから `selected_labels` として返す。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextBlock {
    pub label: String,
    pub content: String,
    pub char_count: i64,
    pub included_by_default: bool,
}

/// Provider Gatewayへ渡す1メッセージ。プロバイダーごとのAPI形式差異は
/// `ai::provider` 実装側で吸収する。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiChatTurn {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiSendRequest {
    pub provider: String,
    pub model: String,
    pub base_url: Option<String>,
    pub temperature: Option<f64>,
    pub max_output_tokens: Option<i64>,
    pub system: Option<String>,
    pub messages: Vec<AiChatTurn>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AiSendResponse {
    pub content: String,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
}

// ---------------------------------------------------------------------------
// Phase 7: 高度AI分析(矛盾チェック/Character口調/Timeline矛盾/設定矛盾/
// 未回収伏線/文体分析)
// ---------------------------------------------------------------------------

pub const AI_ANALYSIS_TYPES: &[&str] =
    &["contradiction", "character_tone", "timeline", "setting", "foreshadowing", "style"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiAnalysisReport {
    pub id: String,
    pub project_id: String,
    pub analysis_type: String,
    pub target_summary: Option<String>,
    pub context_summary: Option<String>,
    pub result: String,
    pub provider: String,
    pub model: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Phase 8: Revision履歴 / Crash Recovery / Auto Backup / ゴミ箱
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Revision {
    pub id: String,
    pub owner_type: String,
    pub owner_id: String,
    pub body: String,
    pub char_count: i64,
    pub trigger: String,
    pub label: Option<String>,
    pub created_at: String,
}

/// ゴミ箱(仕様のTrash/Restore相当)。複数テーブルにまたがる
/// Soft Delete済みレコードを横断的に一覧するための統一表現。
/// `entity_type`は`trash_repository`が対応する各テーブル名の日本語ラベル
/// ではなく、復元/完全削除の際にテーブルを引くための内部識別子
/// (例: "chapter", "scene", "character")。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrashItem {
    pub entity_type: String,
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub deleted_at: String,
}

/// DBファイル全体のバックアップ1件のメタ情報。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupInfo {
    pub filename: String,
    pub created_at: String,
    pub size_bytes: u64,
}

/// AI不要のローカル文体統計(仕様の可読性分析相当)。本文の全文字を対象に
/// 純粋な集計のみ行うため、AI機能が未設定でも常に利用できる
/// (CLAUDE.md「AI機能が利用不能でも通常の小説執筆機能は動作すること」)。
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ReadabilityStats {
    pub total_chars: i64,
    pub sentence_count: i64,
    pub avg_sentence_length: f64,
    pub max_sentence_length: i64,
    pub long_sentence_count: i64,
    pub dialogue_ratio: f64,
    pub kanji_ratio: f64,
    pub hiragana_ratio: f64,
    pub katakana_ratio: f64,
    pub avg_touten_per_sentence: f64,
}
