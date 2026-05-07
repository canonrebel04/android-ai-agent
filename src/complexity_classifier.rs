/// Classification of task complexity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskComplexity {
    /// Simple, straightforward tasks with minimal context
    Trivial,
    /// Standard tasks with moderate context and requirements
    Standard,
    /// Complex tasks requiring significant reasoning and context
    Complex,
    /// Critical tasks requiring maximum capability and reliability
    Critical,
}

/// Classifies a task based on its description using rule-based matching
///
/// # Arguments
/// * `task_description` - A string describing the task to be classified
///
/// # Returns
/// The classified TaskComplexity level
pub fn classify(task_description: &str) -> TaskComplexity {
    let lower_desc = task_description.to_lowercase();

    // Critical: Tasks requiring maximum capability and reliability
    if lower_desc.contains("critical")
        || lower_desc.contains("emergency")
        || lower_desc.contains("life or death")
        || lower_desc.contains("safety-critical")
        || lower_desc.contains("security audit")
        || lower_desc.contains("compliance")
        || lower_desc.contains("legal review")
        || lower_desc.contains("financial risk")
        || lower_desc.contains("production outage")
    {
        return TaskComplexity::Critical;
    }

    // Complex: Tasks requiring significant reasoning and context
    if lower_desc.contains("complex")
        || lower_desc.contains("multi-step")
        || lower_desc.contains("strategic")
        || lower_desc.contains("architecture")
        || lower_desc.contains("design system")
        || lower_desc.contains("refactor")
        || lower_desc.contains("migration")
        || lower_desc.contains("integration")
        || lower_desc.contains("debug")
        || lower_desc.contains("troubleshoot")
        || lower_desc.contains("optimize")
        || lower_desc.contains("performance")
        || lower_desc.contains("scalability")
    {
        return TaskComplexity::Complex;
    }

    // Standard: Standard tasks with moderate context and requirements
    if lower_desc.contains("standard")
        || lower_desc.contains("routine")
        || lower_desc.contains("typical")
        || lower_desc.contains("normal")
        || lower_desc.contains("usual")
        || lower_desc.contains("code review")
        || lower_desc.contains("documentation")
        || lower_desc.contains("testing")
        || lower_desc.contains("implement feature")
        || lower_desc.contains("fix bug")
        || lower_desc.contains("write test")
        || lower_desc.contains("update")
    {
        return TaskComplexity::Standard;
    }

    // Trivial: Simple, straightforward tasks with minimal context
    if lower_desc.contains("trivial")
        || lower_desc.contains("simple")
        || lower_desc.contains("easy")
        || lower_desc.contains("quick")
        || lower_desc.contains("minor")
        || lower_desc.contains("typo")
        || lower_desc.contains("formatting")
        || lower_desc.contains("rename")
        || lower_desc.contains("delete")
        || lower_desc.contains("cleanup")
        || lower_desc.contains("todo")
        || lower_desc.contains("chore")
    {
        return TaskComplexity::Trivial;
    }

    // Default classification based on length and content analysis
    if task_description.len() < 20 {
        if lower_desc.contains("fix") || lower_desc.contains("add") || lower_desc.contains("remove") {
            return TaskComplexity::Trivial;
        }
        return TaskComplexity::Standard;
    }

    if task_description.len() < 100 {
        return TaskComplexity::Standard;
    }

    TaskComplexity::Complex
}

/// Suggests an appropriate model based on task complexity
///
/// # Arguments
/// * `complexity` - The classified TaskComplexity level
///
/// # Returns
/// A string representing the suggested model identifier
pub fn suggest_model(complexity: TaskComplexity) -> String {
    match complexity {
        TaskComplexity::Trivial => "openai/gpt-3.5-turbo".to_string(),
        TaskComplexity::Standard => "openai/gpt-4o-mini".to_string(),
        TaskComplexity::Complex => "anthropic/claude-3-sonnet".to_string(),
        TaskComplexity::Critical => "anthropic/claude-3-opus".to_string(),
    }
}

/// Returns a human-readable display name for the complexity level
///
/// # Arguments
/// * `complexity` - The TaskComplexity level
///
/// # Returns
/// A string representing the display name
pub fn complexity_display_name(complexity: TaskComplexity) -> &'static str {
    match complexity {
        TaskComplexity::Trivial => "Trivial",
        TaskComplexity::Standard => "Standard",
        TaskComplexity::Complex => "Complex",
        TaskComplexity::Critical => "Critical",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_trivial() {
        let result = classify("Fix a typo in the README");
        assert_eq!(result, TaskComplexity::Trivial);
    }

    #[test]
    fn test_classify_standard() {
        let result = classify("Implement a new API endpoint for user profiles");
        assert_eq!(result, TaskComplexity::Standard);
    }

    #[test]
    fn test_classify_complex() {
        let result = classify("Design the system architecture for microservices");
        assert_eq!(result, TaskComplexity::Complex);
    }

    #[test]
    fn test_classify_critical() {
        let result = classify("EMERGENCY: Production database is down");
        assert_eq!(result, TaskComplexity::Critical);
    }

    #[test]
    fn test_suggest_model_all_levels() {
        assert_eq!(suggest_model(TaskComplexity::Trivial), "openai/gpt-3.5-turbo");
        assert_eq!(suggest_model(TaskComplexity::Standard), "openai/gpt-4o-mini");
        assert_eq!(suggest_model(TaskComplexity::Complex), "anthropic/claude-3-sonnet");
        assert_eq!(suggest_model(TaskComplexity::Critical), "anthropic/claude-3-opus");
    }

    #[test]
    fn test_complexity_display_name_all() {
        assert_eq!(complexity_display_name(TaskComplexity::Trivial), "Trivial");
        assert_eq!(complexity_display_name(TaskComplexity::Standard), "Standard");
        assert_eq!(complexity_display_name(TaskComplexity::Complex), "Complex");
        assert_eq!(complexity_display_name(TaskComplexity::Critical), "Critical");
    }

    #[test]
    fn test_classify_edge_cases() {
        let result = classify("CRITICAL: System failure");
        assert_eq!(result, TaskComplexity::Critical);
        
        let result = classify("This is a complex integration task");
        assert_eq!(result, TaskComplexity::Complex);
    }
}
