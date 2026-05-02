use crate::budget_tracker;
use crate::complexity_classifier::{self, TaskComplexity};
use crate::context_manager::ContextManager;
use crate::events::agent_events::AgentEvent;
use crate::http_client::HttpClient;
use crate::model_router::ModelRouter;
use crate::provider::{LlmError, LlmProvider};
use crate::prompt_cache::CacheableProvider;
use crate::safety::policy_enforcer::{PolicyDecision, SkillPolicyEnforcer};
use crate::tool_parser;

pub struct AgentLoopConfig {
    pub max_steps: u32,
    /// Enable prompt caching for supported providers (Anthropic/OpenRouter).
    pub cache_enabled: bool,
}

impl Default for AgentLoopConfig {
    fn default() -> Self {
        Self {
            max_steps: 50,
            cache_enabled: false,
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

    /// Run the agent loop.
    /// Uses prompt caching when `config.cache_enabled` is true and the provider
    /// also implements `CacheableProvider`.
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

        // ── Budget check ──
        if budget_tracker::get_tracker().is_over_budget() {
            self.emit(AgentEvent::TaskFailed {
                error: format!(
                    "Budget exceeded: ${:.2} > ${:.2} threshold",
                    budget_tracker::get_tracker().monthly_cost(),
                    budget_tracker::get_tracker().threshold(),
                ),
            });
            return Err(LlmError::ModelUnavailable("budget exceeded".into()));
        }

        self.emit(AgentEvent::TaskStarted {
            prompt: prompt.to_string(),
            complexity,
        });

        ctx.set_system_prompt(system_prompt);

        for _step in 0..self.config.max_steps {
            // ── Budget check each step ──
            if budget_tracker::get_tracker().is_over_budget() {
                self.emit(AgentEvent::TaskFailed {
                    error: "Budget exceeded mid-task".into(),
                });
                return Err(LlmError::ModelUnavailable("budget exceeded mid-task".into()));
            }

            let response = router
                .call_with_fallback(http, provider, prompt, system_prompt)
                .await?;

            // ── Record token usage to budget tracker ──
            let tier = complexity_to_budget_tier(complexity);
            budget_tracker::get_tracker().record_usage(
                tier,
                response.usage.prompt_tokens as u64,
                response.usage.completion_tokens as u64,
            );

            self.emit(AgentEvent::ModelCalled {
                model: response.model.clone(),
                tier: complexity,
                prompt_tokens: response.usage.prompt_tokens,
            });

            match tool_parser::parse(&response.content) {
                Ok(action) => {
                    let decision = self
                        .policy_enforcer
                        .validate(&action.skill, complexity);
                    match decision {
                        PolicyDecision::Denied { reason } => {
                            self.emit(AgentEvent::TaskFailed {
                                error: reason.clone(),
                            });
                            return Err(LlmError::ModelUnavailable(reason));
                        }
                        PolicyDecision::RequiresConfirmation { skill } => {
                            self.emit(AgentEvent::ConfirmationRequired {
                                skill: skill.clone(),
                                reason: format!(
                                    "Skill '{}' requires user confirmation",
                                    skill
                                ),
                            });
                        }
                        PolicyDecision::Allowed => {}
                    }
                    self.emit(AgentEvent::SkillInvoked {
                        skill: action.skill.clone(),
                        parameters: action.parameters.clone(),
                    });
                    self.emit(AgentEvent::SkillCompleted {
                        skill: action.skill.clone(),
                        success: true,
                        summary: format!(
                            "Executed {} with {:?}",
                            action.skill, action.parameters
                        ),
                    });
                    ctx.add_message("assistant", &response.content);
                    if response.content.contains("DONE") {
                        self.emit(AgentEvent::TaskCompleted {
                            summary: response.content.clone(),
                            usage: response.usage.clone(),
                        });
                        return Ok(response.content);
                    }
                }
                Err(_) => {
                    self.emit(AgentEvent::TaskCompleted {
                        summary: response.content.clone(),
                        usage: response.usage.clone(),
                    });
                    return Ok(response.content);
                }
            }
        }
        self.emit(AgentEvent::LoopGuardTriggered {
            reason: format!("Reached max steps ({})", self.config.max_steps),
        });
        Err(LlmError::AllFallbacksExhausted)
    }

    pub fn events(&self) -> &[AgentEvent] {
        &self.events
    }
    fn emit(&mut self, event: AgentEvent) {
        self.events.push(event);
    }

    /// Run with prompt caching enabled.
    /// Requires a provider that implements both LlmProvider and CacheableProvider
    /// (OpenRouterProvider and AnthropicProvider).
    pub async fn run_cached<P: LlmProvider + CacheableProvider>(
        &mut self,
        http: &HttpClient,
        provider: &P,
        router: &ModelRouter,
        ctx: &mut ContextManager,
        prompt: &str,
        system_prompt: &str,
    ) -> Result<String, LlmError> {
        let complexity = complexity_classifier::classify(prompt);

        if budget_tracker::get_tracker().is_over_budget() {
            self.emit(AgentEvent::TaskFailed {
                error: format!(
                    "Budget exceeded: ${:.2} > ${:.2} threshold",
                    budget_tracker::get_tracker().monthly_cost(),
                    budget_tracker::get_tracker().threshold(),
                ),
            });
            return Err(LlmError::ModelUnavailable("budget exceeded".into()));
        }

        self.emit(AgentEvent::TaskStarted {
            prompt: prompt.to_string(),
            complexity,
        });

        ctx.set_system_prompt(system_prompt);

        // Cache breakpoints: cache system message + last 4 messages of conversation
        let cache_breakpoints = crate::prompt_cache::default_breakpoints(true, Some(4));

        for _step in 0..self.config.max_steps {
            if budget_tracker::get_tracker().is_over_budget() {
                self.emit(AgentEvent::TaskFailed {
                    error: "Budget exceeded mid-task".into(),
                });
                return Err(LlmError::ModelUnavailable("budget exceeded mid-task".into()));
            }

            let (response, _cached) = router
                .call_with_fallback_cached(
                    http,
                    provider,
                    prompt,
                    system_prompt,
                    &cache_breakpoints,
                )
                .await?;

            let tier = complexity_to_budget_tier(complexity);
            budget_tracker::get_tracker().record_usage(
                tier,
                response.usage.prompt_tokens as u64,
                response.usage.completion_tokens as u64,
            );

            self.emit(AgentEvent::ModelCalled {
                model: response.model.clone(),
                tier: complexity,
                prompt_tokens: response.usage.prompt_tokens,
            });

            match tool_parser::parse(&response.content) {
                Ok(action) => {
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
                        }
                        PolicyDecision::Allowed => {}
                    }
                    self.emit(AgentEvent::SkillInvoked {
                        skill: action.skill.clone(),
                        parameters: action.parameters.clone(),
                    });
                    self.emit(AgentEvent::SkillCompleted {
                        skill: action.skill.clone(),
                        success: true,
                        summary: format!("Executed {} with {:?}", action.skill, action.parameters),
                    });
                    ctx.add_message("assistant", &response.content);
                    if response.content.contains("DONE") {
                        self.emit(AgentEvent::TaskCompleted {
                            summary: response.content.clone(),
                            usage: response.usage.clone(),
                        });
                        return Ok(response.content);
                    }
                }
                Err(_) => {
                    self.emit(AgentEvent::TaskCompleted {
                        summary: response.content.clone(),
                        usage: response.usage.clone(),
                    });
                    return Ok(response.content);
                }
            }
        }
        self.emit(AgentEvent::LoopGuardTriggered {
            reason: format!("Reached max steps ({})", self.config.max_steps),
        });
        Err(LlmError::AllFallbacksExhausted)
    }
}

