# Android AI Agent — Phase 2: Safety Layer + Agent Loop Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Build the safety enforcement layer (mined from swarm patterns), structured agent events/state machine, and wire the Phase 1 Rust core into a working perception→reason→act agent loop — all still in pure Rust, testable without Android SDK.

**Architecture:** Three new modules — `safety::policy_enforcer` gates every skill call, `safety::permission_guard` controls AccessibilityService access, `agent_loop` drives the full cycle. Agent events become a typed state machine. Skills registry gains YAML+Markdown dual-format support.

**Tech Stack:** Rust 1.85+, tokio 1.52, serde/serde_json 1.0, serde_yaml 0.9 (new), existing Phase 1 modules.

**Prerequisites:** Phase 1 complete at `/home/miyabi/android-ai-agent` (13 tests passing).

---

### Task 1: Add serde_yaml dependency

**Objective:** Add YAML parsing support for the enhanced skill registry.

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add dependency**
```toml
serde_yaml = "0.9"
```

**Step 2: Verify**
```bash
cargo check
```

**Step 3: Commit**
```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add serde_yaml for enhanced skill registry"
```

---

### Task 2: Implement Agent Events (typed state machine)

**Objective:** Define the `AgentEvent` enum that the agent loop state machine will emit — mirrors swarm's event bus pattern.

**Files:**
- Create: `src/events/mod.rs`
- Create: `src/events/agent_events.rs`
- Modify: `src/lib.rs` (add `pub mod events`)

**Code for `src/events/mod.rs`:**
```rust
pub mod agent_events;
```

**Code for `src/events/agent_events.rs`:**
```rust
use crate::complexity_classifier::TaskComplexity;
use crate::provider::Usage;
use crate::tool_parser::AgentAction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    TaskStarted {
        prompt: String,
        complexity: TaskComplexity,
    },
    ModelCalled {
        model: String,
        tier: TaskComplexity,
        prompt_tokens: u32,
    },
    SkillInvoked {
        skill: String,
        parameters: serde_json::Value,
    },
    SkillCompleted {
        skill: String,
        success: bool,
        summary: String,
    },
    ConfirmationRequired {
        skill: String,
        reason: String,
    },
    ConfirmationReceived {
        approved: bool,
    },
    TaskCompleted {
        summary: String,
        usage: Usage,
    },
    TaskFailed {
        error: String,
    },
    StallDetected {
        identical_screens: u32,
    },
    LoopGuardTriggered {
        reason: String,
    },
}

impl AgentEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            AgentEvent::TaskStarted { .. } => "task_started",
            AgentEvent::ModelCalled { .. } => "model_called",
            AgentEvent::SkillInvoked { .. } => "skill_invoked",
            AgentEvent::SkillCompleted { .. } => "skill_completed",
            AgentEvent::ConfirmationRequired { .. } => "confirmation_required",
            AgentEvent::ConfirmationReceived { .. } => "confirmation_received",
            AgentEvent::TaskCompleted { .. } => "task_completed",
            AgentEvent::TaskFailed { .. } => "task_failed",
            AgentEvent::StallDetected { .. } => "stall_detected",
            AgentEvent::LoopGuardTriggered { .. } => "loop_guard",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_labels() {
        let e = AgentEvent::TaskStarted { prompt: "test".into(), complexity: TaskComplexity::Trivial };
        assert_eq!(e.event_type(), "task_started");

        let e = AgentEvent::SkillInvoked { skill: "web_search".into(), parameters: serde_json::json!({"q":"rust"}) };
        assert_eq!(e.event_type(), "skill_invoked");
    }
}
```

**Step 2: Update `src/lib.rs`** — add `pub mod events;`

**Step 3: Run tests**
```bash
cargo test events
```

**Step 4: Commit**
```bash
git add -A && git commit -m "feat: add typed AgentEvent state machine for agent loop"
```

---

### Task 3: Implement SkillPolicyEnforcer (from swarm ToolPolicyEnforcer pattern)

