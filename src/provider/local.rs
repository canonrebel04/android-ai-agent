//! Local LLM provider — Ollama / llama.cpp / vLLM (OpenAI-compatible).
//! Default: http://localhost:11434/v1 (Ollama)
//! Override via OLLAMA_HOST env var or LocalProvider::with_base_url().
//! No API key required for local models (optional auth for remote deployments).

use super::openai_compat;
use super::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use std::future::Future;

pub struct LocalProvider {
    /// Optional API key for authenticated local deployments.
    api_key: Option<String>,
    base_url: String,
}

impl LocalProvider {
    /// Create a provider pointing at the default Ollama endpoint.
    pub fn new() -> Self {
        let host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".into());
        Self {
            api_key: None,
            base_url: format!("{}/v1", host.trim_end_matches('/')),
        }
    }

    /// Override the base URL (e.g., for llama.cpp server or remote Ollama).
    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Set an API key for authenticated local deployments.
    pub fn with_api_key(mut self, key: String) -> Self {
        self.api_key = Some(key);
        self
    }
}

impl Default for LocalProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmProvider for LocalProvider {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn auth_header(&self) -> (&str, &str) {
        // Ollama doesn't require auth by default
        ("Authorization", self.api_key.as_deref().unwrap_or(""))
    }

    fn call(
        &self,
        client: &reqwest::Client,
        request: &LlmRequest,
    ) -> impl Future<Output = Result<LlmResponse, LlmError>> + Send {
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone().unwrap_or_default();
        let req = request.clone();

        async move {
            let body = serde_json::json!({
                "model": req.model,
                "messages": req.messages.iter().map(|m| {
                    serde_json::json!({"role": m.role, "content": m.content})
                }).collect::<Vec<_>>(),
                "max_tokens": req.max_tokens,
                "temperature": req.temperature,
                "stream": false,
            });

            let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

            let mut builder = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body);

            if !api_key.is_empty() {
                builder = builder.header("Authorization", format!("Bearer {}", api_key));
            }

            let resp = builder.send().await.map_err(LlmError::Http)?;
            let resp = resp.error_for_status().map_err(LlmError::Http)?;
            let json: serde_json::Value = resp.json().await.map_err(LlmError::Http)?;

            openai_compat::parse_openai_response(&json, &req.model)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_default_base_url() {
        let provider = LocalProvider::new();
        assert!(provider.base_url().contains("localhost"));
        assert!(provider.base_url().ends_with("/v1"));
    }

    #[test]
    fn test_local_custom_base_url() {
        let provider = LocalProvider::new()
            .with_base_url("http://192.168.1.50:8080/v1".into());
        assert_eq!(provider.base_url(), "http://192.168.1.50:8080/v1");
    }

    #[test]
    fn test_local_no_auth_by_default() {
        let provider = LocalProvider::new();
        let (_, val) = provider.auth_header();
        assert!(val.is_empty());
    }
}
