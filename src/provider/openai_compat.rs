//! Shared OpenAI-compatible API call implementation.
//! Used by DeepSeek, Mistral, Google Gemini (via OpenRouter format),
//! Ollama, and any provider that speaks the OpenAI chat completions protocol.
//!
//! Request format:
//!   POST {base_url}/chat/completions
//!   Authorization: Bearer {api_key}
//!   Content-Type: application/json
//!   { "model": "...", "messages": [...], "max_tokens": N, "temperature": 0.5 }
//!
//! Response format:
//!   { "choices": [{ "message": { "role": "assistant", "content": "..." } }],
//!     "model": "...",
//!     "usage": { "prompt_tokens": N, "completion_tokens": N, "total_tokens": N } }

use super::{LlmError, LlmRequest, LlmResponse, Usage};
use reqwest::StatusCode;

/// Make an OpenAI-compatible chat completion call.
pub async fn openai_compat_call(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    request: &LlmRequest,
) -> Result<LlmResponse, LlmError> {
    let body = serde_json::json!({
        "model": request.model,
        "messages": request.messages.iter().map(|m| {
            serde_json::json!({"role": m.role, "content": m.content})
        }).collect::<Vec<_>>(),
        "max_tokens": request.max_tokens,
        "temperature": request.temperature,
    });

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(LlmError::Http)?;

    let status = resp.status();

    if status == StatusCode::TOO_MANY_REQUESTS {
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);
        return Err(LlmError::RateLimit(retry_after));
    }

    if status == StatusCode::SERVICE_UNAVAILABLE {
        return Err(LlmError::ModelUnavailable(request.model.clone()));
    }

    let resp = resp.error_for_status().map_err(LlmError::Http)?;
    let json: serde_json::Value = resp.json().await.map_err(LlmError::Http)?;

    parse_openai_response(&json, &request.model)
}

/// Parse the standard OpenAI chat completion response JSON.
pub fn parse_openai_response(
    json: &serde_json::Value,
    fallback_model: &str,
) -> Result<LlmResponse, LlmError> {
    let choices = json["choices"]
        .as_array()
        .ok_or_else(|| LlmError::ModelUnavailable("empty response".into()))?;

    let choice = choices
        .first()
        .ok_or_else(|| LlmError::ModelUnavailable("no choices in response".into()))?;

    let message = &choice["message"];
    let content = message["content"].as_str().unwrap_or("").to_string();

    // DeepSeek: extract reasoning_content if present (thinking mode)
    let reasoning = message["reasoning_content"].as_str().map(|s| s.to_string());

    let model_name = json["model"]
        .as_str()
        .unwrap_or(fallback_model)
        .to_string();

    let usage = &json["usage"];

    // Combine reasoning + content for thinking-mode models
    let full_content = if let Some(ref r) = reasoning {
        if r.is_empty() {
            content
        } else {
            format!("[thinking]\n{}\n[/thinking]\n{}", r, content)
        }
    } else {
        content
    };

    Ok(LlmResponse {
        content: full_content,
        model: model_name,
        usage: Usage {
            prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as u32,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_openai_response_standard() {
        let json = serde_json::json!({
            "choices": [{
                "message": {"role": "assistant", "content": "Hello!"}
            }],
            "model": "test-model",
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
        });
        let resp = parse_openai_response(&json, "fallback").unwrap();
        assert_eq!(resp.content, "Hello!");
        assert_eq!(resp.model, "test-model");
        assert_eq!(resp.usage.prompt_tokens, 10);
        assert_eq!(resp.usage.completion_tokens, 5);
    }

    #[test]
    fn test_parse_reasoning_content() {
        let json = serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "The answer is 42.",
                    "reasoning_content": "Let me think about this..."
                }
            }],
            "model": "deepseek-v4-pro",
            "usage": {"prompt_tokens": 5, "completion_tokens": 3, "total_tokens": 8}
        });
        let resp = parse_openai_response(&json, "fallback").unwrap();
        assert!(resp.content.contains("[thinking]"));
        assert!(resp.content.contains("The answer is 42"));
    }
}
