use super::http_client::{self, HttpRequest};
use super::provider::AiProvider;
use crate::error::{AppError, AppResult};
use crate::models::{AiSendRequest, AiSendResponse};
use serde_json::json;
use std::time::Duration;

const OPENAI_DEFAULT_URL: &str = "https://api.openai.com/v1/chat/completions";

/// OpenAIのChat Completions形式(仕様#40)。`openai`(固定エンドポイント)と
/// `openai_compatible`(ユーザー指定のBase URL、ローカルLLMサーバー等)の
/// 両方をこの1実装でカバーする -- APIの形が同一なため。
pub struct OpenAiCompatibleProvider {
    fixed_url: Option<&'static str>,
}

impl OpenAiCompatibleProvider {
    pub fn openai() -> Self {
        Self { fixed_url: Some(OPENAI_DEFAULT_URL) }
    }
    pub fn custom() -> Self {
        Self { fixed_url: None }
    }
}

impl AiProvider for OpenAiCompatibleProvider {
    fn send(&self, api_key: &str, req: &AiSendRequest) -> AppResult<AiSendResponse> {
        let url = match self.fixed_url {
            Some(u) => u.to_string(),
            None => req.base_url.clone().ok_or_else(|| {
                AppError::Other("OpenAI互換プロバイダーにはBase URLの設定が必要です。".into())
            })?,
        };
        if self.fixed_url.is_some() && api_key.is_empty() {
            return Err(AppError::Other("OpenAI APIキーが設定されていません。設定画面から登録してください。".into()));
        }

        let mut messages = Vec::new();
        if let Some(system) = &req.system {
            messages.push(json!({ "role": "system", "content": system }));
        }
        for m in &req.messages {
            messages.push(json!({ "role": m.role, "content": m.content }));
        }
        let mut body = json!({
            "model": req.model,
            "messages": messages,
        });
        if let Some(temperature) = req.temperature {
            body["temperature"] = json!(temperature);
        }
        if let Some(max_tokens) = req.max_output_tokens {
            body["max_tokens"] = json!(max_tokens);
        }

        let mut headers: Vec<(&str, String)> = vec![("content-type", "application/json".to_string())];
        if !api_key.is_empty() {
            headers.push(("Authorization", format!("Bearer {api_key}")));
        }

        let response = http_client::send(HttpRequest {
            method: "POST",
            url: &url,
            headers: &headers,
            body_json: Some(body.to_string()),
            timeout: Duration::from_secs(120),
        })?;

        if response.status < 200 || response.status >= 300 {
            return Err(AppError::Other(format!(
                "AIプロバイダーがエラーを返しました(status={}): {}",
                response.status,
                truncate(&response.body, 500)
            )));
        }

        let parsed: serde_json::Value = serde_json::from_str(&response.body)
            .map_err(|e| AppError::Other(format!("AIプロバイダーの応答を解析できませんでした: {e}")))?;

        let text = parsed["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();

        Ok(AiSendResponse {
            content: text,
            input_tokens: parsed["usage"]["prompt_tokens"].as_i64(),
            output_tokens: parsed["usage"]["completion_tokens"].as_i64(),
        })
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect::<String>() + "..."
    }
}