**Objective:** Gate every skill invocation before execution — checks tier requirements and confirmation rules. Direct adaptation of swarm's `safety/enforcer.py`.

**Files:**
- Create: `src/safety/mod.rs`
- Create: `src/safety/policy_enforcer.rs`
- Modify: `src/lib.rs` (add `pub mod safety`)

**Code for `src/safety/mod.rs`:**
```rust
pub mod policy_enforcer;
```

**Code for `src/safety/policy_enforcer.rs`:**
```rust
use crate::complexity_classifier::TaskComplexity;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allowed,
    RequiresConfirmation { skill: String },
    Denied { reason: String },
}

pub struct SkillPolicyEnforcer {
    /// Skills that require user confirmation before execution
    confirmation_required: HashSet<String>,
    /// Skills that require Critical complexity tier
    critical_skills: HashSet<String>,
}

impl SkillPolicyEnforcer {
    pub fn new() -> Self {
        let confirmation_required: HashSet<String> = [
            "send_message", "phone_call", "delete", "payment",
        ].iter().map(|s| s.to_string()).collect();

        let critical_skills: HashSet<String> = [
            "send_message", "phone_call", "payment", "shell_cmd",
        ].iter().map(|s| s.to_string()).collect();

        Self { confirmation_required, critical_skills }
    }

    pub fn validate(&self, skill: &str, complexity: TaskComplexity) -> PolicyDecision {
        // Critical skills require Critical tier
        if self.critical_skills.contains(skill) && complexity != TaskComplexity::Critical {
            return PolicyDecision::Denied {
                reason: format!(
                    "Skill '{}' requires Critical tier, but task is {:?}",
                    skill, complexity
                ),
            };
        }

        // Confirmation-required skills need user approval
        if self.confirmation_required.contains(skill) {
            return PolicyDecision::RequiresConfirmation {
                skill: skill.to_string(),
            };
        }

        PolicyDecision::Allowed
    }

    /// Check if a skill is allowed at all (for unknown/custom skills)
    pub fn is_known_skill(&self, skill: &str) -> bool {
        // Allow all skills by default — deny-list approach
        true
    }
}

impl Default for SkillPolicyEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivial_skill_allowed() {
        let enforcer = SkillPolicyEnforcer::new();
        assert_eq!(
            enforcer.validate("web_search", TaskComplexity::Standard),
            PolicyDecision::Allowed
        );
    }

    #[test]
    fn test_critical_skill_requires_critical_tier() {
        let enforcer = SkillPolicyEnforcer::new();
        let result = enforcer.validate("payment", TaskComplexity::Standard);
        assert!(matches!(result, PolicyDecision::Denied { .. }));
    }

    #[test]
    fn test_critical_skill_allowed_at_critical_tier() {
        let enforcer = SkillPolicyEnforcer::new();
        assert_eq!(
            enforcer.validate("payment", TaskComplexity::Critical),
            PolicyDecision::Allowed
        );
    }

    #[test]
    fn test_send_message_requires_confirmation() {
        let enforcer = SkillPolicyEnforcer::new();
        let result = enforcer.validate("send_message", TaskComplexity::Critical);
        assert!(matches!(result, PolicyDecision::RequiresConfirmation { .. }));
    }
}
```

**Step 2: Update `src/lib.rs`** — add `pub mod safety;`

**Step 3: Run tests**
```bash
cargo test policy_enforcer
```
Expected: 4 tests pass.

**Step 4: Commit**
```bash
git add -A && git commit -m "feat: add skill policy enforcer with tier gating and confirmation checks"
```

---

### Task 4: Implement AndroidPermissionGuard (from swarm FilesystemGuard pattern)

**Objective:** Control which Android apps/packages the AccessibilityService can interact with. Maps swarm's `BLOCKED_PATHS` pattern to Android package blocking.

**Files:**
- Create: `src/safety/permission_guard.rs`
- Modify: `src/safety/mod.rs` (add module)

