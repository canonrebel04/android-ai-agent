use crate::provider::Message;

pub struct ContextManager {
    messages: Vec<Message>,
    max_tokens: usize,
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
        self.messages.retain(|m| m.role != "system");
        self.messages.insert(0, Message {
            role: "system".to_string(),
            content: prompt.to_string(),
        });
        self.trim();
    }

    fn estimated_tokens(&self) -> usize {
        self.messages.iter()
            .map(|m| m.role.len() + m.content.len())
            .sum::<usize>() / self.chars_per_token
    }

    fn trim(&mut self) {
        // Bolt ⚡ Optimization: O(N) context trimming
        // We avoid calling Vec::remove() in a loop, which was O(N^2),
        // and instead track `total_chars` in a single pass before a final Vec::drain().
        let mut total_chars: usize = self.messages.iter()
            .map(|m| m.role.len() + m.content.len())
            .sum();

        let has_system = self.messages.first().is_some_and(|m| m.role == "system");
        let start_idx = if has_system { 1 } else { 0 };

        let mut remove_end = start_idx;

        while self.messages.len() - (remove_end - start_idx) > 1
            && (total_chars / self.chars_per_token) > self.max_tokens
        {
            if remove_end < self.messages.len() {
                let m = &self.messages[remove_end];
                total_chars -= m.role.len() + m.content.len();
                remove_end += 1;
            } else {
                break;
            }
        }

        if remove_end > start_idx {
            self.messages.drain(start_idx..remove_end);
        }
    }

    /// Compact old context: summarize messages before the last N turns.
    /// Keeps system message + last `keep_last` messages intact.
    pub fn compact(&mut self, keep_last: usize) {
        if self.messages.len() <= keep_last + 2 {
            return;
        }

        // Keep: system message + last keep_last messages
        let split_idx = self.messages.len().saturating_sub(keep_last);

        // Summarize the middle section
        let old_msgs: Vec<&str> = self.messages[1..split_idx]
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| m.content.as_str())
            .collect();

        if old_msgs.is_empty() {
            return;
        }

        let summary = format!(
            "[Earlier conversation summarized ({} messages)]: {}",
            old_msgs.len(),
            old_msgs.join(" | ")
        );
        let summary = truncate(&summary, 500);

        // Replace old messages with summary
        let system = self.messages[0].clone();
        let recent: Vec<Message> = self.messages[split_idx..].to_vec();

        self.messages.clear();
        self.messages.push(system);
        self.messages.push(Message {
            role: "system".to_string(),
            content: summary,
        });
        self.messages.extend(recent);
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

    /// Check if context is approaching the token limit.
    pub fn is_near_limit(&self, ratio: f64) -> bool {
        self.estimated_tokens() as f64 > self.max_tokens as f64 * ratio
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars { s.to_string() } else { format!("{}...", &s[..max_chars]) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_trim() {
        let mut ctx = ContextManager::new(50);
        ctx.set_system_prompt("You are helpful.");
        ctx.add_message("user", "Hello!");
        ctx.add_message("assistant", "Hi there!");
        assert!(ctx.messages().len() >= 2);
    }

    #[test]
    fn test_trim_keeps_system() {
        let mut ctx = ContextManager::new(10);
        ctx.set_system_prompt("System prompt here.");
        for i in 0..20 {
            ctx.add_message("user", &format!("Message number {}", i));
        }
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
