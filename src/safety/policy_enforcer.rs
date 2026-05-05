use crate::complexity_classifier::TaskComplexity;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allowed,
    RequiresConfirmation { skill: String },
    Denied { reason: String },
}

pub struct SkillPolicyEnforcer {
    confirmation_required: HashSet<String>,
    critical_skills: HashSet<String>,
}

impl SkillPolicyEnforcer {
    pub fn new() -> Self {
        let confirmation_required: HashSet<String> =
            ["send_message", "phone_call", "delete", "payment"]
                .iter()
                .map(|s| s.to_string())
                .collect();

        let critical_skills: HashSet<String> =
            ["send_message", "phone_call", "payment", "shell_cmd"]
                .iter()
                .map(|s| s.to_string())
                .collect();

        Self {
            confirmation_required,
            critical_skills,
        }
    }

    pub fn validate(&self, skill: &str, complexity: TaskComplexity) -> PolicyDecision {
        if self.critical_skills.contains(skill) && complexity != TaskComplexity::Critical {
            return PolicyDecision::Denied {
                reason: format!(
                    "Skill '{}' requires Critical tier, but task is {:?}",
                    skill, complexity
                ),
            };
        }
        if self.confirmation_required.contains(skill) {
            return PolicyDecision::RequiresConfirmation {
                skill: skill.to_string(),
            };
        }
        PolicyDecision::Allowed
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
        let result = enforcer.validate("set_timer", TaskComplexity::Trivial);
        assert_eq!(result, PolicyDecision::Allowed);
    }

    #[test]
    fn test_critical_skill_denied_at_standard() {
        let enforcer = SkillPolicyEnforcer::new();
        let result = enforcer.validate("phone_call", TaskComplexity::Standard);
        assert_eq!(
            result,
            PolicyDecision::Denied {
                reason: "Skill 'phone_call' requires Critical tier, but task is Standard"
                    .to_string(),
            }
        );
    }

    #[test]
    fn test_critical_skill_allowed_at_critical() {
        let enforcer = SkillPolicyEnforcer::new();
        let result = enforcer.validate("shell_cmd", TaskComplexity::Critical);
        assert_eq!(result, PolicyDecision::Allowed);
    }

    #[test]
    fn test_send_message_requires_confirmation() {
        let enforcer = SkillPolicyEnforcer::new();
        // send_message is in both critical_skills and confirmation_required,
        // so at Critical tier it passes the deny check but hits confirmation
        let result = enforcer.validate("send_message", TaskComplexity::Critical);
        assert_eq!(
            result,
            PolicyDecision::RequiresConfirmation {
                skill: "send_message".to_string(),
            }
        );
    }
}
