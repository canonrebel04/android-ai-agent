//! Context injector — assembles pre-fetched facts, web results, and
//! session context into a compact XML block for injection into the LLM prompt.
//!
//! Token budget: 20% of context window reserved for injected context.
//! Facts are ranked by trust, deduplicated, and truncated to fit budget.

use crate::memory::fact_store::ProbeResult;

/// Configuration for context injection.
pub struct InjectorConfig {
    /// Maximum characters for the entire injected context block.
    pub max_chars: usize,
    /// Maximum facts to include.
    pub max_facts: usize,
    /// Minimum trust score for a fact to be included.
    pub min_trust: f64,
}

impl Default for InjectorConfig {
    fn default() -> Self {
        Self {
            max_chars: 3000, // ~750 tokens at 4 chars/token
            max_facts: 15,
            min_trust: 0.4,
        }
    }
}

/// Assembled context ready for prompt injection.
pub struct InjectedContext {
    /// Full XML block to inject into the user prompt.
    pub xml_block: String,
    /// Estimated token count of the injected block.
    pub estimated_tokens: usize,
    /// Number of facts included.
    pub fact_count: usize,
    /// Whether web results were included.
    pub has_web_results: bool,
}

/// Assemble context from facts, web results, and session summary.
pub fn assemble_context(
    facts: &[ProbeResult],
    web_results: &[WebSnippet],
    session_summary: Option<&str>,
    config: &InjectorConfig,
) -> InjectedContext {
    let mut xml = String::from("<active_context>\n");
    let mut fact_count = 0;
    let mut has_web = false;

    // ── Web results (highest priority — up-to-date info) ──
    if !web_results.is_empty() {
        xml.push_str("  <web_search>\n");
        for snippet in web_results.iter().take(3) {
            if xml.len() + snippet.snippet.len() + 40 > config.max_chars {
                break;
            }
            xml.push_str(&format!(
                "    <result query=\"{}\">{}</result>\n",
                xml_escape(&snippet.query),
                xml_escape(&snippet.snippet),
            ));
            has_web = true;
        }
        xml.push_str("  </web_search>\n");
    }

    // ── Relevant facts (ranked by trust * relevance) ──
    if !facts.is_empty() {
        xml.push_str("  <relevant_facts>\n");
        let mut sorted: Vec<_> = facts.iter().collect();
        sorted.sort_by(|a, b| {
            (b.fact.trust * b.relevance)
                .partial_cmp(&(a.fact.trust * a.relevance))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for probe in sorted.iter().take(config.max_facts) {
            if probe.fact.trust < config.min_trust {
                continue;
            }

            let line = format!(
                "    <fact trust=\"{:.2}\" id=\"{}\">{}</fact>\n",
                probe.fact.trust,
                probe.fact.id,
                xml_escape(&truncate_str(&probe.fact.content, 200)),
            );

            if xml.len() + line.len() > config.max_chars {
                break;
            }

            xml.push_str(&line);
            fact_count += 1;
        }
        xml.push_str("  </relevant_facts>\n");
    }

    // ── Session summary (cross-session context) ──
    if let Some(summary) = session_summary {
        let line = format!(
            "  <recent_work>{}</recent_work>\n",
            xml_escape(&truncate_str(summary, 300))
        );
        if xml.len() + line.len() <= config.max_chars {
            xml.push_str(&line);
        }
    }

    xml.push_str("</active_context>");

    let chars = xml.len();
    InjectedContext {
        estimated_tokens: chars / 4, // rough estimate: 4 chars per token
        fact_count,
        has_web_results: has_web,
        xml_block: xml,
    }
}

/// A web search result snippet.
#[derive(Debug, Clone)]
pub struct WebSnippet {
    pub query: String,
    pub snippet: String,
    pub url: String,
}

/// Simple web search result for pre-fetch.
#[derive(Debug, Clone)]
pub struct WebSearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
}

impl WebSearchResult {
    pub fn to_snippet(&self, query: &str) -> WebSnippet {
        WebSnippet {
            query: query.to_string(),
            snippet: truncate_str(&format!("{}: {}", self.title, self.snippet), 150),
            url: self.url.clone(),
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s.to_string()
    } else {
        let boundary = s[..max_chars].rfind(' ').unwrap_or(max_chars);
        format!("{}...", &s[..boundary])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::fact_store::{Fact, FactCategory, ProbeResult};

    fn make_fact(id: i64, content: &str, trust: f64) -> ProbeResult {
        ProbeResult {
            fact: Fact {
                id,
                content: content.to_string(),
                category: FactCategory::General,
                tags: vec![],
                entities: vec![],
                trust,
                created_at: String::new(),
                last_accessed: String::new(),
                access_count: 0,
            },
            relevance: 1.0,
        }
    }

    #[test]
    fn test_assemble_basic() {
        let facts = vec![
            make_fact(1, "android-ai-agent uses Rust edition 2021", 0.9),
            make_fact(2, "serde_json pinned to 1.0.140 for Android compat", 0.85),
        ];

        let ctx = assemble_context(&facts, &[], None, &InjectorConfig::default());
        assert!(ctx.xml_block.contains("relevant_facts"));
        assert!(ctx.xml_block.contains("edition 2021"));
        assert_eq!(ctx.fact_count, 2);
    }

    #[test]
    fn test_low_trust_filtered() {
        let facts = vec![
            make_fact(1, "high trust", 0.9),
            make_fact(2, "low trust", 0.2),
        ];

        let ctx = assemble_context(&facts, &[], None, &InjectorConfig::default());
        assert_eq!(ctx.fact_count, 1);
        assert!(ctx.xml_block.contains("high trust"));
        assert!(!ctx.xml_block.contains("low trust"));
    }

    #[test]
    fn test_web_results_included() {
        let web = vec![WebSnippet {
            query: "rust android ndk".into(),
            snippet: "cargo-ndk 4.1.2 is the latest".into(),
            url: "https://example.com".into(),
        }];

        let ctx = assemble_context(&[], &web, None, &InjectorConfig::default());
        assert!(ctx.has_web_results);
        assert!(ctx.xml_block.contains("web_search"));
        assert!(ctx.xml_block.contains("cargo-ndk"));
    }

    #[test]
    fn test_char_limit_respected() {
        let config = InjectorConfig {
            max_chars: 100,
            ..Default::default()
        };
        let facts: Vec<_> = (0..10)
            .map(|i| {
                make_fact(
                    i as i64,
                    &format!("fact number {} with lots of content", i),
                    0.9,
                )
            })
            .collect();

        let ctx = assemble_context(&facts, &[], None, &config);
        assert!(ctx.xml_block.len() <= 100 + 50); // +50 for XML wrapper overhead
    }
}
