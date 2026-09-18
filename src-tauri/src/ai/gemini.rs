use super::http_client::{self, HttpRequest};
use super::provider::AiProvider;
use crate::error::{AppError, AppResult};
use crate::models::{AiSendRequest, AiSendResponse};
use serde_json::json;
use std::time::Duration;

pub struct GeminiProvider;

impl AiProvider for GeminiProvider {
    fn send(&self, api_key: &str, req: &AiSendRequest) -> AppResult<AiSendResponse> {
        if api_key.is_empty() {
            return Err(AppError::Other("Gemini APIキーが設定されていません。設定画面から登録してください。".into()));
        }
        // Gemini はURLのクエリパラメータでAPIキーを渡す方式。キー自体は
        // curlのargvには載せず(http_clientがヘッダー経由でしか機微情報を
        // 渡さない設計のため)、URLに含める。URLはプロセス一覧に出ても
        // クエリ文字列としてよく知られた形であり、他プロバイダー(ヘッダー
        // 方式)ほど厳密に隠す実装上の手段がGoogle API自体の仕様上ない点に
        // 留意(Google公式SDKも同様にクエリパラメータでキーを渡す)。
        let base = req
            .base_url
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());
        let url = format!("{}/models/{}:generateContent?key={}", base.trim_end_matches('/'), req.model, api_key);

        let contents: Vec<_> = req
            .messages
            .iter()
            .map(|m| {
                let role = if m.role == "assistant" { "model" } else { "user" };
                json!({ "role": role, "parts": [{ "text": m.content }] })
            })
            .collect();

        let mut body = json!({ "contents": contents });
        if let Some(system) = &req.system {
            body["systemInstruction"] = json!({ "parts": [{ "text": system }] });
        }
        let mut generation_config = json!({});
        if let Some(temperature) = req.temperature {
            generation_config["temperature"] = json!(temperature);
        }
        if let Some(max_tokens) = req.max_output_tokens {
            generation_config["maxOutputTokens"] = json!(max_tokens);
        }
        if generation_config.as_object().map(|o| !o.is_empty()).unwrap_or(false) {
            body["generationConfig"] = generation_config;
        }

        let headers = [("content-type", "application/json".to_string())];
        let response = http_client::send(HttpRequest {
            method: "POST",
            url: &url,
            headers: &headers,
            body_json: Some(body.to_string()),
            timeout: Duration::from_secs(120),
        })?;

        if response.status < 200 || response.status >= 300 {
            return Err(AppError::Other(format!(
                "Gemini APIがエラーを返しました(status={}): {}",
                response.status,
                truncate(&response.body, 500)
            )));
        }

        let parsed: serde_json::Value = serde_json::from_str(&response.body)
            .map_err(|e| AppError::Other(format!("Gemini APIの応答を解析できませんでした: {e}")))?;

        let text = parsed["candidates"][0]["content"]["parts"][0]["text"].as_str().unwrap_or("").to_string();

        Ok(AiSendResponse {
            content: text,
            input_tokens: parsed["usageMetadata"]["promptTokenCount"].as_i64(),
            output_tokens: parsed["usageMetadata"]["candidatesTokenCount"].as_i64(),
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
