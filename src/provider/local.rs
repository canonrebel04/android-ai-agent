//! Local provider — LM Studio / Ollama / local-llm format.
//!
//! Default API: http://localhost:1234/v1 (LM Studio)
//! Alternative: http://localhost:11434/v1 (Ollama)
//! No auth required by default. Supports all OpenAI-compatible endpoints.

use super::openai_compat;
use super::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use std::future::Future;
use futures_util::Stream;
use std::pin::Pin;

pub struct LocalProvider {
    base_url: String,
}

impl LocalProvider {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "http://localhost:1234/v1".into()),
        }
    }
}

impl LlmProvider for LocalProvider {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn auth_header(&self) -> (&str, &str) {
        ("Authorization", "Bearer local-no-auth")
    }

    fn call(
        &self,
        client: &reqwest::Client,
        request: &LlmRequest,
    ) -> impl Future<Output = Result<LlmResponse, LlmError>> + Send {
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
            yield "Local streaming not implemented".to_string();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_default_base_url() {
        let provider = LocalProvider::new(None);
        assert_eq!(provider.base_url(), "http://localhost:1234/v1");
    }

    #[test]
    fn test_local_custom_base_url() {
        let provider = LocalProvider::new(Some("http://10.0.0.1:11434/v1".into()));
        assert_eq!(provider.base_url(), "http://10.0.0.1:11434/v1");
    }

    #[test]
    fn test_local_no_auth_by_default() {
        let provider = LocalProvider::new(None);
        assert_eq!(provider.auth_header().0, "Authorization");
    }
}
