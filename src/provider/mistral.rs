//! Mistral AI provider — OpenAI-compatible API.
//! Base URL: https://api.mistral.ai/v1
//! Auth: Bearer token
//! Models: mistral-large, mistral-small, mistral-small-3.2, codestral, etc.

use super::openai_compat;
use super::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use std::future::Future;

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
        let req = request.clone();

        async move {
            openai_compat::openai_compat_call(client, &base_url, &api_key, &req).await
        }
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
