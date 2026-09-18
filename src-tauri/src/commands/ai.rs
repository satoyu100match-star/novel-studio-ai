use crate::ai::keystore::AiKeyStore;
use crate::ai::{context_builder, features, provider};
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::{
    AiChatMessage, AiChatTurn, AiSendRequest, AiSendResponse, AiUsageSummary, ContextBlock, AI_PROVIDER_KINDS,
};
use crate::repositories::{ai_chat_repository, ai_usage_repository};
use tauri::State;

#[tauri::command]
pub fn list_ai_provider_kinds() -> AppResult<Vec<String>> {
    Ok(AI_PROVIDER_KINDS.iter().map(|s| s.to_string()).collect())
}

#[tauri::command]
pub fn list_ai_writing_features() -> AppResult<Vec<String>> {
    Ok(features::AI_WRITING_FEATURES.iter().map(|s| s.to_string()).collect())
}

#[tauri::command]
pub fn set_ai_api_key(key_store: State<AiKeyStore>, provider: String, api_key: String) -> AppResult<()> {
    if !AI_PROVIDER_KINDS.contains(&provider.as_str()) {
        return Err(AppError::Other(format!("未対応のAIプロバイダーです: {provider}")));
    }
    key_store.set(&provider, api_key);
    Ok(())
}

#[tauri::command]
pub fn has_ai_api_key(key_store: State<AiKeyStore>, provider: String) -> AppResult<bool> {
    Ok(key_store.has(&provider))
}

#[tauri::command]
pub fn clear_ai_api_key(key_store: State<AiKeyStore>, provider: String) -> AppResult<()> {
    key_store.clear(&provider);
    Ok(())
}

#[tauri::command]
pub fn build_ai_context(
    db: State<Db>,
    project_id: String,
    chapter_id: Option<String>,
    scene_id: Option<String>,
    user_question: Option<String>,
) -> AppResult<Vec<ContextBlock>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    context_builder::build_context(
        &conn,
        &project_id,
        chapter_id.as_deref(),
        scene_id.as_deref(),
        user_question.as_deref(),
    )
}

#[tauri::command]
pub fn list_ai_chat_messages(db: State<Db>, project_id: String) -> AppResult<Vec<AiChatMessage>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    ai_chat_repository::list_by_project(&conn, &project_id)
}

#[tauri::command]
pub fn clear_ai_chat(db: State<Db>, project_id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    ai_chat_repository::clear(&conn, &project_id)
}

/// Context Inspector(仕様#42)でユーザーが確認・選択したブロックのみを
/// システムプロンプトへ組み込み、AI Gatewayへ送信する。承認前のバイパス
/// 経路は作らない(docs/AI.md 2章方針)。
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn send_ai_chat_message(
    db: State<Db>,
    key_store: State<AiKeyStore>,
    project_id: String,
    provider_kind: String,
    model: String,
    base_url: Option<String>,
    temperature: Option<f64>,
    max_output_tokens: Option<i64>,
    selected_context: Vec<ContextBlock>,
    user_content: String,
) -> AppResult<Vec<AiChatMessage>> {
    if !AI_PROVIDER_KINDS.contains(&provider_kind.as_str()) {
        return Err(AppError::Other(format!("未対応のAIプロバイダーです: {provider_kind}")));
    }
    if user_content.trim().is_empty() {
        return Err(AppError::Other("メッセージが空です。".into()));
    }

    let context_summary = if selected_context.is_empty() {
        None
    } else {
        Some(selected_context.iter().map(|b| b.label.as_str()).collect::<Vec<_>>().join(", "))
    };

    let user_message = {
        let conn = db.conn.lock().expect("db mutex poisoned");
        ai_chat_repository::append(&conn, &project_id, "user", &user_content, context_summary.as_deref(), None, None)?
    };

    let history = {
        let conn = db.conn.lock().expect("db mutex poisoned");
        ai_chat_repository::list_by_project(&conn, &project_id)?
    };

    let system = if selected_context.is_empty() {
        None
    } else {
        let mut lines = vec![
            "あなたは小説創作を支援するアシスタントです。以下は作者が確認・選択した、この作品に関する参考情報です。".to_string(),
        ];
        for block in &selected_context {
            lines.push(format!("\n## {}\n{}", block.label, block.content));
        }
        Some(lines.join("\n"))
    };

    let messages: Vec<AiChatTurn> =
        history.iter().map(|m| AiChatTurn { role: m.role.clone(), content: m.content.clone() }).collect();

    let api_key = key_store.get(&provider_kind).unwrap_or_default();
    let request = AiSendRequest {
        provider: provider_kind.clone(),
        model: model.clone(),
        base_url,
        temperature,
        max_output_tokens,
        system,
        messages,
    };

    let ai_provider = provider::build_provider(&provider_kind)?;
    let response = ai_provider.send(&api_key, &request)?;

    let assistant_message = {
        let conn = db.conn.lock().expect("db mutex poisoned");
        ai_usage_repository::log(
            &conn,
            Some(project_id.as_str()),
            "chat",
            &provider_kind,
            &model,
            response.input_tokens,
            response.output_tokens,
        )?;
        ai_chat_repository::append(
            &conn,
            &project_id,
            "assistant",
            &response.content,
            None,
            Some(&provider_kind),
            Some(&model),
        )?
    };

    Ok(vec![user_message, assistant_message])
}

#[tauri::command]
pub fn get_ai_usage_summary(db: State<Db>, project_id: Option<String>) -> AppResult<AiUsageSummary> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    ai_usage_repository::summary(&conn, project_id.as_deref())
}

/// Phase 6: 個別のAI執筆支援機能(推敲/続き案/描写追加/会話改善/各種生成)。
/// チャットとは異なり会話履歴には保存しない(1回限りの提案生成のため)。
/// 応答を実際に原稿やキャラクター等へ反映するのは必ずフロント側での
/// ユーザーの明示的な操作(適用ボタン)であり、このコマンドはテキストを
/// 生成して返すだけで、いかなるテーブルへの書き込みも行わない
/// (docs/AI.md 4章の承認フローをバックエンド側からも壊さないための設計)。
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn run_ai_writing_feature(
    db: State<Db>,
    key_store: State<AiKeyStore>,
    project_id: String,
    feature: String,
    provider_kind: String,
    model: String,
    base_url: Option<String>,
    temperature: Option<f64>,
    max_output_tokens: Option<i64>,
    selected_context: Vec<ContextBlock>,
    input_text: Option<String>,
) -> AppResult<AiSendResponse> {
    if !AI_PROVIDER_KINDS.contains(&provider_kind.as_str()) {
        return Err(AppError::Other(format!("未対応のAIプロバイダーです: {provider_kind}")));
    }
    let base_prompt = features::system_prompt(&feature)?;
    let input_text = input_text.unwrap_or_default();
    if features::requires_input_text(&feature) && input_text.trim().is_empty() {
        return Err(AppError::Other("対象の本文が空です。".into()));
    }

    let mut system_lines = vec![base_prompt.to_string()];
    for block in &selected_context {
        system_lines.push(format!("\n## {}\n{}", block.label, block.content));
    }
    let system = Some(system_lines.join("\n"));

    let user_content =
        if input_text.trim().is_empty() { "(追加の指示はありません。上記の情報のみを踏まえて提案してください。)".to_string() } else { input_text };

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

    {
        let conn = db.conn.lock().expect("db mutex poisoned");
        ai_usage_repository::log(
            &conn,
            Some(project_id.as_str()),
            &feature,
            &provider_kind,
            &model,
            response.input_tokens,
            response.output_tokens,
        )?;
    }

    Ok(response)
}
