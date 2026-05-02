# Swarm Repo — Reusable Patterns for Android AI Agent

> Source: `canonrebel04/swarm` — Multi-agent orchestration TUI (Python, 522 files)

## What Swarm Is

A provider-agnostic multi-agent orchestration system with TUI. Overseer + Coordinator manage a fleet of specialized coding agents (Claude Code, Codex, Gemini, OpenCode, Hermes) bound by role contracts. Agents run in isolated git worktrees with safety enforcement at every layer.

## Patterns Directly Applicable to android-ai-agent

### 1. Role-Based Tool Policy Enforcement (safety/enforcer.py)

Swarm's `ToolPolicyEnforcer` validates every tool call against a role's allowed/blocked lists BEFORE execution. This maps directly to the Android agent's skill confirmation system.

**Swarm pattern:**
```python
allowed, reason = enforcer.validate_action("scout", "Edit")
# -> (False, "Scout cannot use Edit")
```

**Android adaptation:** Wrap every skill execution in a policy check:
```rust
pub struct SkillPolicyEnforcer {
    confirmation_required: HashSet<String>,  // send, delete, pay, call
    critical_skills: HashSet<String>,         // require Critical tier
}

impl SkillPolicyEnforcer {
    pub fn validate(&self, skill: &str, complexity: TaskComplexity) -> PolicyDecision {
        if self.critical_skills.contains(skill) && complexity != Critical {
            return PolicyDecision::denied("Critical skills require Critical tier");
        }
        if self.confirmation_required.contains(skill) {
            return PolicyDecision::requires_confirmation(skill);
        }
        PolicyDecision::allowed()
    }
}
```

### 2. Filesystem Guard with Allow/Block Lists (safety/fs_guard.py)

Swarm maintains `BLOCKED_PATHS` and `SAFE_READ_DIRS` sets. Every file access goes through `check_read`/`check_write`. Different roles have different write permissions.

**Swarm pattern:**
```python
BLOCKED_PATHS = {".git/config", ".env", "/etc", "/proc", "/sys"}
SAFE_READ_DIRS = {"src/", "tests/", "docs/"}
WRITE_ALLOWED_ROLES = {"developer", "builder", "lead", "merger", "tester"}
```

**Android adaptation** — permission gating for AccessibilityService actions:
```rust
pub struct AndroidPermissionGuard {
    blocked_packages: HashSet<String>,  // banking, password managers, settings
    read_only_contexts: HashSet<String>, // notification reading vs screen control
}

impl AndroidPermissionGuard {
    pub fn can_control_app(&self, package: &str) -> bool {
        !self.blocked_packages.contains(package)
    }
    pub fn can_read_screen(&self, context: &str) -> bool { ... }
    pub fn can_perform_gesture(&self, gesture: &str) -> bool { ... }
}
```

### 3. YAML Role Contracts → Skill TOML Definitions

Swarm defines agent roles in YAML with `may`/`may_not` lists and `handoff_to` chains:
```yaml
role: developer
may: [read_repo, edit_code, run_local_tests, analyze_architecture]
may_not: [merge_branches, redefine_requirements, spawn_agents]
handoff_to: [developer, tester, reviewer]
```

**Android adaptation:** The existing skill TOML format already captures this. Enhance with:
```toml
[skill]
name = "send_message"
may = ["read_contacts", "compose_sms", "send_whatsapp"]
may_not = ["read_banking_apps", "access_settings"]
requires_confirmation = true
handoff_restriction = ["cannot_delegate_to_other_skills"]
```

### 4. Identity Anchoring in System Prompts

Swarm injects role identity into every agent's system prompt to prevent drift:
```
You are Developer. Your role: complex implementation and reasoning.
You may: read code, edit files, run tests, analyze architecture.
You may NOT: merge branches, redefine requirements, review your own work.
```

**Android adaptation for the agent loop:**
```
You are an Android agent with [Critical] tier access.
You may: open apps, read screens, tap buttons, type text.
You may NOT: access banking apps, modify system settings, send money without confirmation.
Current MEMORY.md loaded. Stick to your task.
```

