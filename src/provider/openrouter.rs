use super::{LlmError, LlmProvider, LlmRequest, LlmResponse, Usage};
use reqwest::StatusCode;

pub struct OpenRouterProvider {
    api_key: String,
    base_url: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
        }
    }
}

impl LlmProvider for OpenRouterProvider {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn auth_header(&self) -> (&str, &str) {
        ("Authorization", &self.api_key)
    }

    fn call(
        &self,
        client: &reqwest::Client,
        request: &LlmRequest,
    ) -> impl std::future::Future<Output = Result<LlmResponse, LlmError>> + Send {
        let api_key = self.api_key.clone();
        let base_url = self.base_url.clone();
        let request = request.clone();

        async move {
            let body = serde_json::json!({
                "model": request.model,
                "messages": request.messages.iter().map(|m| {
                    serde_json::json!({"role": m.role, "content": m.content})
                }).collect::<Vec<_>>(),
                "max_tokens": request.max_tokens,
                "temperature": request.temperature,
            });

            let resp = client
                .post(format!("{}/chat/completions", base_url))
                .header("Authorization", format!("Bearer {}", api_key))
                .header("HTTP-Referer", "http://localhost")
                .header("X-Title", "Android AI Agent")
                .json(&body)
                .send()
                .await?;

            let status = resp.status();

            // Handle rate limiting (429) before consuming the response
            if status == StatusCode::TOO_MANY_REQUESTS {
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30);
                return Err(LlmError::RateLimit(retry_after));
            }

            // Handle model unavailable (503)
            if status == StatusCode::SERVICE_UNAVAILABLE {
                return Err(LlmError::ModelUnavailable(request.model.clone()));
            }

            // Convert any other non-success status to an Http error
            let resp = resp.error_for_status()?;

            let json: serde_json::Value = resp.json().await?;
            let choices = json["choices"].as_array()
                .ok_or_else(|| LlmError::ModelUnavailable("empty response".into()))?;
            let choice = choices.first()
                .ok_or_else(|| LlmError::ModelUnavailable("no choices in response".into()))?;
            let message = &choice["message"];
            let usage = &json["usage"];

            Ok(LlmResponse {
                content: message["content"].as_str().unwrap_or("").to_string(),
                model: json["model"]
                    .as_str()
                    .unwrap_or(&request.model)
                    .to_string(),
                usage: Usage {
                    prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                    completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
                    total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as u32,
                },
            })
        }
    }
}
