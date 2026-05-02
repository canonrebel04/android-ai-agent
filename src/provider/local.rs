use super::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use std::future::Future;

pub struct LocalProvider;

impl LocalProvider {
    pub fn new() -> Self {
        Self
    }
}

impl LlmProvider for LocalProvider {
    fn base_url(&self) -> &str {
        "http://127.0.0.1:8080/v1"
    }

    fn auth_header(&self) -> (&str, &str) {
        ("", "")
    }

    fn call(
        &self,
        _client: &reqwest::Client,
        _request: &LlmRequest,
    ) -> impl Future<Output = Result<LlmResponse, LlmError>> + Send {
        async move {
            Err(LlmError::ModelUnavailable(
                "local provider not yet implemented".into(),
            ))
        }
    }
}
