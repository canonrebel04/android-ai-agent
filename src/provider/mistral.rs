//! Mistral provider — OpenAI-compatible API format.
//!
//! API: https://api.mistral.ai/v1 (Bearer auth)
//! Models: mistral-small-3.2, mistral-large-latest, pixtral-12b-latest
//! Pricing: small $0.2/$0.6 per 1M, large $2/$6 (competitive with Claude Sonnet).

use super::openai_compat;
use super::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use std::future::Future;
use futures_util::Stream;
use std::pin::Pin;

pub struct MistralProvider {
    api_key: String,
}

impl MistralProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl LlmProvider for MistralProvider {
    fn base_url(&self) -> &str {
        "https://api.mistral.ai/v1"
    }

    fn auth_header(&self) -> (&str, &str) {
        ("Authorization", &self.api_key)
    }

    fn call(
        &self,
        client: &reqwest::Client,
        request: &LlmRequest,
    ) -> impl Future<Output = Result<LlmResponse, LlmError>> + Send {
        let api_key = self.api_key.clone();
        let base_url = self.base_url().to_string();
        let model = request.model.clone();
        let req = request.clone();

        async move {
            let body = serde_json::json!({
                "model": req.model,
                "messages": req.messages.iter().map(|m| {
                    serde_json::json!({"role": m.role, "content": m.content})
                }).collect::<Vec<_>>(),
                "max_tokens": req.max_tokens,
                "temperature": req.temperature,
            });

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

            openai_compat::parse_openai_response(&json, &model)
        }
    }

    fn call_stream(
        &self,
        _client: &reqwest::Client,
        _request: &LlmRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>> {
        Box::pin(async_stream::try_stream! {
            yield "Mistral streaming not implemented".to_string();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mistral_base_url() {
        let provider = MistralProvider::new("test".into());
        assert_eq!(provider.base_url(), "https://api.mistral.ai/v1");
    }
}
