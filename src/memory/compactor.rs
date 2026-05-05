//! Context compactor — summarizes old conversation turns and prunes
//! stale context to stay within token budget without losing critical info.
//!
//! Two strategies, borrowed from OpenClaw:
//! 1. Soft trim  — keep head + tail of large tool results, add truncation notice
//! 2. Hard prune — replace entire tool result with placeholder
//!
//! Guards: never prune first user message (bootstrap protection),
//! never prune last N assistant turns.

/// Configuration for context compaction.
#[derive(Debug, Clone)]
pub struct CompactorConfig {
    /// Max total context characters before compaction triggers.
    pub max_chars: usize,
    /// Ratio of content to keep from tool results (head + tail).
    /// E.g., 0.3 means keep 15% head + 15% tail.
    pub soft_trim_ratio: f64,
    /// Ratio at which hard clear is used instead of soft trim.
    /// E.g., 0.5 means hard-clear when context exceeds 50% of budget.
    pub hard_clear_ratio: f64,
    /// Never prune the last N assistant turns.
    pub keep_last_assistants: usize,
    /// Minimum character count before a tool result is prunable.
    pub min_prunable_chars: usize,
}

impl Default for CompactorConfig {
    fn default() -> Self {
        Self {
            max_chars: 100_000, // ~25K tokens
            soft_trim_ratio: 0.3,
            hard_clear_ratio: 0.5,
            keep_last_assistants: 3,
            min_prunable_chars: 5000,
        }
    }
}

/// A conversation turn.
#[derive(Debug, Clone)]
pub struct Turn {
    pub role: String, // "user", "assistant", "tool"
    pub content: String,
    pub index: usize, // Position in conversation (0 = first)
}

/// Result of compacting conversation turns.
pub struct CompactedContext {
    pub turns: Vec<Turn>,
    pub total_chars: usize,
    pub turns_pruned: usize,
    pub chars_saved: usize,
}

/// Compact a conversation to fit within the character budget.
pub fn compact(turns: &[Turn], config: &CompactorConfig) -> CompactedContext {
    let current_chars: usize = turns.iter().map(|t| t.content.len()).sum();

    if current_chars <= config.max_chars {
        return CompactedContext {
            turns: turns.to_vec(),
            total_chars: current_chars,
            turns_pruned: 0,
            chars_saved: 0,
        };
    }

    let mut result: Vec<Turn> = turns.to_vec();
    let mut pruned = 0;
    let original = current_chars;

    // Identify which turns can be pruned
    let total = result.len();
    let last_assistant_indices: Vec<usize> = result
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, t)| t.role == "assistant")
        .take(config.keep_last_assistants)
        .map(|(i, _)| i)
        .collect();

    // Find trimmable tool results (not first user, not last N assistants)
    for i in (0..total).rev() {
        let current_len: usize = result.iter().map(|t| t.content.len()).sum();
        if current_len <= config.max_chars {
            break;
        }

        // Never prune first user message (bootstrap)
        if i == 0 && result[i].role == "user" {
            continue;
        }

        // Never prune last N assistant turns
        if last_assistant_indices.contains(&i) {
            continue;
        }

        let turn = &mut result[i];

        // Only prune tool results or long messages
        if turn.content.len() < config.min_prunable_chars {
            continue;
        }

        let budget_remaining = config.max_chars as f64;
        let current_usage = current_len as f64;

        if current_usage > budget_remaining * config.hard_clear_ratio {
            // Hard clear: replace with placeholder
            let placeholder = format!("[Old {} result cleared to save context space]", turn.role);
            turn.content = placeholder;
            pruned += 1;
        } else {
            // Soft trim: keep head + tail
            let keep_chars = (turn.content.len() as f64 * config.soft_trim_ratio) as usize;
            let head = keep_chars / 2;

            if head > 20 && turn.content.len() > head * 2 {
                let tail_start = turn.content.len().saturating_sub(head);
                turn.content = format!(
                    "{}...\n[{} chars trimmed]\n...{}",
                    &turn.content[..head],
                    turn.content.len() - head * 2,
                    &turn.content[tail_start..]
                );
                pruned += 1;
            }
        }
    }

    let final_chars: usize = result.iter().map(|t| t.content.len()).sum();

    CompactedContext {
        turns: result,
        total_chars: final_chars,
        turns_pruned: pruned,
        chars_saved: original.saturating_sub(final_chars),
    }
}

/// Generate a system prompt cache boundary comment.
/// Everything before this is stable (safe to cache).
/// Everything after is dynamic (changes per turn).
pub const CACHE_BOUNDARY: &str = "<!-- CACHE_BOUNDARY -->";

/// Split a system prompt at the cache boundary.
pub fn split_at_cache_boundary(prompt: &str) -> (String, String) {
    if let Some(pos) = prompt.find(CACHE_BOUNDARY) {
        let stable = prompt[..pos].trim().to_string();
        let dynamic = prompt[pos + CACHE_BOUNDARY.len()..].trim().to_string();
        (stable, dynamic)
    } else {
        (prompt.to_string(), String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_turn(role: &str, content: &str, index: usize) -> Turn {
        Turn {
            role: role.to_string(),
            content: content.to_string(),
            index,
        }
    }

    #[test]
    fn test_no_compaction_needed() {
        let turns = vec![
            make_turn("user", "hello", 0),
            make_turn("assistant", "hi there", 1),
        ];
        let config = CompactorConfig {
            max_chars: 1000,
            ..Default::default()
        };
        let result = compact(&turns, &config);
        assert_eq!(result.turns_pruned, 0);
        assert_eq!(result.turns.len(), 2);
    }

    #[test]
    fn test_hard_clear_large_tool_result() {
        let big_content = "x".repeat(10_000);
        let turns = vec![
            make_turn("user", "run this command", 0),
            make_turn("tool", &big_content, 1),
            make_turn("assistant", "done", 2),
        ];
        let config = CompactorConfig {
            max_chars: 5000,
            min_prunable_chars: 100,
            ..Default::default()
        };
        let result = compact(&turns, &config);
        assert!(result.turns_pruned > 0);
        assert!(result.total_chars <= 5000 + 200); // allow some overhead
    }

    #[test]
    fn test_preserves_first_user_message() {
        let msg = "This is the first user message — must be preserved";
        let big = "x".repeat(10_000);
        let turns = vec![
            make_turn("user", msg, 0),
            make_turn("tool", &big, 1),
            make_turn("assistant", "ok", 2),
        ];
        let config = CompactorConfig {
            max_chars: 1000,
            min_prunable_chars: 100,
            ..Default::default()
        };
        let result = compact(&turns, &config);
        assert_eq!(result.turns[0].content, msg);
    }

    #[test]
    fn test_split_cache_boundary() {
        let prompt = "stable content\n<!-- CACHE_BOUNDARY -->\ndynamic content";
        let (stable, dynamic) = split_at_cache_boundary(prompt);
        assert!(stable.contains("stable content"));
        assert!(dynamic.contains("dynamic content"));
        assert!(!stable.contains("CACHE_BOUNDARY"));
    }

    #[test]
    fn test_no_boundary_returns_full() {
        let prompt = "no boundary here";
        let (stable, dynamic) = split_at_cache_boundary(prompt);
        assert_eq!(stable, prompt);
        assert!(dynamic.is_empty());
    }
}
