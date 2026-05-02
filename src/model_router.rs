use crate::complexity_classifier::{self, TaskComplexity};
use crate::http_client::HttpClient;
use crate::prompt_cache::{CacheBreakpoint, CacheableProvider, CachedRequest};
use crate::provider::{LlmError, LlmProvider, LlmRequest, LlmResponse, Message};

#[derive(Debug, Clone)]
pub struct ModelTier {
    pub complexity: TaskComplexity,
    pub primary: String,
    pub fallbacks: Vec<String>,
    pub max_tokens: u32,
    pub temperature: f32,
}

pub struct ModelRouter {
    tiers: Vec<ModelTier>,
}

impl ModelRouter {
    pub fn new(tiers: Vec<ModelTier>) -> Self {
        Self { tiers }
    }

    pub fn default_tiers() -> Vec<ModelTier> {
        vec![
            ModelTier {
                complexity: TaskComplexity::Trivial,
                primary: "deepseek/deepseek-v4-flash".into(),
                fallbacks: vec![
                    "openrouter/google/gemini-flash-2.5".into(),
                    "openrouter/mistralai/mistral-small-3.2".into(),
                ],
                max_tokens: 2048,
                temperature: 0.3,
            },
            ModelTier {
                complexity: TaskComplexity::Standard,
                primary: "openrouter/mistralai/mistral-small-3.2".into(),
                fallbacks: vec![
                    "openrouter/google/gemini-flash-2.5".into(),
                    "deepseek/deepseek-v4-flash".into(),
                ],
                max_tokens: 4096,
                temperature: 0.5,
            },
            ModelTier {
                complexity: TaskComplexity::Complex,
                primary: "openrouter/anthropic/claude-sonnet-4-6".into(),
                fallbacks: vec![
                    "openrouter/deepseek/deepseek-v4-pro".into(),
                    "openrouter/mistralai/mistral-small-3.2".into(),
                ],
                max_tokens: 8192,
                temperature: 0.7,
            },
            ModelTier {
                complexity: TaskComplexity::Critical,
                primary: "openrouter/anthropic/claude-opus-4-6".into(),
                fallbacks: vec![
                    "openrouter/anthropic/claude-sonnet-4-6".into(),
                    "openrouter/deepseek/deepseek-v4-pro".into(),
                ],
                max_tokens: 4096,
                temperature: 0.1,
            },
        ]
    }

    pub fn select_tier(&self, complexity: TaskComplexity) -> &ModelTier {
        self.tiers
            .iter()
            .find(|t| t.complexity == complexity)
            .unwrap_or_else(|| &self.tiers[0])
    }

    pub async fn call_with_fallback<P: LlmProvider>(
        &self,
        client: &HttpClient,
        provider: &P,
        prompt: &str,
        system_prompt: &str,
    ) -> Result<LlmResponse, LlmError> {
        let complexity = complexity_classifier::classify(prompt);
        let tier = self.select_tier(complexity);

        let mut all_models = vec![tier.primary.clone()];
        all_models.extend(tier.fallbacks.iter().cloned());

        let mut last_error = None;

        for model in &all_models {
            let request = LlmRequest {
                model: model.clone(),
                messages: vec![
                    Message {
                        role: "system".into(),
                        content: system_prompt.to_string(),
                    },
                    Message {
                        role: "user".into(),
                        content: prompt.to_string(),
                    },
                ],
                max_tokens: tier.max_tokens,
                temperature: tier.temperature,
            };

            match provider.call(client.inner(), &request).await {
                Ok(response) => return Ok(response),
                Err(LlmError::RateLimit(_)) | Err(LlmError::ModelUnavailable(_)) => {
                    last_error = Some(LlmError::AllFallbacksExhausted);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap_or(LlmError::AllFallbacksExhausted))
    }

    /// Like call_with_fallback but applies prompt caching breakpoints.
    /// Requires the provider to implement CacheableProvider (Anthropic/OpenRouter).
    pub async fn call_with_fallback_cached<P: LlmProvider + CacheableProvider>(
        &self,
        client: &HttpClient,
        provider: &P,
        prompt: &str,
        system_prompt: &str,
        cache_breakpoints: &[CacheBreakpoint],
    ) -> Result<(LlmResponse, Option<CachedRequest>), LlmError> {
        let complexity = complexity_classifier::classify(prompt);
        let tier = self.select_tier(complexity);
        let tier_name = complexity; // for budget tracking

        let mut all_models = vec![tier.primary.clone()];
        all_models.extend(tier.fallbacks.iter().cloned());

        let mut last_error = None;

        for model in &all_models {
            let request = LlmRequest {
                model: model.clone(),
                messages: vec![
                    Message {
                        role: "system".into(),
                        content: system_prompt.to_string(),
                    },
                    Message {
                        role: "user".into(),
                        content: prompt.to_string(),
                    },
                ],
                max_tokens: tier.max_tokens,
                temperature: tier.temperature,
            };

            // Build request body (same shape as OpenRouter's JSON)
            let body = serde_json::json!({
                "model": request.model,
                "messages": request.messages.iter().map(|m| {
                    serde_json::json!({"role": m.role, "content": m.content})
                }).collect::<Vec<_>>(),
                "max_tokens": request.max_tokens,
                "temperature": request.temperature,
            });

            // Apply cache breakpoints
            let cached = provider.apply_cache(&body, cache_breakpoints);

            // Build headers
            let (auth_name, auth_value) = provider.auth_header();
            let mut headers: Vec<(String, String)> = vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                (auth_name.to_string(), auth_value.to_string()),
            ];
            headers.extend(cached.extra_headers.clone());

            // Send request through http_client with cache support
            let url = format!("{}/chat/completions", provider.base_url());
            match client.send_cached(
                url,
                headers,
                body,
                cached.modified_body,
                cached.extra_headers.clone(),
            ).await {
                Ok(resp) => {
                    let resp = resp.error_for_status()
                        .map_err(LlmError::Http)?;
                    let json: serde_json::Value = resp.json().await
                        .map_err(LlmError::Http)?;

                    // Parse OpenRouter-compatible response
                    let choices = json["choices"].as_array()
                        .ok_or_else(|| LlmError::ModelUnavailable("empty response".into()))?;
                    let choice = choices.first()
                        .ok_or_else(|| LlmError::ModelUnavailable("no choices".into()))?;
                    let message = &choice["message"];
                    let content = message["content"].as_str().unwrap_or("").to_string();
                    let model_name = json["model"].as_str().unwrap_or(model).to_string();
                    let usage_json = &json["usage"];

                    let response = LlmResponse {
                        content,
                        model: model_name,
                        usage: crate::provider::Usage {
                            prompt_tokens: usage_json["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                            completion_tokens: usage_json["completion_tokens"].as_u64().unwrap_or(0) as u32,
                            total_tokens: usage_json["total_tokens"].as_u64().unwrap_or(0) as u32,
                        },
                    };
                    let _ = tier_name; // used for budget tracking in agent_loop
                    return Ok((response, None));
                }
                Err(e) => {
                    last_error = Some(LlmError::Http(e));
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or(LlmError::AllFallbacksExhausted))
    }
}