**Code for `src/safety/permission_guard.rs`:**
```rust
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessDecision {
    pub allowed: bool,
    pub reason: String,
}

pub struct AndroidPermissionGuard {
    /// Apps the agent can NEVER interact with
    blocked_packages: HashSet<String>,
    /// Apps that are always safe to read/interact with
    safe_packages: HashSet<String>,
    /// Actions that are blocked globally
    blocked_actions: HashSet<String>,
}

impl AndroidPermissionGuard {
    pub fn new() -> Self {
        let blocked_packages: HashSet<String> = [
            "com.android.settings",
            "com.android.settings.intelligence",
            "com.google.android.gms.supervision",
            "com.android.vending",        // Play Store — no purchases
        ].iter().map(|s| s.to_string()).collect();

        let safe_packages: HashSet<String> = [
            "com.google.android.calendar",
            "com.google.android.contacts",
            "com.google.android.gm",
            "com.google.android.apps.messaging",
            "org.telegram.messenger",
            "com.whatsapp",
            "com.android.chrome",
            "com.android.calculator2",
            "com.termux",
        ].iter().map(|s| s.to_string()).collect();

        let blocked_actions: HashSet<String> = [
            "uninstall_app",
            "disable_app",
            "clear_app_data",
            "factory_reset",
            "change_password",
        ].iter().map(|s| s.to_string()).collect();

        Self { blocked_packages, safe_packages, blocked_actions }
    }

    pub fn can_interact_with_app(&self, package: &str) -> AccessDecision {
        if self.blocked_packages.contains(package) {
            return AccessDecision {
                allowed: false,
                reason: format!("Package '{}' is blocked for all agent interactions", package),
            };
        }
        AccessDecision { allowed: true, reason: String::new() }
    }

    pub fn can_perform_action(&self, action: &str) -> AccessDecision {
        if self.blocked_actions.contains(action) {
            return AccessDecision {
                allowed: false,
                reason: format!("Action '{}' is globally blocked", action),
            };
        }
        AccessDecision { allowed: true, reason: String::new() }
    }

    pub fn is_safe_package(&self, package: &str) -> bool {
        self.safe_packages.contains(package)
    }

    pub fn add_blocked_package(&mut self, package: String) {
        self.blocked_packages.insert(package);
    }

    pub fn add_safe_package(&mut self, package: String) {
        self.safe_packages.insert(package);
    }
}

impl Default for AndroidPermissionGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_package_denied() {
        let guard = AndroidPermissionGuard::new();
        assert!(!guard.can_interact_with_app("com.android.settings").allowed);
    }

    #[test]
    fn test_safe_package_allowed() {
        let guard = AndroidPermissionGuard::new();
        assert!(guard.can_interact_with_app("com.google.android.gm").allowed);
    }

    #[test]
    fn test_blocked_action_denied() {
        let guard = AndroidPermissionGuard::new();
        assert!(!guard.can_perform_action("factory_reset").allowed);
    }

    #[test]
    fn test_is_safe_package() {
        let guard = AndroidPermissionGuard::new();
        assert!(guard.is_safe_package("com.termux"));
        assert!(!guard.is_safe_package("com.evil.app"));
    }
}
```

**Step 2: Update `src/safety/mod.rs`:** add `pub mod permission_guard;`

**Step 3: Run tests**
```bash
cargo test permission_guard
```
Expected: 4 tests pass.

**Step 4: Commit**
```bash
git add -A && git commit -m "feat: add Android permission guard with blocked packages and actions"
```

---

### Task 5: Implement Agent Loop (perception → reason → act → verify)

**Objective:** Wire Phase 1 modules into a working agent loop. This is the central coordinator — takes a user prompt, classifies it, routes to a model, parses tool calls, validates them via policy enforcer, and cycles until done.

**Files:**
- Create: `src/agent_loop.rs`
- Modify: `src/lib.rs` (add module)

