use crate::complexity_classifier::{self, TaskComplexity};
use crate::http_client::HttpClient;
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
                primary: "openrouter/google/gemini-flash-2.5".into(),
                fallbacks: vec!["openrouter/mistralai/mistral-small-3.2".into()],
                max_tokens: 1024,
                temperature: 0.3,
            },
            ModelTier {
                complexity: TaskComplexity::Standard,
                primary: "openrouter/mistralai/mistral-small-3.2".into(),
                fallbacks: vec!["openrouter/google/gemini-flash-2.5".into()],
                max_tokens: 4096,
                temperature: 0.5,
            },
            ModelTier {
                complexity: TaskComplexity::Complex,
                primary: "openrouter/anthropic/claude-sonnet-4-6".into(),
                fallbacks: vec![
                    "openrouter/mistralai/mistral-small-3.2".into(),
                    "openrouter/google/gemini-flash-2.5".into(),
                ],
                max_tokens: 8192,
                temperature: 0.7,
            },
            ModelTier {
                complexity: TaskComplexity::Critical,
                primary: "openrouter/anthropic/claude-opus-4-6".into(),
                fallbacks: vec!["openrouter/anthropic/claude-sonnet-4-6".into()],
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
}
