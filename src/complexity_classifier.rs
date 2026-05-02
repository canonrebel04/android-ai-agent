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
    if word_count > 12 {
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
        assert_eq!(
            classify("write a function to sort this array"),
            TaskComplexity::Complex
        );
    }

    #[test]
    fn test_complex_multistep() {
        assert_eq!(
            classify("open gmail then forward the email to mike"),
            TaskComplexity::Complex
        );
    }

    #[test]
    fn test_critical() {
        assert_eq!(classify("send $100 to Alice"), TaskComplexity::Critical);
        assert_eq!(classify("delete all files"), TaskComplexity::Critical);
        assert_eq!(
            classify("buy tickets for the concert"),
            TaskComplexity::Critical
        );
    }
}
