# Android AI App — Model Layer

## Supported Providers

| Provider | Base URL | Auth |
|---|---|---|
| OpenRouter | `https://openrouter.ai/api/v1` | Bearer API key |
| Mistral Direct | `https://api.mistral.ai/v1` | Bearer API key |
| Anthropic Direct | `https://api.anthropic.com/v1` | `x-api-key` header |
| Google Gemini | `https://generativelanguage.googleapis.com/v1beta/openai` | Bearer API key |
| OpenAI | `https://api.openai.com/v1` | Bearer API key |
| Local (llama.cpp) | `http://127.0.0.1:8080/v1` | None |
| Custom endpoint | User-defined | User-defined |

OpenRouter is the recommended default — a single key unlocks 300+ models.

## Tiered Model Routing

Automatically routes tasks to the cheapest model that can handle them. Don't burn Opus tokens on "set a timer."

```
TaskComplexity enum:
  Trivial   — heartbeats, status checks, simple reminders
  Standard  — everyday tasks, drafting, searching
  Complex   — multi-step planning, code, analysis
  Critical  — high-stakes: payments, sends, deletes
```

### Default Tier Configuration

| Tier | Default Model | Est. Cost/1M tokens |
|---|---|---|
| Trivial | `openrouter/google/gemini-flash-2.5` | ~$0.075 |
| Standard | `openrouter/mistralai/mistral-small-3.2` | ~$0.10 |
| Complex | `openrouter/anthropic/claude-sonnet-4-6` | ~$3.00 |
| Critical | `openrouter/anthropic/claude-opus-4-6` | ~$15.00 |

All tiers are user-editable with drag-to-reorder fallback chains in settings.

### Complexity Classifier (Zero-cost, rule-based in Rust)

```rust
fn classify_task(prompt: &str, context: &AgentContext) -> TaskComplexity {
    // Destructive keywords → Critical
    if contains_any(prompt, ["send", "delete", "pay", "transfer", "buy", "post"]) {
        return Critical;
    }
    // Code keywords → Complex
    if contains_any(prompt, ["code", "script", "write", "debug", "function"]) {
        return Complex;
    }
    // Multi-step indicators → Complex
    if contains_any(prompt, ["then", "after", "next", "finally", "step"]) {
        return Complex;
    }
    // Long prompts → Standard
    if prompt.word_count() > 15 { return Standard; }
    Trivial
}
```

## Model Fallback Chains

```rust
pub async fn call_with_fallback(request: &LlmRequest, tiers: &[ModelTier])
    -> Result<LlmResponse>
{
    let tier = select_tier(request.complexity, tiers);
    // Try primary, then each fallback in order
    for model in [tier.primary, ...tier.fallbacks] {
        match http_client::call(model, request).await {
            Ok(response) => return Ok(response),
            Err(RateLimit(retry_after)) => sleep(retry_after).await; continue,
            Err(ModelUnavailable) => continue,
            Err(e) => return Err(e),
        }
    }
    Err(AllFallbacksExhausted)
}
```

## Prompt Caching

For Anthropic and OpenRouter (on select models), send `cache_control` markers on system prompt and memory file. Cuts input costs 50–90% on cached tokens for an always-on agent.

```rust
fn build_messages_with_cache(system, memory, history, user_turn) {
    json!({
        "system": [
            { "type": "text", "text": system,
              "cache_control": { "type": "ephemeral" } },
            { "type": "text", "text": memory,
              "cache_control": { "type": "ephemeral" } }
        ],
        "messages": build_history(history, user_turn)
    })
}
```

## Settings: Auto Model

Option to use `openrouter/auto` for intelligent model routing — lets OpenRouter pick the best model per request instead of local tier selection.
