pub mod openrouter;
pub mod anthropic;
pub mod deepseek;
pub mod google;
pub mod mistral;
pub mod local;
pub mod openai_compat;

use serde::{Deserialize, Serialize};
use std::future::Future;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("rate limited, retry after {0}s")]
    RateLimit(u64),
    #[error("model unavailable: {0}")]
    ModelUnavailable(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("all fallbacks exhausted")]
    AllFallbacksExhausted,
}

pub trait LlmProvider: Send + Sync {
    fn base_url(&self) -> &str;
    fn auth_header(&self) -> (&str, &str);
    fn call(
        &self,
        client: &reqwest::Client,
        request: &LlmRequest,
    ) -> impl Future<Output = Result<LlmResponse, LlmError>> + Send;
}
