use super::http_client::{self, HttpRequest};
use super::provider::AiProvider;
use crate::error::{AppError, AppResult};
use crate::models::{AiSendRequest, AiSendResponse};
use serde_json::json;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider;

impl AiProvider for AnthropicProvider {
    fn send(&self, api_key: &str, req: &AiSendRequest) -> AppResult<AiSendResponse> {
        if api_key.is_empty() {
            return Err(AppError::Other("Anthropic APIキーが設定されていません。設定画面から登録してください。".into()));
        }
        let url = req.base_url.clone().unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let messages: Vec<_> = req
            .messages
            .iter()
            .map(|m| json!({ "role": m.role, "content": m.content }))
            .collect();
        let mut body = json!({
            "model": req.model,
            "max_tokens": req.max_output_tokens.unwrap_or(2048),
            "messages": messages,
        });
        if let Some(system) = &req.system {
            body["system"] = json!(system);
        }
        if let Some(temperature) = req.temperature {
            body["temperature"] = json!(temperature);
        }

        let headers = [
            ("x-api-key", api_key.to_string()),
            ("anthropic-version", ANTHROPIC_VERSION.to_string()),
            ("content-type", "application/json".to_string()),
        ];
        let response = http_client::send(HttpRequest {
            method: "POST",
            url: &url,
            headers: &headers,
            body_json: Some(body.to_string()),
            timeout: Duration::from_secs(120),
        })?;

        if response.status < 200 || response.status >= 300 {
            return Err(AppError::Other(format!(
                "Anthropic APIがエラーを返しました(status={}): {}",
                response.status,
                truncate(&response.body, 500)
            )));
        }

        let parsed: serde_json::Value = serde_json::from_str(&response.body)
            .map_err(|e| AppError::Other(format!("Anthropic APIの応答を解析できませんでした: {e}")))?;

        let text = parsed["content"]
            .as_array()
            .and_then(|blocks| blocks.iter().find(|b| b["type"] == "text"))
            .and_then(|b| b["text"].as_str())
            .unwrap_or("")
            .to_string();

        Ok(AiSendResponse {
            content: text,
            input_tokens: parsed["usage"]["input_tokens"].as_i64(),
            output_tokens: parsed["usage"]["output_tokens"].as_i64(),
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
