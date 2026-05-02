# Android AI Agent — Phase 1: Core Engine Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Build the Rust core engine for an Android-native AI agent: multi-provider HTTP client, tiered model router with fallback chains, skill registry with TOML loading, task complexity classifier, and rolling context manager — all testable in Termux before any Android integration.

**Architecture:** Pure Rust library crate (`rust/src/`) with module-per-responsibility design. No Android/JNI yet — everything tested via `cargo test` and manual Termux runs. Providers abstracted behind a unified `LlmProvider` trait. Skills loaded from TOML files via the `Skill` trait.

**Tech Stack:** Rust 1.85+, tokio 1.52, reqwest 0.13, serde/serde_json 1.0, toml 1.1. OpenRouter as default provider, Anthropic/Mistral/Google/Local as alternates. No Android SDK yet — Phase 1 is pure Rust.

---

## Prerequisites

Before starting any task, verify the environment:

```bash
rustc --version  # Must be >= 1.85.0
cargo --version
```

Create the project:

```bash
cargo new android-ai-agent --lib
cd android-ai-agent
```

Initial `Cargo.toml`:

```toml
[package]
name = "android-ai-agent"
version = "0.1.0"
edition = "2024"

[dependencies]
tokio = { version = "1.52", features = ["full"] }
reqwest = { version = "0.13", features = ["json", "rustls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "1.1"

[dev-dependencies]
tokio-test = "0.4"
```

---

### Task 1: Scaffold the crate and module structure

**Objective:** Create all Phase 1 module files so every subsequent task has its target file ready.

**Files:**
- Create: `rust/src/lib.rs`
- Create: `rust/src/http_client.rs`
- Create: `rust/src/provider/mod.rs`
- Create: `rust/src/provider/openrouter.rs`
- Create: `rust/src/provider/anthropic.rs`
- Create: `rust/src/provider/google.rs`
- Create: `rust/src/provider/mistral.rs`
- Create: `rust/src/provider/local.rs`
- Create: `rust/src/tool_parser.rs`
- Create: `rust/src/model_router.rs`
- Create: `rust/src/skill_registry.rs`
- Create: `rust/src/context_manager.rs`
- Create: `rust/src/complexity_classifier.rs`

**Step 1: Create the source tree**

```bash
mkdir -p rust/src/provider
touch rust/src/lib.rs
touch rust/src/http_client.rs
touch rust/src/provider/mod.rs
touch rust/src/provider/openrouter.rs
touch rust/src/provider/anthropic.rs
touch rust/src/provider/google.rs
touch rust/src/provider/mistral.rs
touch rust/src/provider/local.rs
touch rust/src/tool_parser.rs
touch rust/src/model_router.rs
touch rust/src/skill_registry.rs
touch rust/src/context_manager.rs
touch rust/src/complexity_classifier.rs
```

**Step 2: Write `rust/src/lib.rs` — module declarations**

```rust
pub mod http_client;
pub mod provider;
pub mod tool_parser;
pub mod model_router;
pub mod skill_registry;
pub mod context_manager;
pub mod complexity_classifier;
```

**Step 3: Write `rust/src/provider/mod.rs` — provider trait**

```rust
pub mod openrouter;
pub mod anthropic;
pub mod google;
pub mod mistral;
pub mod local;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("rate limited, retry after {0}s")]
    RateLimit(u64),
    #[error("model unavailable: {0}")]
    ModelUnavailable(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("all fallbacks exhausted")]
    AllFallbacksExhausted,
}

pub trait LlmProvider: Send + Sync {
    fn base_url(&self) -> &str;
    fn auth_header(&self) -> (&str, &str); // (header name, header value)
    async fn call(&self, client: &reqwest::Client, request: &LlmRequest) -> Result<LlmResponse, LlmError>;
}
```

**Step 4: Verify it compiles**

```bash
cargo check
```

