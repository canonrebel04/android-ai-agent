use serde::{Deserialize, Serialize};

use crate::complexity_classifier::TaskComplexity;
use crate::provider::Usage;

/// Typed state-machine events for the agent loop.
///
/// Events are emitted at each stage of the orchestration cycle, allowing
/// observers (logging, UI, replay, telemetry) to hook into the agent's
/// lifecycle without coupling to the loop internals.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    /// A new task has been submitted to the agent.
    TaskStarted {
        prompt: String,
        complexity: TaskComplexity,
    },
    /// An LLM call is about to be made (or has been dispatched).
    ModelCalled {
        model: String,
        tier: TaskComplexity,
        prompt_tokens: u32,
    },
    /// A skill is being invoked with parameters.
    SkillInvoked {
        skill: String,
        parameters: serde_json::Value,
    },
    /// A skill has finished executing.
    SkillCompleted {
        skill: String,
        success: bool,
        summary: String,
    },
    /// A destructive or sensitive skill requires user confirmation.
    ConfirmationRequired {
        skill: String,
        reason: String,
    },
    /// The user has responded to a confirmation prompt.
    ConfirmationReceived {
        approved: bool,
    },
    /// The task completed successfully.
    TaskCompleted {
        summary: String,
        usage: Usage,
    },
    /// The task failed with an error.
    TaskFailed {
        error: String,
    },
    /// Consecutive identical screen captures detected (loop stall).
    StallDetected {
        identical_screens: u32,
    },
    /// A safety loop-guard triggered (max iterations, etc.).
    LoopGuardTriggered {
        reason: String,
    },
}

impl AgentEvent {
    /// Returns a `snake_case` string label for the event variant.
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
            AgentEvent::LoopGuardTriggered { .. } => "loop_guard_triggered",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_matches_variant() {
        let events: Vec<(AgentEvent, &str)> = vec![
            (
                AgentEvent::TaskStarted {
                    prompt: "hello".into(),
                    complexity: TaskComplexity::Trivial,
                },
                "task_started",
            ),
            (
                AgentEvent::ModelCalled {
                    model: "gpt-4".into(),
                    tier: TaskComplexity::Standard,
                    prompt_tokens: 42,
                },
                "model_called",
            ),
            (
                AgentEvent::SkillInvoked {
                    skill: "open_app".into(),
                    parameters: serde_json::json!({"name": "calculator"}),
                },
                "skill_invoked",
            ),
            (
                AgentEvent::SkillCompleted {
                    skill: "open_app".into(),
                    success: true,
                    summary: "opened calculator".into(),
                },
                "skill_completed",
            ),
            (
                AgentEvent::ConfirmationRequired {
                    skill: "send_money".into(),
                    reason: "destructive action".into(),
                },
                "confirmation_required",
            ),
            (
                AgentEvent::ConfirmationReceived { approved: false },
                "confirmation_received",
            ),
            (
                AgentEvent::TaskCompleted {
                    summary: "done".into(),
                    usage: Usage {
                        prompt_tokens: 100,
                        completion_tokens: 50,
                        total_tokens: 150,
                    },
                },
                "task_completed",
            ),
            (
                AgentEvent::TaskFailed {
                    error: "timeout".into(),
                },
                "task_failed",
            ),
            (
                AgentEvent::StallDetected {
                    identical_screens: 3,
                },
                "stall_detected",
            ),
            (
                AgentEvent::LoopGuardTriggered {
                    reason: "max iterations".into(),
                },
                "loop_guard_triggered",
            ),
        ];

        for (event, expected) in events {
            assert_eq!(
                event.event_type(),
                expected,
                "event_type() mismatch for {:?}",
                event
            );
        }
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let original = AgentEvent::TaskStarted {
            prompt: "write a test".into(),
            complexity: TaskComplexity::Complex,
        };

        let json = serde_json::to_string(&original).expect("serialize");
        let restored: AgentEvent = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(
            restored.event_type(),
            "task_started",
            "round-tripped event should preserve variant"
        );
        if let AgentEvent::TaskStarted { prompt, complexity } = restored {
            assert_eq!(prompt, "write a test");
            assert_eq!(complexity, TaskComplexity::Complex);
        } else {
            panic!("wrong variant after round-trip");
        }
    }
}
