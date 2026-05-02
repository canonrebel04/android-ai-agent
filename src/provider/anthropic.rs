use super::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use std::future::Future;

pub struct AnthropicProvider {
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl LlmProvider for AnthropicProvider {
    fn base_url(&self) -> &str {
        "https://api.anthropic.com/v1"
    }

    fn auth_header(&self) -> (&str, &str) {
        ("x-api-key", &self.api_key)
    }

    fn call(
        &self,
        _client: &reqwest::Client,
        _request: &LlmRequest,
    ) -> impl Future<Output = Result<LlmResponse, LlmError>> + Send {
        async move {
            Err(LlmError::ModelUnavailable(
                "anthropic provider not yet implemented".into(),
            ))
        }
    }
}
