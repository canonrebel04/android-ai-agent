//! Entity extraction and fact indexing.
//! Extracts named entities from text using regex patterns,
//! then indexes them against the fact_store for fast retrieval.

use crate::memory::fact_store::{Fact, FactStore, ProbeResult};
use regex::Regex;
use std::collections::HashSet;

/// Extract likely entity names from text.
/// Matches: capitalized words, hyphenated tech terms, repo names (owner/repo),
/// version numbers, and quoted strings.
pub fn extract_entities(text: &str) -> Vec<String> {
    let mut entities = HashSet::new();

    // Repo names: owner/repo or owner/repo-name
    let repo_re = Regex::new(r"\b([a-zA-Z0-9][-a-zA-Z0-9]*/[a-zA-Z0-9][-_.a-zA-Z0-9]*)\b").unwrap();
    for cap in repo_re.captures_iter(text) {
        entities.insert(cap[1].to_lowercase());
    }

    // Capitalized multi-word sequences (likely proper nouns)
    let proper_re = Regex::new(r"\b([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)\b").unwrap();
    for cap in proper_re.captures_iter(text) {
        entities.insert(cap[1].to_lowercase());
    }

    // Single capitalized words (not at start of sentence, min 3 chars)
    // Also matches at start of text via ^ alternative
    let cap_re = Regex::new(r"(?:^|(?:\.|\?|!|\n)\s+|[^a-zA-Z])([A-Z][a-zA-Z]{2,})\b").unwrap();
    for cap in cap_re.captures_iter(text) {
        let word = &cap[1];
        // Filter out common words that happen to be capitalized
        if !is_common_word(word) {
            entities.insert(word.to_lowercase());
        }
    }

    // Tech terms with dots or hyphens: rust-analyzer, claude-sonnet-4, com.example
    let tech_re = Regex::new(r"\b([a-zA-Z][-a-zA-Z0-9]*[-.][a-zA-Z][-a-zA-Z0-9.]*)\b").unwrap();
    for cap in tech_re.captures_iter(text) {
        let term = &cap[1];
        if term.len() > 3 && term.contains('-') {
            entities.insert(term.to_lowercase());
        }
    }

    // Version patterns: v1.0.0, 0.22, edition 2021
    let version_re = Regex::new(r"\b(v?\d+\.\d+(?:\.\d+)?)\b").unwrap();
    for cap in version_re.captures_iter(text) {
        entities.insert(cap[1].to_string());
    }

    entities.into_iter().collect()
}

fn is_common_word(word: &str) -> bool {
    let common = [
        "The", "This", "That", "These", "Those", "There", "Their", "They",
        "When", "Where", "Which", "While", "Would", "Could", "Should",
        "About", "After", "Before", "During", "Without", "Within",
        "However", "Therefore", "Because", "Although",
        "First", "Second", "Third", "Next", "Last", "Final",
        "Android", "Phase", "Task", "Step", "Goal", "Note",
    ];
    common.contains(&word)
}

/// Full probe pipeline: extract entities → probe fact_store for each → deduplicate → rank.
pub fn probe_all(store: &FactStore, text: &str) -> Vec<ProbeResult> {
    let entities = extract_entities(text);
    let mut seen: HashSet<i64> = HashSet::new();
    let mut results = Vec::new();

    for entity in &entities {
        if let Ok(facts) = store.probe(entity) {
            for fact in facts {
                if seen.insert(fact.id) {
                    // Relevance = trust * (1 if exact entity match, 0.5 if partial)
                    let relevance = fact.trust;
                    results.push(ProbeResult {
                        fact,
                        relevance,
                    });
                }
            }
        }
    }

    // Sort by relevance (trust * recency bonus)
    results.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}

/// Deep reason: for each pair of entities, find facts connecting both.
pub fn reason_all(store: &FactStore, entities: &[&str]) -> Vec<Fact> {
    let mut seen: HashSet<i64> = HashSet::new();
    let mut results = Vec::new();

    // Try all entity pairs
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            if let Ok(facts) = store.reason(&[entities[i], entities[j]]) {
                for fact in facts {
                    if seen.insert(fact.id) {
                        results.push(fact);
                    }
                }
            }
        }
    }

    // Also try single-entity probes for uncovered entities
    for entity in entities {
        if let Ok(facts) = store.probe(entity) {
            for fact in facts {
                if seen.insert(fact.id) {
                    results.push(fact);
                }
            }
        }
    }

    results
}

/// Search for a topic with entity-aware boosting.
pub fn search_with_entities(
    store: &FactStore,
    query: &str,
    limit: usize,
) -> Vec<ProbeResult> {
    let entities = extract_entities(query);

    // FTS5 search
    let mut results = store.search(query, limit * 2).unwrap_or_default();

    // Boost results that match extracted entities
    for r in &mut results {
        for entity in &entities {
            if r.fact.content.to_lowercase().contains(entity) {
                r.relevance += 0.2;
            }
        }
        for entity in &r.fact.entities {
            if entities.contains(entity) {
                r.relevance += 0.1;
            }
        }
    }

    results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
    results.truncate(limit);
    results
}

#[cfg(test)]
mod tests {
    use crate::memory::fact_store::FactCategory;
    use super::*;

    #[test]
    fn test_extract_repo_names() {
        let entities = extract_entities("I'm working on canonrebel04/android-ai-agent and openclaw/openclaw");
        assert!(entities.contains(&"canonrebel04/android-ai-agent".to_string()));
        assert!(entities.contains(&"openclaw/openclaw".to_string()));
    }

    #[test]
    fn test_extract_tech_terms() {
        let entities = extract_entities("Using claude-sonnet-4 and deepseek-v4-pro for routing");
        assert!(entities.contains(&"claude-sonnet-4".to_string()));
        assert!(entities.contains(&"deepseek-v4-pro".to_string()));
    }

    #[test]
    fn test_extract_versions() {
        let entities = extract_entities("Upgraded to tokio 1.52 and serde_json 1.0.140");
        assert!(entities.contains(&"1.0.140".to_string()));
    }

    #[test]
    fn test_extract_proper_nouns() {
        let entities = extract_entities("OpenClaw and Hermes Agent both have memory systems");
        assert!(entities.contains(&"hermes agent".to_string()));
        assert!(entities.contains(&"openclaw".to_string()));
    }

    #[test]
    fn test_probe_all_with_fact_store() {
        let store = FactStore::open_in_memory().unwrap();
        store.add("claude-sonnet-4 costs $3/$15 per 1M", FactCategory::Tool, &["pricing"], &["claude-sonnet-4", "pricing"], 0.9).unwrap();
        store.add("deepseek-v4 costs $0.14/$0.28 per 1M", FactCategory::Tool, &["pricing"], &["deepseek-v4", "pricing"], 0.85).unwrap();

        let results = probe_all(&store, "What is the pricing of claude-sonnet-4?");
        assert!(!results.is_empty());
        assert!(results[0].fact.content.contains("claude-sonnet-4"));
    }
}
