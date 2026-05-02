use super::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use std::future::Future;

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
        _client: &reqwest::Client,
        _request: &LlmRequest,
    ) -> impl Future<Output = Result<LlmResponse, LlmError>> + Send {
        async move {
            Err(LlmError::ModelUnavailable(
                "google provider not yet implemented".into(),
            ))
        }
    }
}
