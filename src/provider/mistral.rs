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
        _client: &reqwest::Client,
        _request: &LlmRequest,
    ) -> impl Future<Output = Result<LlmResponse, LlmError>> + Send {
        async move {
            Err(LlmError::ModelUnavailable(
                "mistral provider not yet implemented".into(),
            ))
        }
    }
}
