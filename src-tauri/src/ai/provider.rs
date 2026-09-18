//! Provider Abstraction (仕様#40, docs/AI.md 1章)。
//!
//! `AiProvider` トレイトを唯一の境界とし、呼び出し側(AI Feature/Context
//! Builder/commands)はどのプロバイダーが選ばれているかを意識しない。
//! 新しいプロバイダーを追加する場合は、このファイルの
//! `AI_PROVIDER_KINDS`定数と`build_provider`のみを更新すればよい。

use crate::error::{AppError, AppResult};
use crate::models::{AiSendRequest, AiSendResponse};

pub trait AiProvider {
    fn send(&self, api_key: &str, req: &AiSendRequest) -> AppResult<AiSendResponse>;
}

pub fn build_provider(kind: &str) -> AppResult<Box<dyn AiProvider>> {
    match kind {
        "anthropic" => Ok(Box::new(super::anthropic::AnthropicProvider)),
        "openai" => Ok(Box::new(super::openai_compatible::OpenAiCompatibleProvider::openai())),
        "gemini" => Ok(Box::new(super::gemini::GeminiProvider)),
        "openai_compatible" => Ok(Box::new(super::openai_compatible::OpenAiCompatibleProvider::custom())),
        other => Err(AppError::Other(format!("未対応のAIプロバイダーです: {other}"))),
    }
}
