//! Fact deduplication — prevents redundant facts from being stored.
//!
//! Three strategies:
//! 1. Hash dedup — SHA-256 of normalized content → skip if exists
//! 2. Semantic dedup — cosine similarity on word-bag → skip if >0.85
//! 3. Contains-check — new fact contained in existing → skip
//!                     existing contained in new → replace

use crate::memory::fact_store::FactStore;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

/// Result of checking whether a fact is a duplicate.
#[derive(Debug)]
pub enum DedupResult {
    /// The fact is new — store it.
    New,
    /// The fact is a duplicate of an existing fact.
    Duplicate { existing_id: i64, similarity: f64 },
    /// The new fact subsumes an existing one (contains all info + more).
    /// Replace the existing with this one.
    Subsumes { existing_id: i64 },
}

/// Deduplicate a candidate fact against the store.
pub fn check_duplicate(store: &FactStore, content: &str, entities: &[&str]) -> DedupResult {
    // 1. Hash check — fast, exact match on normalized content
    let hash = hash_content(content);
    let normalized = normalize(content);

    // 2. Probe existing facts for the same entities
    let mut candidates = Vec::new();
    for entity in entities {
        if let Ok(facts) = store.probe(entity) {
            candidates.extend(facts);
        }
    }

    // Deduplicate candidate list
    let mut seen: HashSet<i64> = HashSet::new();
    candidates.retain(|f| seen.insert(f.id));

    for fact in &candidates {
        let existing_hash = hash_content(&fact.content);
        let existing_normalized = normalize(&fact.content);

        // Exact hash match
        if hash == existing_hash {
            return DedupResult::Duplicate {
                existing_id: fact.id,
                similarity: 1.0,
            };
        }

        // Contains check: new is substring of existing
        if existing_normalized.contains(&normalized) && normalized.len() > 10 {
            return DedupResult::Duplicate {
                existing_id: fact.id,
                similarity: normalized.len() as f64 / existing_normalized.len() as f64,
            };
        }

        // Contains check: existing is substring of new → new subsumes old
        if normalized.contains(&existing_normalized) && existing_normalized.len() > 10 {
            return DedupResult::Subsumes {
                existing_id: fact.id,
            };
        }

        // Semantic dedup: word-bag overlap
        let similarity = jaccard_similarity(&normalized, &existing_normalized);
        if similarity > 0.85 {
            return DedupResult::Duplicate {
                existing_id: fact.id,
                similarity,
            };
        }
    }

    DedupResult::New
}

/// Normalize text for comparison: lowercase, strip punctuation, collapse whitespace.
fn normalize(text: &str) -> String {
    let lower = text.to_lowercase();
    let no_punct: String = lower
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    let words: Vec<&str> = no_punct.split_whitespace().collect();
    words.join(" ")
}

/// SHA-256 hash of normalized content.
fn hash_content(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(normalize(text).as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Jaccard similarity between two texts (word-level).
/// Returns 0.0 (no overlap) to 1.0 (identical word sets).
fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let words_a: HashSet<&str> = a.split_whitespace().collect();
    let words_b: HashSet<&str> = b.split_whitespace().collect();

    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    intersection as f64 / union as f64
}

/// Smart add: check for duplicates before inserting. Returns the fact ID (new or existing).
pub fn smart_add(
    store: &FactStore,
    content: &str,
    category: crate::memory::fact_store::FactCategory,
    tags: &[&str],
    entities: &[&str],
    trust: f64,
) -> i64 {
    match check_duplicate(store, content, entities) {
        DedupResult::New => store
            .add(content, category, tags, entities, trust)
            .unwrap_or(-1),
        DedupResult::Duplicate { existing_id, .. } => {
            // Boost trust on the existing fact since it's being "rediscovered"
            let _ = store.feedback(existing_id, true);
            existing_id
        }
        DedupResult::Subsumes { existing_id } => {
            // Replace the old fact with the newer, more complete version
            let _ = store.update(existing_id, Some(content), Some(0.05));
            existing_id
        }
    }
}

/// Message-level dedup for conversation turns.
/// Returns true if the message is redundant (already said).
pub fn is_redundant_message(new_msg: &str, recent_msgs: &[&str]) -> bool {
    if new_msg.len() < 10 {
        return false;
    }

    let norm_new = normalize(new_msg);

    for msg in recent_msgs {
        let norm_existing = normalize(msg);

        // Exact match after normalization
        if norm_new == norm_existing {
            return true;
        }

        // New contained in existing
        if norm_existing.contains(&norm_new) {
            return true;
        }

        // High Jaccard similarity
        if jaccard_similarity(&norm_new, &norm_existing) > 0.9 {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::fact_store::FactCategory;

    #[test]
    fn test_normalize_lowercase_strips_punctuation() {
        let n = normalize("Hello, World! This is a Test.");
        assert_eq!(n, "hello world this is a test");
    }

    #[test]
    fn test_hash_content_deterministic() {
        let h1 = hash_content("android-ai-agent uses Rust");
        let h2 = hash_content("android-ai-agent uses Rust");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_jaccard_identical() {
        let sim = jaccard_similarity("hello world", "hello world");
        assert!((sim - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_jaccard_no_overlap() {
        let sim = jaccard_similarity("hello world", "goodbye mars");
        assert!((sim - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_smart_add_dedup() {
        let store = FactStore::open_in_memory().unwrap();

        let id1 = smart_add(
            &store,
            "serde_json pinned to 1.0.140 for Android",
            FactCategory::Project,
            &["rust"],
            &["serde_json", "android"],
            0.8,
        );
        assert!(id1 > 0);

        // Same content should dedup
        let id2 = smart_add(
            &store,
            "serde_json pinned to 1.0.140 for Android",
            FactCategory::Project,
            &["rust"],
            &["serde_json", "android"],
            0.8,
        );
        assert_eq!(id1, id2);

        // Only one fact in store
        let all = store.list(10).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_is_redundant_message() {
        let recent = vec!["android-ai-agent uses Rust edition 2021"];
        assert!(is_redundant_message(
            "android-ai-agent uses Rust edition 2021",
            &recent
        ));
        assert!(!is_redundant_message(
            "it also uses Kotlin for the UI",
            &recent
        ));
    }
}
