pub mod anthropic;
pub mod deepseek;
pub mod google;
pub mod local;
pub mod mistral;
pub mod openai_compat;
pub mod openrouter;

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

use futures_util::Stream;
use std::pin::Pin;

pub trait LlmProvider: Send + Sync {
    fn base_url(&self) -> &str;
    fn auth_header(&self) -> (&str, &str);
    fn call(
        &self,
        client: &reqwest::Client,
        request: &LlmRequest,
    ) -> impl Future<Output = Result<LlmResponse, LlmError>> + Send;

    fn call_stream(
        &self,
        client: &reqwest::Client,
        request: &LlmRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>;
}

use crate::prompt_cache::{CacheBreakpoint, CacheableProvider, CachedRequest};

pub enum ProviderBackend {
    Anthropic(anthropic::AnthropicProvider),
    Google(google::GoogleProvider),
    OpenRouter(openrouter::OpenRouterProvider),
    DeepSeek(deepseek::DeepSeekProvider),
    Mistral(mistral::MistralProvider),
    Local(local::LocalProvider),
}

impl ProviderBackend {
    pub fn base_url(&self) -> &str {
        match self {
            Self::Anthropic(p) => p.base_url(),
            Self::Google(p) => p.base_url(),
            Self::OpenRouter(p) => p.base_url(),
            Self::DeepSeek(p) => p.base_url(),
            Self::Mistral(p) => p.base_url(),
            Self::Local(p) => p.base_url(),
        }
    }

    pub fn auth_header(&self) -> (&str, &str) {
        match self {
            Self::Anthropic(p) => p.auth_header(),
            Self::Google(p) => p.auth_header(),
            Self::OpenRouter(p) => p.auth_header(),
            Self::DeepSeek(p) => p.auth_header(),
            Self::Mistral(p) => p.auth_header(),
            Self::Local(p) => p.auth_header(),
        }
    }

    pub async fn call(
        &self,
        client: &reqwest::Client,
        request: &LlmRequest,
    ) -> Result<LlmResponse, LlmError> {
        match self {
            Self::Anthropic(p) => p.call(client, request).await,
            Self::Google(p) => p.call(client, request).await,
            Self::OpenRouter(p) => p.call(client, request).await,
            Self::DeepSeek(p) => p.call(client, request).await,
            Self::Mistral(p) => p.call(client, request).await,
            Self::Local(p) => p.call(client, request).await,
        }
    }

    pub fn call_stream(
        &self,
        client: &reqwest::Client,
        request: &LlmRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>> {
        match self {
            Self::Anthropic(p) => p.call_stream(client, request),
            Self::Google(p) => p.call_stream(client, request),
            Self::OpenRouter(p) => p.call_stream(client, request),
            Self::DeepSeek(p) => p.call_stream(client, request),
            Self::Mistral(p) => p.call_stream(client, request),
            Self::Local(p) => p.call_stream(client, request),
        }
    }

    pub fn apply_cache(
        &self,
        body: &serde_json::Value,
        breakpoints: &[CacheBreakpoint],
    ) -> CachedRequest {
        match self {
            Self::Anthropic(p) => p.apply_cache(body, breakpoints),
            Self::OpenRouter(p) => p.apply_cache(body, breakpoints),
            _ => CachedRequest::default(),
        }
    }
}