### 5. Generic OpenAI-Compatible Provider (utils/provider.py)

Clean abstraction supporting any `/v1/models` + `/v1/chat/completions` endpoint via a single config:
```python
@dataclass
class ProviderConfig:
    base_url: str
    api_key: str | None
    model: str
```
Features: `fetch_models()`, `chat_completion()`, `check_health()`, env-file API key loading.

**Android adaptation:** Our `LlmProvider` trait already does this. Add `fetch_models()` and `check_health()` methods to the trait.

### 6. Event-Driven Orchestration

Swarm's event bus fires: AgentSpawned → TaskAssigned → OutputReceived → ToolCalled → TaskCompleted → HandoffTriggered.

**Android adaptation for the agent loop state machine:**
```rust
pub enum AgentEvent {
    TaskStarted { prompt: String, complexity: TaskComplexity },
    ModelCalled { model: String, tier: TaskComplexity },
    SkillInvoked { skill: String, params: Value },
    SkillCompleted { skill: String, result: SkillResult },
    ConfirmationRequired { action: AgentAction },
    ConfirmationReceived { approved: bool },
    TaskCompleted { summary: String, tokens: Usage },
    TaskFailed { error: String },
    StallDetected { identical_screens: u32 },
    LoopGuardTriggered { reason: String },
}
```

### 7. Anti-Drift Detection

Swarm's `Monitor` agent periodically inspects worker outputs for signs of role violation. Has `anti_drift.py` module.

**Android adaptation:** The existing `loop_guard` (max identical screens before abort) and `stall_detection` can be extended with:
```rust
pub struct DriftDetector {
    task_original_prompt: String,
    allowed_domains: Vec<String>,  // from skill configs
    forbidden_patterns: Vec<String>,
}

impl DriftDetector {
    pub fn check_drift(&self, current_action: &AgentAction) -> Option<DriftWarning> {
        // Check if agent wandered into banking/settings/payment apps not in allowed_domains
    }
}
```

### 8. Skill Registry: YAML Metadata + Markdown Instructions

Swarm loads skills from paired `.yaml` (metadata, tool allow/block lists) + `.md` (detailed instructions):
```
src/skills/definitions/
├── web_research.yaml   ← allowed tools, version, category
├── web_research.md     ← detailed markdown instructions
├── sql_expert.yaml
└── sql_expert.md
```

**Android adaptation:** Extend our TOML-only skill format to optionally load `.md` instructions:
```rust
pub struct TomlSkill {
    config: SkillConfig,     // from .toml
    instructions: String,    // from .md (optional)
}
```

## Swarm Architecture → Android Agent Mapping

| Swarm Concept | Android Agent Equivalent |
|---|---|
| Overseer (LLM decomposer) | Agent Loop (perception→reason→act) |
| Coordinator (lifecycle mgmt) | State Machine (task lifecycle) |
| Role Contracts (YAML may/may_not) | Skill TOML (complexity, requires_confirmation) |
| ToolPolicyEnforcer | SkillPolicyEnforcer (pre-execution validation) |
| FilesystemGuard | AndroidPermissionGuard (app-level access control) |
| Anti-Drift Monitor | DriftDetector + LoopGuard |
| Event Bus (async events) | AgentEvent enum (state machine transitions) |
| Skill Registry (YAML+MD) | Skill Registry (TOML, optionally +MD) |
| Identity Anchoring | System prompt role injection |
| Runtime Adapters | LlmProvider trait (multi-provider) |
| Cost Caps | Tiered model routing (Complexity→cost control) |

## New Rust Module Ideas from Swarm

Two modules worth adding to the Android agent's Rust core:

```
rust/src/
├── safety/
│   ├── policy_enforcer.rs    ← validates skill invocations against tier/confirmation rules
│   └── permission_guard.rs   ← app-level access control for AccessibilityService
└── events/
    └── agent_events.rs       ← structured event enum for agent loop state machine
```

These add safety layers Swarm proved necessary — policy enforcement BEFORE execution, app-level permission gating, and structured events for debuggability.
