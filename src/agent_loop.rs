use crate::complexity_classifier::{self, TaskComplexity};
use crate::context_manager::ContextManager;
use crate::events::agent_events::AgentEvent;
use crate::http_client::HttpClient;
use crate::model_router::ModelRouter;
use crate::provider::{LlmError, LlmProvider};
use crate::safety::policy_enforcer::{PolicyDecision, SkillPolicyEnforcer};
use crate::tool_parser;

pub struct AgentLoopConfig {
    pub max_steps: u32,
}

impl Default for AgentLoopConfig {
    fn default() -> Self {
        Self { max_steps: 50 }
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
            prompt: prompt.to_string(), complexity,
        });

        ctx.set_system_prompt(system_prompt);

        for _step in 0..self.config.max_steps {
            let response = router.call_with_fallback(http, provider, prompt, system_prompt).await?;
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

    pub fn events(&self) -> &[AgentEvent] { &self.events }
    fn emit(&mut self, event: AgentEvent) { self.events.push(event); }
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
        agent.emit(AgentEvent::TaskStarted { prompt: "test".into(), complexity: TaskComplexity::Trivial });
        assert_eq!(agent.events().len(), 1);
    }
}