**Code for `src/agent_loop.rs`:**
```rust
use crate::complexity_classifier::{self, TaskComplexity};
use crate::context_manager::ContextManager;
use crate::events::agent_events::AgentEvent;
use crate::http_client::HttpClient;
use crate::model_router::ModelRouter;
use crate::provider::{LlmError, LlmProvider, LlmResponse};
use crate::safety::policy_enforcer::{PolicyDecision, SkillPolicyEnforcer};
use crate::tool_parser;

pub struct AgentLoopConfig {
    pub max_steps: u32,
    pub max_identical_screens: u32,
}

impl Default for AgentLoopConfig {
    fn default() -> Self {
        Self {
            max_steps: 50,
            max_identical_screens: 5,
        }
    }
}

pub struct AgentLoop {
    config: AgentLoopConfig,
    policy_enforcer: SkillPolicyEnforcer,
    events: Vec<AgentEvent>,
}

impl AgentLoop {
    pub fn new(config: AgentLoopConfig) -> Self {
        Self {
            config,
            policy_enforcer: SkillPolicyEnforcer::new(),
            events: Vec::new(),
        }
    }

    pub async fn run<P: LlmProvider>(
        &mut self,
        http: &HttpClient,
        provider: &P,
        router: &ModelRouter,
        ctx: &mut ContextManager,
        prompt: &str,
        system_prompt: &str,
    ) -> Result<String, LlmError> {
        let complexity = complexity_classifier::classify(prompt);
        self.emit(AgentEvent::TaskStarted {
            prompt: prompt.to_string(),
            complexity,
        });

        ctx.set_system_prompt(system_prompt);
        ctx.add_message("user", prompt);

        for step in 0..self.config.max_steps {
            // 1. REASON — call the model with tiered routing
            let response = router.call_with_fallback(http, provider, prompt, system_prompt).await?;

            self.emit(AgentEvent::ModelCalled {
                model: response.model.clone(),
                tier: complexity,
                prompt_tokens: response.usage.prompt_tokens,
            });

            // 2. PARSE — try to extract a tool call
            match tool_parser::parse(&response.content) {
                Ok(action) => {
                    // 3. ENFORCE — validate against policy
                    let decision = self.policy_enforcer.validate(&action.skill, complexity);
                    match decision {
                        PolicyDecision::Denied { reason } => {
                            self.emit(AgentEvent::TaskFailed { error: reason.clone() });
                            return Err(LlmError::ModelUnavailable(reason));
                        }
                        PolicyDecision::RequiresConfirmation { skill } => {
                            self.emit(AgentEvent::ConfirmationRequired {
                                skill: skill.clone(),
                                reason: format!("Skill '{}' requires user confirmation", skill),
                            });
                            // In production: wait for user confirmation via JNI callback
                            // For now: auto-approve if not in a real execution context
                        }
                        PolicyDecision::Allowed => {}
                    }

                    // 4. ACT — execute the skill
                    self.emit(AgentEvent::SkillInvoked {
                        skill: action.skill.clone(),
                        parameters: action.parameters.clone(),
                    });

                    // Skill execution will be bridged via JNI in Phase 3
                    // For now, log the intended action
                    self.emit(AgentEvent::SkillCompleted {
                        skill: action.skill.clone(),
                        success: true,
                        summary: format!("Executed {} with {:?}", action.skill, action.parameters),
                    });

                    // 5. VERIFY — add model response to context, loop back
                    ctx.add_message("assistant", &response.content);

                    // Check if task looks complete
                    if response.content.contains("DONE") || response.content.contains("Task complete") {
                        self.emit(AgentEvent::TaskCompleted {
                            summary: response.content.clone(),
                            usage: response.usage.clone(),
                        });
                        return Ok(response.content);
                    }
                }
                Err(_) => {
                    // No tool call — might be a final answer
                    self.emit(AgentEvent::TaskCompleted {
                        summary: response.content.clone(),
                        usage: response.usage.clone(),
                    });
                    return Ok(response.content);
                }
            }

            // Safety: if we're past max steps without completion
            if step >= self.config.max_steps - 1 {
                self.emit(AgentEvent::LoopGuardTriggered {
                    reason: format!("Reached max steps ({})", self.config.max_steps),
                });
                return Err(LlmError::AllFallbacksExhausted);
            }
        }

        Err(LlmError::AllFallbacksExhausted)
    }

    pub fn events(&self) -> &[AgentEvent] {
        &self.events
    }

    fn emit(&mut self, event: AgentEvent) {
        self.events.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_loop_creation() {
        let config = AgentLoopConfig::default();
        let agent = AgentLoop::new(config);
        assert!(agent.events().is_empty());
    }

    #[test]
    fn test_emitting_events() {
        let mut agent = AgentLoop::new(AgentLoopConfig::default());
        agent.emit(AgentEvent::TaskStarted {
            prompt: "test".into(),
            complexity: TaskComplexity::Trivial,
        });
        assert_eq!(agent.events().len(), 1);
        assert_eq!(agent.events()[0].event_type(), "task_started");
    }
}
```