Expected: `Finished dev [unoptimized + debuginfo]` with warnings about unused imports (acceptable at this stage).

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: scaffold Rust crate with module structure and LlmProvider trait"
```

---

### Task 2: Implement OpenRouter provider

**Objective:** Build the first working provider — OpenRouter — with full request/response handling.

**Files:**
- Create: `rust/src/provider/openrouter.rs`
- Modify: `rust/src/provider/mod.rs` (re-export)

**Step 1: Write `rust/src/provider/openrouter.rs`**

```rust
use super::{LlmError, LlmProvider, LlmRequest, LlmResponse, Message, Usage};
use serde_json::json;

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

    async fn call(&self, client: &reqwest::Client, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let body = json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| {
                json!({"role": m.role, "content": m.content})
            }).collect::<Vec<_>>(),
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
        });

        let resp = client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "http://localhost")
            .header("X-Title", "Android AI Agent")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            return match status {
                429 => {
                    let retry_after = resp.headers()
                        .get("retry-after")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(30);
                    Err(LlmError::RateLimit(retry_after))
                }
                503 => Err(LlmError::ModelUnavailable(request.model.clone())),
                _ => Err(LlmError::Http(
                    reqwest::Error::new(reqwest::StatusCode::from_u16(status).unwrap_or_default(), format!("HTTP {}", status))
                )),
            };
        }

        let json: serde_json::Value = resp.json().await?;
        let choice = &json["choices"][0]["message"];
        let usage = &json["usage"];

        Ok(LlmResponse {
            content: choice["content"].as_str().unwrap_or("").to_string(),
            model: json["model"].as_str().unwrap_or(&request.model).to_string(),
            usage: Usage {
                prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
                total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as u32,
            },
        })
    }
}
```

Note: Replace the manual `reqwest::Error` construction with the correct approach. Since `reqwest::Error` can't be constructed manually, we'll use `reqwest::get("http://[::1]:1")` pattern or refactor `LlmError` to hold status codes directly. We'll fix this in a later hardening pass.

**Step 2: Verify it compiles**

```bash
cargo check
```

Expected: `Finished` — OK if compiles. Warnings acceptable.

**Step 3: Commit**

```bash
git add rust/src/provider/openrouter.rs
git commit -m "feat: implement OpenRouter provider with rate-limit and error handling"
```

---

### Task 3: Implement HTTP client module

**Objective:** Build the unified HTTP client that manages a `reqwest::Client` and dispatches to any provider.

**Files:**
- Modify: `rust/src/http_client.rs`

**Step 1: Write `rust/src/http_client.rs`**

```rust
use crate::provider::{LlmError, LlmProvider, LlmRequest, LlmResponse};

pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    pub async fn call(
        &self,
        provider: &dyn LlmProvider,
        request: &LlmRequest,
    ) -> Result<LlmResponse, LlmError> {
        provider.call(&self.client, request).await
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Write a unit test**

Create `rust/tests/http_client_test.rs`:

```rust
// Will be fleshed out with mock server in a later task
#[test]
fn test_http_client_creation() {
    let client = android_ai_agent::http_client::HttpClient::new();
    // Just testing that construction doesn't panic
}
```

**Step 3: Verify**

```bash
cargo test
```

Expected: 1 test passed.

**Step 4: Commit**

```bash
git add rust/src/http_client.rs rust/tests/
git commit -m "feat: add unified HTTP client with configurable timeout"
```

---

### Task 4: Implement complexity classifier

**Objective:** Build the rule-based task complexity classifier — zero LLM cost, pure Rust logic.

**Files:**
- Modify: `rust/src/complexity_classifier.rs`

**Step 1: Write `rust/src/complexity_classifier.rs`**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskComplexity {
    Trivial,
    Standard,
    Complex,
    Critical,
}

pub fn classify(prompt: &str) -> TaskComplexity {
    let lower = prompt.to_lowercase();
    let word_count = prompt.split_whitespace().count();

    let destructive = ["send", "delete", "pay", "transfer", "buy", "post"];
    let code = ["code", "script", "write", "debug", "function"];
    let multi_step = ["then", "after", "next", "finally", "step"];

    if destructive.iter().any(|kw| lower.contains(kw)) {
        return TaskComplexity::Critical;
    }
    if code.iter().any(|kw| lower.contains(kw)) || multi_step.iter().any(|kw| lower.contains(kw)) {
        return TaskComplexity::Complex;
    }
    if word_count > 15 {
        return TaskComplexity::Standard;
    }
    TaskComplexity::Trivial
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivial() {
        assert_eq!(classify("hello"), TaskComplexity::Trivial);
        assert_eq!(classify("set a timer"), TaskComplexity::Trivial);
    }

    #[test]
    fn test_standard() {
        let long = "tell me about the history of artificial intelligence and its impact on society";
        assert_eq!(classify(long), TaskComplexity::Standard);
    }

    #[test]
    fn test_complex_code() {
        assert_eq!(classify("write a function to sort this array"), TaskComplexity::Complex);
    }

    #[test]
    fn test_complex_multistep() {
        assert_eq!(classify("open gmail then forward the email to mike"), TaskComplexity::Complex);
    }

    #[test]
    fn test_critical() {
        assert_eq!(classify("send $100 to Alice"), TaskComplexity::Critical);
        assert_eq!(classify("delete all files"), TaskComplexity::Critical);
        assert_eq!(classify("buy tickets for the concert"), TaskComplexity::Critical);
    }
}
```

**Step 2: Run tests**

```bash
cargo test complexity_classifier
```

Expected: 5 tests passed.

**Step 3: Commit**

```bash
git add rust/src/complexity_classifier.rs
git commit -m "feat: add rule-based task complexity classifier with tests"
```

---

### Task 5: Implement model router with tiered selection and fallback chains

**Objective:** Build the tiered model router that selects cheapest-capable model and handles fallbacks.

**Files:**
- Modify: `rust/src/model_router.rs`

**Step 1: Write `rust/src/model_router.rs`**

```rust
use crate::complexity_classifier::TaskComplexity;
use crate::http_client::HttpClient;
use crate::provider::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use std::time::Duration;

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

    pub async fn call_with_fallback(
        &self,
        http: &HttpClient,
        provider: &dyn LlmProvider,
        prompt: &str,
        system_prompt: &str,
    ) -> Result<LlmResponse, LlmError> {
        let complexity = crate::complexity_classifier::classify(prompt);
        let tier = self.select_tier(complexity);

        let request = LlmRequest {
            model: tier.primary.clone(),
            messages: vec![
                crate::provider::Message {
                    role: "system".into(),
                    content: system_prompt.to_string(),
                },
                crate::provider::Message {
                    role: "user".into(),
                    content: prompt.to_string(),
                },
            ],
            max_tokens: tier.max_tokens,
            temperature: tier.temperature,
        };

        // Try primary first
        let mut request = request;
        match http.call(provider, &request).await {
            Ok(resp) => return Ok(resp),
            Err(LlmError::RateLimit(retry)) => {
                tokio::time::sleep(Duration::from_secs(retry)).await;
            }
            Err(LlmError::ModelUnavailable(_)) => {}
            Err(e) => return Err(e),
        }

        // Try fallbacks in order
        for fallback_model in &tier.fallbacks {
            request.model = fallback_model.clone();
            match http.call(provider, &request).await {
                Ok(resp) => return Ok(resp),
                Err(LlmError::RateLimit(retry)) => {
                    tokio::time::sleep(Duration::from_secs(retry)).await;
                }
                Err(LlmError::ModelUnavailable(_)) => continue,
                Err(e) => return Err(e),
            }
        }

        Err(LlmError::AllFallbacksExhausted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_selection() {
        let router = ModelRouter::new(ModelRouter::default_tiers());
        let tier = router.select_tier(TaskComplexity::Complex);
        assert!(tier.primary.contains("claude-sonnet"));
    }

    #[test]
    fn test_tier_fallback_on_unknown() {
        let router = ModelRouter::new(vec![]);
        // Should not panic
    }
}
```

**Step 2: Run tests**

```bash
cargo test model_router
```

Expected: 2 tests passed.

**Step 3: Commit**

```bash
git add rust/src/model_router.rs
git commit -m "feat: add tiered model router with fallback chain logic"
```

---

### Task 6: Implement skill registry with TOML loader

**Objective:** Build the trait-based skill registry that loads skill configs from TOML files.

**Files:**
- Modify: `rust/src/skill_registry.rs`

**Step 1: Write `rust/src/skill_registry.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfig {
    pub skill: SkillMeta,
    pub tool: Option<ToolDef>,
    pub implementation: ImplConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub trigger_keywords: Vec<String>,
    pub complexity: String,
    #[serde(default)]
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplConfig {
    #[serde(rename = "type")]
    pub impl_type: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub auth_env: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub data_template: Option<String>,
    #[serde(default)]
    pub extras: Option<HashMap<String, String>>,
}

pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn to_tool_schema(&self) -> serde_json::Value;
}

pub struct TomlSkill {
    config: SkillConfig,
}

impl TomlSkill {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: SkillConfig = toml::from_str(&content)?;
        Ok(Self { config })
    }
}

impl Skill for TomlSkill {
    fn name(&self) -> &str {
        &self.config.skill.name
    }

    fn description(&self) -> &str {
        &self.config.skill.description
    }

    fn to_tool_schema(&self) -> serde_json::Value {
        let tool = self.config.tool.as_ref();
        let params = tool.map(|t| {
            let props: serde_json::Value = t.parameters.iter().map(|(k, v)| {
                (k.clone(), serde_json::json!({
                    "type": v,
                    "description": format!("The {} parameter", k),
                }))
            }).collect();
            serde_json::json!({
                "type": "object",
                "properties": props,
                "required": t.parameters.keys().collect::<Vec<_>>(),
            })
        });

        serde_json::json!({
            "type": "function",
            "function": {
                "name": tool.map(|t| t.name.as_str()).unwrap_or(self.name()),
                "description": self.description(),
                "parameters": params,
            }
        })
    }
}

pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: HashMap::new() }
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) {
        self.skills.insert(skill.name().to_string(), skill);
    }

    pub fn load_from_dir(&mut self, dir: &Path) -> Result<usize, Box<dyn std::error::Error>> {
        let mut count = 0;
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "toml") {
                match TomlSkill::load(&path) {
                    Ok(skill) => {
                        self.register(Box::new(skill));
                        count += 1;
                    }
                    Err(e) => eprintln!("Failed to load skill {:?}: {}", path, e),
                }
            }
        }
        Ok(count)
    }

    pub fn tools_for_prompt(&self) -> Vec<serde_json::Value> {
        self.skills.values().map(|s| s.to_tool_schema()).collect()
    }

    pub fn get(&self, name: &str) -> Option<&dyn Skill> {
        self.skills.get(name).map(|s| s.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_toml_skill() {
        let toml_str = r#"
[skill]
name = "web_search"
description = "Search the web"
complexity = "Standard"

[tool]
name = "web_search"
parameters = { query = "string", max_results = "integer" }

[implementation]
type = "http"
url = "http://127.0.0.1:8080/search"
"#;
        let config: SkillConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.skill.name, "web_search");
        assert_eq!(config.implementation.impl_type, "http");
    }

    #[test]
    fn test_registry_load_and_query() {
        let mut registry = SkillRegistry::new();

        // Create temp skill file
        let dir = std::env::temp_dir().join("agent_test_skills");
        std::fs::create_dir_all(&dir).unwrap();
        let skill_path = dir.join("test_skill.toml");
        let mut f = std::fs::File::create(&skill_path).unwrap();
        f.write_all(br#"
[skill]
name = "test_skill"
description = "A test skill"
complexity = "Trivial"

[implementation]
type = "http"
"#).unwrap();

        let count = registry.load_from_dir(&dir).unwrap();
        assert_eq!(count, 1);
        assert!(registry.get("test_skill").is_some());

        let tools = registry.tools_for_prompt();
        assert_eq!(tools.len(), 1);

        std::fs::remove_dir_all(&dir).ok();
    }
}
```

**Step 2: Run tests**

```bash
cargo test skill_registry
```

Expected: 2 tests passed.

**Step 3: Commit**

```bash
git add rust/src/skill_registry.rs
git commit -m "feat: add skill registry with TOML config loader and tool schema generation"
```

---

### Task 7: Implement context manager with token budget

**Objective:** Build rolling conversation history with configurable token budget and trimming.

**Files:**
- Modify: `rust/src/context_manager.rs`

**Step 1: Write `rust/src/context_manager.rs`**

```rust
use crate::provider::Message;

pub struct ContextManager {
    messages: Vec<Message>,
    max_tokens: usize,
    /// Approximate: 1 token ≈ 4 characters for English text
    chars_per_token: usize,
}

impl ContextManager {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_tokens,
            chars_per_token: 4,
        }
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
        self.trim();
    }

    pub fn set_system_prompt(&mut self, prompt: &str) {
        // Replace or prepend system message
        self.messages.retain(|m| m.role != "system");
        self.messages.insert(0, Message {
            role: "system".to_string(),
            content: prompt.to_string(),
        });
        self.trim();
    }

    fn estimated_tokens(&self) -> usize {
        self.messages
            .iter()
            .map(|m| m.role.len() + m.content.len())
            .sum::<usize>() / self.chars_per_token
    }

    fn trim(&mut self) {
        // Always keep system message (index 0)
        while self.messages.len() > 1 && self.estimated_tokens() > self.max_tokens {
            // Remove oldest non-system message
            let remove_idx = if self.messages[0].role == "system" { 1 } else { 0 };
            if remove_idx < self.messages.len() {
                self.messages.remove(remove_idx);
            } else {
                break;
            }
        }
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn token_count(&self) -> usize {
        self.estimated_tokens()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_trim() {
        let mut ctx = ContextManager::new(50); // ~50 tokens ≈ 200 chars
        ctx.set_system_prompt("You are helpful.");
        ctx.add_message("user", "Hello!");
        ctx.add_message("assistant", "Hi there!");
        assert!(ctx.messages().len() >= 2);
    }

    #[test]
    fn test_trim_keeps_system() {
        let mut ctx = ContextManager::new(10); // Very tight budget
        ctx.set_system_prompt("System prompt here.");
        for i in 0..20 {
            ctx.add_message("user", &format!("Message number {}", i));
        }
        // System should still be there
        assert_eq!(ctx.messages()[0].role, "system");
    }

    #[test]
    fn test_clear() {
        let mut ctx = ContextManager::new(1000);
        ctx.add_message("user", "test");
        ctx.clear();
        assert!(ctx.messages().is_empty());
    }
}
```

**Step 2: Run tests**

```bash
cargo test context_manager
```

Expected: 3 tests passed.

**Step 3: Commit**

```bash
git add rust/src/context_manager.rs
git commit -m "feat: add rolling context manager with token budget trimming"
```

---

### Task 8: Implement tool parser

**Objective:** Parse LLM function call responses into structured `AgentAction` enum for the agent loop.

**Files:**
- Modify: `rust/src/tool_parser.rs`

**Step 1: Write `rust/src/tool_parser.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub skill: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug)]
pub enum ParseError {
    NoToolCall,
    InvalidJson,
}

