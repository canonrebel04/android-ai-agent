//! Google Gemini provider — OpenAI-compatible API via OpenRouter format.
//! Direct Gemini API uses Google Generative AI format, but OpenRouter bridges it.
//! For direct access: https://generativelanguage.googleapis.com/v1beta (different format).
//! This implementation uses the OpenAI-compatible endpoint most providers expose.
//!
//! Alternative: set GOOGLE_API_KEY and use native Gemini API.
//! Currently routes through OpenRouter for consistency.

use super::openai_compat;
use super::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use futures_util::Stream;
use std::future::Future;
use std::pin::Pin;

pub struct GoogleProvider {
    api_key: String,
}

impl GoogleProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl LlmProvider for GoogleProvider {
    fn base_url(&self) -> &str {
        "https://generativelanguage.googleapis.com/v1beta/openai"
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
        let req = request.clone();

        async move {
            // Gemini OpenAI-compat uses query param auth, not header
            let body = serde_json::json!({
                "model": req.model,
                "messages": req.messages.iter().map(|m| {
                    serde_json::json!({"role": m.role, "content": m.content})
                }).collect::<Vec<_>>(),
                "max_tokens": req.max_tokens,
                "temperature": req.temperature,
            });

            let url = format!(
                "{}/chat/completions?key={}",
                base_url.trim_end_matches('/'),
                api_key
            );

            let resp = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(LlmError::Http)?;

            let resp = resp.error_for_status().map_err(LlmError::Http)?;
            let json: serde_json::Value = resp.json().await.map_err(LlmError::Http)?;

            openai_compat::parse_openai_response(&json, &req.model)
        }
    }

    fn call_stream(
        &self,
        _client: &reqwest::Client,
        _request: &LlmRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>> {
        Box::pin(async_stream::try_stream! {
            yield "Google streaming not implemented".to_string();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_base_url() {
        let provider = GoogleProvider::new("test".into());
        assert_eq!(
            provider.base_url(),
            "https://generativelanguage.googleapis.com/v1beta/openai"
        );
    }
}
