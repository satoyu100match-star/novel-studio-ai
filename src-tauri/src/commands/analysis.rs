use crate::ai::keystore::AiKeyStore;
use crate::ai::{analysis, context_builder, provider};
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::{AiAnalysisReport, AiChatTurn, AiSendRequest, ContextBlock, ReadabilityStats, AI_ANALYSIS_TYPES, AI_PROVIDER_KINDS};
use crate::repositories::{analysis_repository, ai_usage_repository, characters_repository};
use tauri::State;

#[tauri::command]
pub fn list_ai_analysis_types() -> AppResult<Vec<String>> {
    Ok(AI_ANALYSIS_TYPES.iter().map(|s| s.to_string()).collect())
}

#[tauri::command]
pub fn build_ai_analysis_context(
    db: State<Db>,
    project_id: String,
    analysis_type: String,
    character_id: Option<String>,
) -> AppResult<Vec<ContextBlock>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    context_builder::build_analysis_context(&conn, &project_id, &analysis_type, character_id.as_deref())
}

/// AI不要のローカル文体統計(可読性分析)。設定不要で常に計算できる。
#[tauri::command]
pub fn compute_readability_stats(db: State<Db>, project_id: String) -> AppResult<ReadabilityStats> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    let text = context_builder::full_manuscript_text(&conn, &project_id)?;
    Ok(crate::readability::compute(&text))
}

/// Phase 7: 高度AI分析の実行。Phase6の執筆支援機能とは異なり、分析結果は
/// 「読んで参考にするだけ」で原稿等への書き込みは発生しない。実行結果は
/// `ai_analysis_reports`に保存し、後から履歴として見返せるようにする。
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn run_ai_analysis(
    db: State<Db>,
    key_store: State<AiKeyStore>,
    project_id: String,
    analysis_type: String,
    character_id: Option<String>,
    provider_kind: String,
    model: String,
    base_url: Option<String>,
    temperature: Option<f64>,
    max_output_tokens: Option<i64>,
    selected_context: Vec<ContextBlock>,
) -> AppResult<AiAnalysisReport> {
    if !AI_PROVIDER_KINDS.contains(&provider_kind.as_str()) {
        return Err(AppError::Other(format!("未対応のAIプロバイダーです: {provider_kind}")));
    }
    let base_prompt = analysis::system_prompt(&analysis_type)?;
    if analysis::requires_character(&analysis_type) && character_id.is_none() {
        return Err(AppError::Other("対象キャラクターを選択してください。".into()));
    }

    let target_summary = if let Some(cid) = &character_id {
        let conn = db.conn.lock().expect("db mutex poisoned");
        characters_repository::get(&conn, cid)?.map(|c| c.name)
    } else {
        None
    };

    if selected_context.is_empty() {
        return Err(AppError::Other("分析対象の情報が選択されていません。".into()));
    }

    let mut system_lines = vec![base_prompt.to_string()];
    for block in &selected_context {
        system_lines.push(format!("\n## {}\n{}", block.label, block.content));
    }
    let system = Some(system_lines.join("\n"));
    let context_summary = Some(selected_context.iter().map(|b| b.label.as_str()).collect::<Vec<_>>().join(", "));

    let user_content = "上記の情報をもとに分析結果を日本語で述べてください。".to_string();

    let api_key = key_store.get(&provider_kind).unwrap_or_default();
    let request = AiSendRequest {
        provider: provider_kind.clone(),
        model: model.clone(),
        base_url,
        temperature,
        max_output_tokens,
        system,
        messages: vec![AiChatTurn { role: "user".to_string(), content: user_content }],
    };

    let ai_provider = provider::build_provider(&provider_kind)?;
    let response = ai_provider.send(&api_key, &request)?;

    let conn = db.conn.lock().expect("db mutex poisoned");
    ai_usage_repository::log(
        &conn,
        Some(project_id.as_str()),
        &format!("analysis_{analysis_type}"),
        &provider_kind,
        &model,
        response.input_tokens,
        response.output_tokens,
    )?;
    analysis_repository::create(
        &conn,
        &project_id,
        &analysis_type,
        target_summary.as_deref(),
        context_summary.as_deref(),
        &response.content,
        &provider_kind,
        &model,
    )
}

#[tauri::command]
pub fn list_ai_analysis_reports(
    db: State<Db>,
    project_id: String,
    analysis_type: Option<String>,
) -> AppResult<Vec<AiAnalysisReport>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    analysis_repository::list_by_project(&conn, &project_id, analysis_type.as_deref())
}

#[tauri::command]
pub fn delete_ai_analysis_report(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    analysis_repository::delete(&conn, &id)
}