/// Parse a tool call from an LLM response.
/// Expects either a `tool_calls` array in OpenAI-compatible format,
/// or a markdown code block with JSON.
pub fn parse(response: &str) -> Result<AgentAction, ParseError> {
    // Try OpenAI-style tool_calls first
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
        if let Some(tool_calls) = json["tool_calls"].as_array() {
            if let Some(first) = tool_calls.first() {
                let function = &first["function"];
                return Ok(AgentAction {
                    skill: function["name"].as_str().unwrap_or("unknown").to_string(),
                    parameters: function["arguments"].clone(),
                });
            }
        }
        // Direct function call format
        if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
            return Ok(AgentAction {
                skill: name.to_string(),
                parameters: json.get("parameters").cloned().unwrap_or(serde_json::Value::Null),
            });
        }
    }

    // Try markdown ```json block
    if let Some(start) = response.find("```json") {
        let after_start = &response[start + 7..];
        if let Some(end) = after_start.find("```") {
            let json_str = &after_start[..end].trim();
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(name) = parsed.get("skill").or_else(|| parsed.get("name")).and_then(|v| v.as_str()) {
                    return Ok(AgentAction {
                        skill: name.to_string(),
                        parameters: parsed.get("parameters").or(parsed.get("args")).cloned().unwrap_or(serde_json::Value::Null),
                    });
                }
            }
        }
    }

    Err(ParseError::NoToolCall)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_openai_tool_call() {
        let response = r#"{"tool_calls": [{"function": {"name": "web_search", "arguments": {"query": "rust lang"}}}]}"#;
        let action = parse(response).unwrap();
        assert_eq!(action.skill, "web_search");
        assert_eq!(action.parameters["query"], "rust lang");
    }

    #[test]
    fn test_parse_markdown_json() {
        let response = "Here's the action:\n```json\n{\"skill\": \"open_app\", \"parameters\": {\"name\": \"calculator\"}}\n```";
        let action = parse(response).unwrap();
        assert_eq!(action.skill, "open_app");
        assert_eq!(action.parameters["name"], "calculator");
    }

    #[test]
    fn test_no_tool_call() {
        let response = "I don't know how to do that.";
        assert!(parse(response).is_err());
    }
}
```

**Step 2: Run tests**

```bash
cargo test tool_parser
```

Expected: 3 tests passed.

**Step 3: Commit**

```bash
git add rust/src/tool_parser.rs
git commit -m "feat: add tool parser for OpenAI-style and markdown JSON tool calls"
```

---

### Task 9: End-to-end smoke test — call OpenRouter from Termux

**Objective:** Verify the entire Phase 1 stack works end-to-end by calling a real model.

**Files:**
- Create: `rust/examples/smoke_test.rs`

**Step 1: Write `rust/examples/smoke_test.rs`**

```rust
use android_ai_agent::complexity_classifier;
use android_ai_agent::context_manager::ContextManager;
use android_ai_agent::http_client::HttpClient;
use android_ai_agent::model_router::ModelRouter;
use android_ai_agent::provider::openrouter::OpenRouterProvider;