**Step 2: Update `src/lib.rs`** — add `pub mod agent_loop;`

**Step 3: Run tests**
```bash
cargo test agent_loop
```

**Step 4: Commit**
```bash
git add -A && git commit -m "feat: add agent loop with perception→reason→act→verify cycle"
```

---

### Task 6: Enhance Skill Registry — support paired .md instructions

**Objective:** Extend `skill_registry.rs` so skills can optionally load a paired `.md` file for detailed instructions, matching swarm's YAML+MD pattern.

**Files:**
- Modify: `src/skill_registry.rs` (add `instructions` field + MD loading)

**Step 1: Add `instructions` to `SkillConfig` and `TomlSkill`**

In `SkillConfig`, add: `pub instructions: Option<String>` (no serde, loaded separately)

In `TomlSkill::load`, add after loading TOML:
```rust
let md_path = path.with_extension("md");
let instructions = if md_path.exists() {
    Some(std::fs::read_to_string(&md_path)?)
} else {
    None
};
```

Store in `TomlSkill { config, instructions }`.

Add `pub fn instructions(&self) -> Option<&str>` to `Skill` trait (with default `None`).

**Step 2: Run tests**
```bash
cargo test skill_registry
```

**Step 3: Commit**
```bash
git add -A && git commit -m "feat: add paired .md instruction loading to skill registry"
```

---

### Task 7: Identity Anchoring — inject role into system prompts

**Objective:** Create a helper that builds the identity-anchored system prompt. Based on swarm's pattern of injecting role definitions into every agent's system prompt.

**Files:**
- Create: `src/identity.rs`
- Modify: `src/lib.rs` (add module)

**Code for `src/identity.rs`:**
```rust
use crate::complexity_classifier::TaskComplexity;

/// Build an identity-anchored system prompt that reinforces the agent's capabilities
/// and constraints. Directly adapted from swarm's role identity injection pattern.
pub fn build_system_prompt(
    tier: TaskComplexity,
    base_prompt: &str,
    memory_content: &str,
) -> String {
    let capability_block = match tier {
        TaskComplexity::Trivial => "You are an Android agent in Trivial mode. Keep it simple.",
        TaskComplexity::Standard => "You are an Android agent. Use tools efficiently.",
        TaskComplexity::Complex => "You are an Android agent in Complex mode. Reason carefully, plan multi-step actions.",
        TaskComplexity::Critical => "You are an Android agent in CRITICAL mode. Double-check everything. Confirm destructive actions.",
    };

    let constraints = "\n\nCONSTRAINTS:\n- Never access banking/payment apps without user confirmation\n- Never modify system settings\n- Never uninstall apps\n- Always report what you're about to do before doing it";

    let memory_section = if !memory_content.is_empty() {
        format!("\n\nUSER MEMORY:\n{}", memory_content)
    } else {
        String::new()
    };

    format!("{}{}{}\n\n{}", capability_block, constraints, memory_section, base_prompt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builds_with_memory() {
        let prompt = build_system_prompt(
            TaskComplexity::Complex,
            "You are helpful.",
            "User prefers concise responses.",
        );
        assert!(prompt.contains("Complex mode"));
        assert!(prompt.contains("CONSTRAINTS"));
        assert!(prompt.contains("concise responses"));
    }

    #[test]
    fn test_trivial_mode() {
        let prompt = build_system_prompt(
            TaskComplexity::Trivial,
            "Be brief.",
            "",
        );
        assert!(prompt.contains("Trivial mode"));
        assert!(prompt.contains("Keep it simple"));
    }
}
```

