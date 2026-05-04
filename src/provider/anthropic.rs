//! Anthropic direct provider — native Anthropic Messages API.
//!
//! API: POST https://api.anthropic.com/v1/messages
//! Auth: x-api-key header
//! Version: anthropic-version: 2023-06-01
//! Beta: anthropic-beta: prompt-caching-2024-07-31 (when caching enabled)
//!
//! Also implements CacheableProvider for prompt caching support.
//! Model auto-mapping: unsupported model names fall back to claude-sonnet-4-20250514.

use crate::prompt_cache::{CacheBreakpoint, CacheableProvider, CachedRequest};
use super::{LlmError, LlmProvider, LlmRequest, LlmResponse, Usage};
use serde_json::json;
use std::future::Future;
use futures_util::{Stream, StreamExt};
use std::pin::Pin;

const ANTHROPIC_VERSION: &str = "2023-06-01";

pub struct AnthropicProvider {
    api_key: String,
    /// Enable prompt caching (reduces costs ~90% for cached tokens).
    pub cache_enabled: bool,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            cache_enabled: false,
        }
    }
}

impl CacheableProvider for AnthropicProvider {
    fn apply_cache(
        &self,
        body: &serde_json::Value,
        breakpoints: &[CacheBreakpoint],
    ) -> CachedRequest {
        if !self.cache_enabled || breakpoints.is_empty() {
            return CachedRequest::default();
        }

        let mut body = body.clone();
        let messages = body
            .get_mut("messages")
            .and_then(|m| m.as_array_mut());

        let Some(messages) = messages else {
            return CachedRequest::default();
        };

        let mark_cache_on = |msg: &mut serde_json::Value| {
            if let Some(content) = msg.get_mut("content") {
                if let Some(arr) = content.as_array_mut() {
                    for block in arr.iter_mut() {
                        if let Some(obj) = block.as_object_mut() {
                            obj.insert(
                                "cache_control".to_string(),
                                json!({"type": "ephemeral"}),
                            );
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
            extra_headers: vec![
                (
                    "anthropic-version".to_string(),
                    ANTHROPIC_VERSION.to_string(),
                ),
                (
                    "anthropic-beta".to_string(),
                    "prompt-caching-2024-07-31".to_string(),
                ),
            ],
        }
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
        client: &reqwest::Client,
        request: &LlmRequest,
    ) -> impl Future<Output = Result<LlmResponse, LlmError>> + Send {
        let api_key = self.api_key.clone();
        let cache_enabled = self.cache_enabled;
        let req = request.clone();

        async move {
            // Convert to Anthropic Messages format
            let mut messages: Vec<serde_json::Value> = Vec::new();
            for msg in &req.messages {
                if msg.role == "system" {
                    // System message handled separately below
                    continue;
                }
                messages.push(serde_json::json!({
                    "role": msg.role,
                    "content": [{"type": "text", "text": msg.content}]
                }));
            }

            // Find system message
            let system = req.messages.iter()
                .find(|m| m.role == "system")
                .map(|m| m.content.clone());

            let mut body = serde_json::json!({
                "model": req.model,
                "max_tokens": req.max_tokens,
                "messages": messages,
            });

            if let Some(ref sys) = system {
                body["system"] = json!(sys);
            }

            if req.temperature > 0.0 {
                body["temperature"] = json!(req.temperature);
            }

            // Apply caching if enabled
            let mut headers = vec![
                ("x-api-key".to_string(), api_key.clone()),
                ("anthropic-version".to_string(), ANTHROPIC_VERSION.to_string()),
                ("Content-Type".to_string(), "application/json".to_string()),
            ];

            if cache_enabled {
                headers.push(("anthropic-beta".to_string(), "prompt-caching-2024-07-31".to_string()));

                // Add cache_control markers to the last 4 messages
                if let Some(arr) = body["messages"].as_array_mut() {
                    let len = arr.len();
                    let start = len.saturating_sub(4);
                    for msg in arr.iter_mut().skip(start) {
                        if let Some(content) = msg.get_mut("content") {
                            if let Some(blocks) = content.as_array_mut() {
                                for block in blocks.iter_mut() {
                                    if let Some(obj) = block.as_object_mut() {
                                        obj.insert("cache_control".to_string(), json!({"type": "ephemeral"}));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let url = format!("{}/messages", AnthropicProvider::new(api_key.clone()).base_url());

            let mut req_builder = client.post(&url).json(&body);
            for (k, v) in &headers {
                req_builder = req_builder.header(k.as_str(), v.as_str());
            }

            let resp = req_builder.send().await.map_err(LlmError::Http)?;
            let resp = resp.error_for_status().map_err(LlmError::Http)?;
            let json: serde_json::Value = resp.json().await.map_err(LlmError::Http)?;

            // Parse Anthropic response
            let content_blocks = json["content"].as_array()
                .ok_or_else(|| LlmError::ModelUnavailable("no content in response".into()))?;

            let text: String = content_blocks.iter()
                .filter_map(|block| block["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n");

            let model_name = json["model"].as_str().unwrap_or(&req.model).to_string();
            let usage_json = &json["usage"];

            Ok(LlmResponse {
                content: text,
                model: model_name,
                usage: Usage {
                    prompt_tokens: usage_json["input_tokens"].as_u64().unwrap_or(0) as u32,
                    completion_tokens: usage_json["output_tokens"].as_u64().unwrap_or(0) as u32,
                    total_tokens: (usage_json["input_tokens"].as_u64().unwrap_or(0)
                        + usage_json["output_tokens"].as_u64().unwrap_or(0)) as u32,
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
        let req = request.clone();
        let client = client.clone();

        Box::pin(async_stream::try_stream! {
            let mut messages: Vec<serde_json::Value> = Vec::new();
            for msg in &req.messages {
                if msg.role == "system" { continue; }
                messages.push(serde_json::json!({
                    "role": msg.role,
                    "content": [{"type": "text", "text": msg.content}]
                }));
            }

            let system = req.messages.iter()
                .find(|m| m.role == "system")
                .map(|m| m.content.clone());

            let mut body = serde_json::json!({
                "model": req.model,
                "max_tokens": req.max_tokens,
                "messages": messages,
                "stream": true,
            });

            if let Some(ref sys) = system { body["system"] = json!(sys); }
            if req.temperature > 0.0 { body["temperature"] = json!(req.temperature); }

            let url = format!("{}/messages", "https://api.anthropic.com/v1");
            let resp = client.post(&url)
                .header("x-api-key", api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .header("Content-Type", "application/json")
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
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(type_name) = json["type"].as_str() {
                                if type_name == "content_block_delta" {
                                    if let Some(delta_text) = json["delta"]["text"].as_str() {
                                        yield delta_text.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cache_last_messages() {
        let provider = AnthropicProvider {
            api_key: "test".into(),
            cache_enabled: true,
        };
        let body = json!({
            "model": "claude-sonnet-4-20250514",
            "messages": [
                {"role": "user", "content": [{"type": "text", "text": "a"}]},
                {"role": "assistant", "content": [{"type": "text", "text": "b"}]},
                {"role": "user", "content": [{"type": "text", "text": "c"}]},
            ]
        });
        let result = provider.apply_cache(&body, &[CacheBreakpoint::LastMessages(2)]);
        assert!(result.cache_enabled);
        assert!(result
            .extra_headers
            .iter()
            .any(|(k, _v)| k == "anthropic-beta"));
        let modified = result.modified_body.unwrap();
        let msgs = modified["messages"].as_array().unwrap();
        let last = &msgs[2];
        let content = last["content"].as_array().unwrap();
        let block = &content[0];
        assert!(block.get("cache_control").is_some());
    }

    #[test]
    fn test_cache_disabled_skips() {
        let provider = AnthropicProvider {
            api_key: "test".into(),
            cache_enabled: false,
        };
        let body = json!({"messages": []});
        let result = provider.apply_cache(&body, &[CacheBreakpoint::LastMessages(1)]);
        assert!(!result.cache_enabled);
    }

    #[test]
    fn test_empty_breakpoints_skips() {
        let provider = AnthropicProvider {
            api_key: "test".into(),
            cache_enabled: true,
        };
        let body = json!({"messages": [{"role": "user", "content": [{"type": "text", "text": "hi"}]}]});
        let result = provider.apply_cache(&body, &[]);
        assert!(!result.cache_enabled);
    }

    #[test]
    fn test_system_message_cache() {
        let provider = AnthropicProvider {
            api_key: "test".into(),
            cache_enabled: true,
        };
        let body = json!({
            "messages": [
                {"role": "user", "content": [{"type": "text", "text": "hello"}]},
                {"role": "assistant", "content": [{"type": "text", "text": "hi"}]},
            ]
        });
        let result = provider.apply_cache(&body, &[CacheBreakpoint::SystemMessages]);
        assert!(result.cache_enabled);
        let modified = result.modified_body.unwrap();
        let first_msg = &modified["messages"].as_array().unwrap()[0];
        let block = &first_msg["content"].as_array().unwrap()[0];
        assert!(block.get("cache_control").is_some());
    }
}
