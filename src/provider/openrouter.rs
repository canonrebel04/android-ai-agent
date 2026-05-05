use super::{LlmError, LlmProvider, LlmRequest, LlmResponse, Usage};
use crate::prompt_cache::{CacheBreakpoint, CacheableProvider, CachedRequest};
use futures_util::{Stream, StreamExt};
use reqwest::StatusCode;
use serde_json::json;
use std::pin::Pin;

pub struct OpenRouterProvider {
    api_key: String,
    base_url: String,
    /// Enable prompt caching passthrough for Anthropic models via OpenRouter.
    pub cache_enabled: bool,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
            cache_enabled: false,
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
            let choices = json["choices"]
                .as_array()
                .ok_or_else(|| LlmError::ModelUnavailable("empty response".into()))?;
            let choice = choices
                .first()
                .ok_or_else(|| LlmError::ModelUnavailable("no choices in response".into()))?;
            let message = &choice["message"];
            let usage = &json["usage"];

            Ok(LlmResponse {
                content: message["content"].as_str().unwrap_or("").to_string(),
                model: json["model"].as_str().unwrap_or(&request.model).to_string(),
                usage: Usage {
                    prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                    completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
                    total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as u32,
                },
            })
        }
    }

    fn call_stream(
        &self,
        client: &reqwest::Client,
        request: &LlmRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>> {
        let api_key = self.api_key.clone();
        let base_url = self.base_url.clone();
        let request = request.clone();
        let client = client.clone();

        Box::pin(async_stream::try_stream! {
            let body = serde_json::json!({
                "model": request.model,
                "messages": request.messages.iter().map(|m| {
                    serde_json::json!({"role": m.role, "content": m.content})
                }).collect::<Vec<_>>(),
                "max_tokens": request.max_tokens,
                "temperature": request.temperature,
                "stream": true,
            });

            let resp = client
                .post(format!("{}/chat/completions", base_url))
                .header("Authorization", format!("Bearer {}", api_key))
                .header("HTTP-Referer", "http://localhost")
                .header("X-Title", "Android AI Agent")
                .json(&body)
                .send()
                .await
                .map_err(LlmError::Http)?;

            let mut stream = resp.bytes_stream();
            while let Some(item) = stream.next().await {
                let chunk = item.map_err(LlmError::Http)?;
                let text = String::from_utf8_lossy(&chunk);

                for line in text.lines() {
                    if line.starts_with("data: ") {
                        let data = &line[6..];
                        if data == "[DONE]" {
                            break;
                        }
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                yield content.to_string();
                            }
                        }
                    }
                }
            }
        })
    }
}

impl CacheableProvider for OpenRouterProvider {
    fn apply_cache(
        &self,
        body: &serde_json::Value,
        breakpoints: &[CacheBreakpoint],
    ) -> CachedRequest {
        if !self.cache_enabled || breakpoints.is_empty() {
            return CachedRequest::default();
        }

        let mut body = body.clone();
        let messages = body.get_mut("messages").and_then(|m| m.as_array_mut());

        let Some(messages) = messages else {
            return CachedRequest::default();
        };

        let mark_cache_on = |msg: &mut serde_json::Value| {
            if let Some(content) = msg.get_mut("content") {
                if let Some(arr) = content.as_array_mut() {
                    for block in arr.iter_mut() {
                        if let Some(obj) = block.as_object_mut() {
                            obj.insert("cache_control".to_string(), json!({"type": "ephemeral"}));
                        }
                    }
                }
            }
        };

        for bp in breakpoints {
            match bp {
                CacheBreakpoint::SystemMessages => {
                    if let Some(first) = messages.first_mut() {
                        mark_cache_on(first);
                    }
                }
                CacheBreakpoint::LastMessages(n) => {
                    let len = messages.len();
                    let start = len.saturating_sub(*n);
                    for msg in messages.iter_mut().skip(start) {
                        mark_cache_on(msg);
                    }
                }
                CacheBreakpoint::AtMessage(idx) => {
                    if *idx < messages.len() {
                        mark_cache_on(&mut messages[*idx]);
                    }
                }
            }
        }

        CachedRequest {
            cache_enabled: true,
            modified_body: Some(body),
            extra_headers: vec![(
                "anthropic-beta".to_string(),
                "prompt-caching-2024-07-31".to_string(),
            )],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_openrouter_cache_last_messages() {
        let provider = OpenRouterProvider {
            api_key: "test".into(),
            base_url: "https://test".into(),
            cache_enabled: true,
        };
        let body = json!({
            "model": "anthropic/claude-sonnet-4",
            "messages": [
                {"role": "user", "content": [{"type": "text", "text": "a"}]},
                {"role": "assistant", "content": [{"type": "text", "text": "b"}]},
            ]
        });
        let result = provider.apply_cache(&body, &[CacheBreakpoint::LastMessages(1)]);
        assert!(result.cache_enabled);
        assert!(result
            .extra_headers
            .iter()
            .any(|(k, _v)| k == "anthropic-beta"));
    }

    #[test]
    fn test_openrouter_cache_disabled_skips() {
        let provider = OpenRouterProvider {
            api_key: "test".into(),
            base_url: "https://test".into(),
            cache_enabled: false,
        };
        let body = json!({"messages": []});
        let result = provider.apply_cache(&body, &[CacheBreakpoint::LastMessages(1)]);
        assert!(!result.cache_enabled);
    }
}
