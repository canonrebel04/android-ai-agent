//! DeepSeek provider — supports both OpenAI-compatible and Anthropic-compatible APIs.
//!
//! OpenAI API: https://api.deepseek.com (Bearer auth)
//! Anthropic API: https://api.deepseek.com/anthropic (x-api-key auth, model auto-mapped)
//!
//! Models: deepseek-v4-flash (cheap, 1M context), deepseek-v4-pro (powerful, 1M context)
//! Both support thinking mode via `thinking: {type: "enabled"}` field.
//! Pricing: flash $0.14/$0.28 per 1M in/out, pro $0.435/$0.87 (75% off currently).

use super::openai_compat;
use super::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use std::future::Future;

pub struct DeepSeekProvider {
    api_key: String,
    /// Use Anthropic-compatible API (https://api.deepseek.com/anthropic) instead of OpenAI.
    pub use_anthropic_api: bool,
    /// Enable thinking mode (adds reasoning_content to responses).
    pub thinking_enabled: bool,
}

impl DeepSeekProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            use_anthropic_api: false,
            thinking_enabled: false,
        }
    }

    /// Use the Anthropic-compatible API endpoint.
    pub fn with_anthropic_api(mut self) -> Self {
        self.use_anthropic_api = true;
        self
    }

    /// Enable thinking mode for reasoning-capable tasks.
    pub fn with_thinking(mut self) -> Self {
        self.thinking_enabled = true;
        self
    }
}

impl LlmProvider for DeepSeekProvider {
    fn base_url(&self) -> &str {
        if self.use_anthropic_api {
            "https://api.deepseek.com/anthropic"
        } else {
            "https://api.deepseek.com"
        }
    }

    fn auth_header(&self) -> (&str, &str) {
        if self.use_anthropic_api {
            ("x-api-key", &self.api_key)
        } else {
            ("Authorization", &self.api_key)
        }
    }

    fn call(
        &self,
        client: &reqwest::Client,
        request: &LlmRequest,
    ) -> impl Future<Output = Result<LlmResponse, LlmError>> + Send {
        let api_key = self.api_key.clone();
        let base_url = self.base_url().to_string();
        let thinking = self.thinking_enabled;
        let model = request.model.clone();
        let max_tokens = request.max_tokens;
        let temperature = request.temperature;
        let messages: Vec<_> = request
            .messages
            .iter()
            .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            .collect();

        async move {
            let mut body = serde_json::json!({
                "model": model,
                "messages": messages,
                "max_tokens": max_tokens,
                "temperature": temperature,
            });

            // Add thinking mode if enabled
            if thinking {
                body["thinking"] = serde_json::json!({"type": "enabled"});
                body["reasoning_effort"] = serde_json::json!("medium");
            }

            let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

            let resp = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(LlmError::Http)?;

            let resp = resp.error_for_status().map_err(LlmError::Http)?;
            let json: serde_json::Value = resp.json().await.map_err(LlmError::Http)?;

            openai_compat::parse_openai_response(&json, &request.model)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepseek_base_url() {
        let provider = DeepSeekProvider::new("sk-test".into());
        assert_eq!(provider.base_url(), "https://api.deepseek.com");

        let anthropic = DeepSeekProvider::new("sk-test".into()).with_anthropic_api();
        assert_eq!(anthropic.base_url(), "https://api.deepseek.com/anthropic");
    }

    #[test]
    fn test_deepseek_auth_header() {
        let provider = DeepSeekProvider::new("sk-test".into());
        assert_eq!(provider.auth_header(), ("Authorization", "sk-test"));

        let anthropic = DeepSeekProvider::new("sk-test".into()).with_anthropic_api();
        assert_eq!(anthropic.auth_header(), ("x-api-key", "sk-test"));
    }

    #[test]
    fn test_thinking_enabled_by_default() {
        let provider = DeepSeekProvider::new("sk-test".into());
        assert!(!provider.thinking_enabled);
    }
}
