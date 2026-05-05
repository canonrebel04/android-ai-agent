use crate::complexity_classifier::TaskComplexity;

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

    format!(
        "{}{}{}\n\n{}",
        capability_block, constraints, memory_section, base_prompt
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_builds_with_memory() {
        let prompt = build_system_prompt(
            TaskComplexity::Complex,
            "You are helpful.",
            "User prefers concise.",
        );
        assert!(prompt.contains("Complex mode"));
        assert!(prompt.contains("CONSTRAINTS"));
        assert!(prompt.contains("concise"));
    }
    #[test]
    fn test_trivial_mode() {
        let prompt = build_system_prompt(TaskComplexity::Trivial, "Be brief.", "");
        assert!(prompt.contains("Trivial mode"));
        assert!(prompt.contains("Keep it simple"));
    }

    #[test]
    fn test_standard_mode() {
        let prompt = build_system_prompt(TaskComplexity::Standard, "Do the task.", "");
        assert!(prompt.contains("You are an Android agent. Use tools efficiently."));
        assert!(!prompt.contains("USER MEMORY:"));
        assert!(prompt.contains("Do the task."));
    }

    #[test]
    fn test_critical_mode() {
        let prompt = build_system_prompt(
            TaskComplexity::Critical,
            "Execute carefully.",
            "User prefers safety.",
        );
        assert!(prompt.contains("CRITICAL mode"));
        assert!(prompt.contains("Confirm destructive actions"));
        assert!(prompt.contains(
            "USER MEMORY:
User prefers safety."
        ));
        assert!(prompt.contains("Execute carefully."));
    }
}