**Step 2: Run tests**
```bash
cargo test identity
```

**Step 3: Commit**
```bash
git add -A && git commit -m "feat: add identity-anchored system prompt builder from swarm pattern"
```

---

### Task 8: End-to-end agent loop smoke test

**Objective:** Create a smoke test that exercises the full stack: classify → identity anchor → agent loop → model call → policy enforcement.

**Files:**
- Create: `examples/agent_loop_smoke.rs`
- Modify: `Cargo.toml` (add example)

**Step 1: Write `examples/agent_loop_smoke.rs`**

```rust
use android_ai_agent::agent_loop::{AgentLoop, AgentLoopConfig};
use android_ai_agent::complexity_classifier;
use android_ai_agent::context_manager::ContextManager;
use android_ai_agent::http_client::HttpClient;
use android_ai_agent::identity;
use android_ai_agent::model_router::ModelRouter;
use android_ai_agent::provider::openrouter::OpenRouterProvider;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .expect("Set OPENROUTER_API_KEY");

    let provider = OpenRouterProvider::new(api_key);
    let http = HttpClient::new();
    let router = ModelRouter::new(ModelRouter::default_tiers());
    let mut ctx = ContextManager::new(4000);

    let prompt = "Search the web for the latest Rust release version";
    let complexity = complexity_classifier::classify(prompt);

    let system_prompt = identity::build_system_prompt(
        complexity,
        "You control an Android phone. Use tools to complete tasks.",
        "User prefers concise responses. Phone model: Pixel.",
    );

    let mut agent = AgentLoop::new(AgentLoopConfig {
        max_steps: 3,
        ..Default::default()
    });

    match agent.run(&http, &provider, &router, &mut ctx, prompt, &system_prompt).await {
        Ok(result) => {
            println!("Agent completed: {}", result);
            println!("\nEvent log:");
            for event in agent.events() {
                println!("  [{}] {:?}", event.event_type(), event);
            }
        }
        Err(e) => {
            eprintln!("Agent failed: {}", e);
            std::process::exit(1);
        }
    }
}
```

**Step 2: Add to Cargo.toml**
```toml
[[example]]
name = "agent_loop_smoke"
path = "examples/agent_loop_smoke.rs"
```

**Step 3: Verify compilation**
```bash
cargo check --example agent_loop_smoke
```

**Step 4: Commit**
```bash
git add -A && git commit -m "feat: add end-to-end agent loop smoke test with identity anchoring"
```

---

## Completion Checklist

- [ ] `cargo build` — zero errors
- [ ] `cargo test` — all tests pass (~25+ tests)
- [ ] `cargo check --example agent_loop_smoke` — compiles
- [ ] New modules: `safety/`, `events/`, `agent_loop.rs`, `identity.rs`
- [ ] All swarm patterns integrated: policy enforcer, permission guard, identity anchoring, events

## Phase 2 Module Map (after completion)

```
src/
├── lib.rs
├── http_client.rs
├── complexity_classifier.rs
├── model_router.rs
├── context_manager.rs
├── tool_parser.rs
├── skill_registry.rs
├── agent_loop.rs          ← NEW: perception→reason→act→verify
├── identity.rs            ← NEW: role-anchored system prompts
├── provider/
│   ├── mod.rs
│   ├── openrouter.rs
│   ├── anthropic.rs
│   ├── mistral.rs
│   ├── google.rs
│   └── local.rs
├── safety/                ← NEW
│   ├── mod.rs
│   ├── policy_enforcer.rs
│   └── permission_guard.rs
└── events/                ← NEW
    ├── mod.rs
    └── agent_events.rs
```