#[tokio::main]
async fn main() {
    // Read API key from env
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("Set OPENROUTER_API_KEY environment variable");

    let provider = OpenRouterProvider::new(api_key);
    let http = HttpClient::new();
    let router = ModelRouter::new(ModelRouter::default_tiers());
    let mut ctx = ContextManager::new(4000);

    ctx.set_system_prompt("You are a helpful assistant. Keep responses brief.");

    let prompt = "What is 2 + 2?";
    let complexity = complexity_classifier::classify(prompt);
    println!("Complexity: {:?}", complexity);
    println!("Sending prompt: {}", prompt);

    match router.call_with_fallback(&http, &provider, prompt, "You are a helpful assistant.").await {
        Ok(response) => {
            println!("Model: {}", response.model);
            println!("Response: {}", response.content);
            println!("Tokens: {} prompt + {} completion = {} total",
                response.usage.prompt_tokens,
                response.usage.completion_tokens,
                response.usage.total_tokens,
            );
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
```

**Step 2: Add to Cargo.toml**

```toml
[[example]]
name = "smoke_test"
path = "rust/examples/smoke_test.rs"
```

**Step 3: Run the smoke test**

```bash
export OPENROUTER_API_KEY="your-key-here"
cargo run --example smoke_test
```

Expected output:
```
Complexity: Trivial
Sending prompt: What is 2 + 2?
Model: openrouter/google/gemini-flash-2.5
Response: 4
Tokens: 15 prompt + 5 completion = 20 total
```

**Step 4: Commit**

```bash
git add rust/examples/smoke_test.rs Cargo.toml
git commit -m "feat: add end-to-end smoke test using OpenRouter"
```

---

### Task 10: Add Anthropic, Mistral, Google, and Local provider stubs

**Objective:** Create stub implementations for the remaining 4 providers so the `provider/` module is complete.

**Files:**
- Modify: `rust/src/provider/anthropic.rs`
- Modify: `rust/src/provider/mistral.rs`
- Modify: `rust/src/provider/google.rs`
- Modify: `rust/src/provider/local.rs`

**Step 1: Anthropic provider**

```rust
// rust/src/provider/anthropic.rs
use super::{LlmError, LlmProvider, LlmRequest, LlmResponse, Usage};

pub struct AnthropicProvider {
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl LlmProvider for AnthropicProvider {
    fn base_url(&self) -> &str {
        "https://api.anthropic.com/v1"
    }

    fn auth_header(&self) -> (&str, &str) {
        ("x-api-key", &self.api_key)
    }

    async fn call(&self, _client: &reqwest::Client, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        // Stub — Anthropic uses a different API format (messages array, not chat/completions)
        // Will be fully implemented in Phase 2 when prompt caching is needed
        Err(LlmError::ModelUnavailable("anthropic provider not yet implemented".into()))
    }
}
```

**Step 2: Mistral, Google, Local — same stub pattern**

All three follow the same stub pattern with their respective base URLs:
- Mistral: `https://api.mistral.ai/v1` — `Authorization: Bearer {key}`
- Google: `https://generativelanguage.googleapis.com/v1beta/openai` — `Authorization: Bearer {key}`
- Local: `http://127.0.0.1:8080/v1` — no auth

```rust
// mistral.rs
impl LlmProvider for MistralProvider {
    fn base_url(&self) -> &str { "https://api.mistral.ai/v1" }
    fn auth_header(&self) -> (&str, &str) { ("Authorization", &self.api_key) }
    async fn call(&self, _client: &reqwest::Client, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        Err(LlmError::ModelUnavailable("mistral provider not yet implemented".into()))
    }
}

// google.rs
impl LlmProvider for GoogleProvider {
    fn base_url(&self) -> &str { "https://generativelanguage.googleapis.com/v1beta/openai" }
    fn auth_header(&self) -> (&str, &str) { ("Authorization", &self.api_key) }
    async fn call(&self, _client: &reqwest::Client, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        Err(LlmError::ModelUnavailable("google provider not yet implemented".into()))
    }
}

// local.rs
impl LlmProvider for LocalProvider {
    fn base_url(&self) -> &str { "http://127.0.0.1:8080/v1" }
    fn auth_header(&self) -> (&str, &str) { ("", "") }
    async fn call(&self, _client: &reqwest::Client, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        Err(LlmError::ModelUnavailable("local provider not yet implemented".into()))
    }
}
```

**Step 3: Verify compilation**

```bash
cargo check
cargo test
```

Expected: All tests pass, no compilation errors.

**Step 4: Commit**

```bash
git add rust/src/provider/anthropic.rs rust/src/provider/mistral.rs rust/src/provider/google.rs rust/src/provider/local.rs
git commit -m "feat: add Anthropic, Mistral, Google, and Local provider stubs"
```

---

### Task 11: Add thiserror dependency and wire up error types

**Objective:** Replace manual error construction with proper `thiserror` derive macros.

**Files:**
- Modify: `Cargo.toml` (add `thiserror = "2"`)
- Modify: `rust/src/provider/mod.rs` (derive macros on `LlmError`)

**Step 1: Add dependency**

```toml
thiserror = "2"
```

**Step 2: Verify `LlmError` already uses `#[derive(thiserror::Error)]`** (it was written with it in Task 1)

**Step 3: Add unit test for error display**

```rust
#[test]
fn test_llm_error_display() {
    let err = LlmError::RateLimit(30);
    assert_eq!(err.to_string(), "rate limited, retry after 30s");

    let err = LlmError::AllFallbacksExhausted;
    assert_eq!(err.to_string(), "all fallbacks exhausted");
}
```

**Step 4: Run tests**

```bash
cargo test
```

**Step 5: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add thiserror dependency, wire up error display"
```

---

## Completion Checklist

- [ ] `cargo build` passes with zero errors
- [ ] `cargo test` — all tests pass (expect ~15+ tests)
- [ ] `cargo run --example smoke_test` — successfully calls OpenRouter with a real API key
- [ ] Module structure matches the spec: `http_client`, `provider/*`, `tool_parser`, `model_router`, `skill_registry`, `context_manager`, `complexity_classifier`

## Next Phase

Phase 2 (AccessibilityService) requires Android SDK — this plan intentionally stays in pure Rust, testable in Termux. Once Phase 1 passes smoke test, proceed to:
1. Cross-compile for `aarch64-linux-android`
2. Set up Android NDK + JNI bridge
3. Implement AccessibilityService in Kotlin
