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
        while self.messages.len() > 1 && self.estimated_tokens() > self.max_tokens {
            let remove_idx = if self.messages[0].role == "system" { 1 } else { 0 };
            if remove_idx < self.messages.len() {
                self.messages.remove(remove_idx);
            } else {
                break;
            }
        }
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
