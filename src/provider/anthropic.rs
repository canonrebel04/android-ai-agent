use super::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use crate::prompt_cache::{CacheBreakpoint, CacheableProvider, CachedRequest};
use serde_json::json;
use std::future::Future;

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
                    // Anthropic system message is top-level, not in messages array.
                    // We mark it by adding cache_control to the first message.
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