/// Map complexity classifier output to budget tracker tier.
fn complexity_to_budget_tier(c: TaskComplexity) -> budget_tracker::Tier {
    match c {
        TaskComplexity::Trivial => budget_tracker::Tier::Trivial,
        TaskComplexity::Standard => budget_tracker::Tier::Standard,
        TaskComplexity::Complex => budget_tracker::Tier::Complex,
        TaskComplexity::Critical => budget_tracker::Tier::Critical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_loop_creation() {
        let agent = AgentLoop::new(AgentLoopConfig::default());
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
    }

    #[test]
    fn test_cache_config_default() {
        let config = AgentLoopConfig::default();
        assert!(!config.cache_enabled);
        assert_eq!(config.max_steps, 50);
    }

    #[test]
    fn test_cache_config_enabled() {
        let config = AgentLoopConfig {
            max_steps: 25,
            cache_enabled: true,
        };
        assert!(config.cache_enabled);
        assert_eq!(config.max_steps, 25);
    }

    #[test]
    fn test_budget_blocks_task() {
        let bt = budget_tracker::get_tracker();
        bt.set_threshold(0.0001);
        bt.record_usage(budget_tracker::Tier::Standard, 1000, 500);
        assert!(bt.is_over_budget());

        // Reset for other tests
        bt.reset_month();
        bt.set_threshold(5.0);
    }

    #[test]
    fn test_complexity_to_budget_tier_mapping() {
        assert_eq!(
            complexity_to_budget_tier(TaskComplexity::Trivial),
            budget_tracker::Tier::Trivial
        );
        assert_eq!(
            complexity_to_budget_tier(TaskComplexity::Standard),
            budget_tracker::Tier::Standard
        );
        assert_eq!(
            complexity_to_budget_tier(TaskComplexity::Complex),
            budget_tracker::Tier::Complex
        );
        assert_eq!(
            complexity_to_budget_tier(TaskComplexity::Critical),
            budget_tracker::Tier::Critical
        );
    }
}
