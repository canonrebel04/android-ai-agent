//! Web search pre-fetch — searches the web before each LLM call for up-to-date info.
//!
//! Supported providers:
//! - Websurfx (Rust, on-device default — http://localhost:8084)
//! - SearXNG (self-hosted — http://localhost:8080)
//! - Brave Search API (cloud — https://api.search.brave.com)
//!
//! Features:
//! - Concurrent search queries (max 3)
//! - Result caching with TTL (default 5 minutes)
//! - Query deduplication within cache window
//! - Snippet truncation (150 chars per result)
//!
//! Config via env vars: BRAVE_API_KEY, SEARXNG_URL

use crate::memory::injector::WebSearchResult;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

// ── Config ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WebPrefetchConfig {
    /// Which search provider to use.
    pub provider: SearchProvider,
    /// Maximum concurrent search queries.
    pub max_queries: usize,
    /// Maximum results per query.
    pub max_results: usize,
    /// Maximum characters per result snippet.
    pub max_chars_per_result: usize,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Cache TTL in seconds.
    pub cache_ttl_secs: u64,
    /// Whether pre-fetch is enabled.
    pub enabled: bool,
}

impl Default for WebPrefetchConfig {
    fn default() -> Self {
        Self {
            provider: SearchProvider::None,
            max_queries: 2,
            max_results: 3,
            max_chars_per_result: 150,
            timeout_secs: 5,
            cache_ttl_secs: 300, // 5 min
            enabled: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SearchProvider {
    /// No provider configured — pre-fetch disabled.
    None,
    /// Websurfx — Rust meta-search engine. Default for on-device.
    /// Runs locally: cargo build --release && ./websurfx
    /// Endpoint: http://localhost:8084/search?q=
    Websurfx { base_url: String },
    /// SearXNG — self-hosted meta-search.
    SearXNG { base_url: String },
    /// Brave Search API — cloud-based.
    Brave { api_key: String },
}

impl SearchProvider {
    /// Auto-detect from environment variables.
    /// Priority: WEBSURFX_URL > SEARXNG_URL > BRAVE_API_KEY
    pub fn from_env() -> Self {
        if let Ok(url) = std::env::var("WEBSURFX_URL") {
            if !url.is_empty() {
                return SearchProvider::Websurfx { base_url: url };
            }
        }
        if let Ok(url) = std::env::var("SEARXNG_URL") {
            if !url.is_empty() {
                return SearchProvider::SearXNG { base_url: url };
            }
        }
        if let Ok(key) = std::env::var("BRAVE_API_KEY") {
            if !key.is_empty() {
                return SearchProvider::Brave { api_key: key };
            }
        }
        SearchProvider::None
    }

    /// Create a Websurfx provider (on-device default).
    pub fn websurfx_local(port: u16) -> Self {
        SearchProvider::Websurfx {
            base_url: format!("http://localhost:{}", port),
        }
    }
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct BraveResponse {
    web: Option<BraveWeb>,
}

#[derive(Debug, Deserialize)]
struct BraveWeb {
    results: Vec<BraveResult>,
}

#[derive(Debug, Deserialize)]
struct BraveResult {
    title: String,
    description: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct SearXNGResponse {
    results: Vec<SearXNGResult>,
}

#[derive(Debug, Deserialize)]
struct SearXNGResult {
    title: String,
    content: String,
    url: String,
}

// ── Cache ─────────────────────────────────────────────────────────────────────

struct CacheEntry {
    results: Vec<WebSearchResult>,
    timestamp: Instant,
}

/// Thread-safe query result cache.
pub struct SearchCache {
    entries: Mutex<HashMap<String, CacheEntry>>,
    ttl: Duration,
}

impl SearchCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    fn get(&self, query: &str) -> Option<Vec<WebSearchResult>> {
        let entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.get(query) {
            if entry.timestamp.elapsed() < self.ttl {
                return Some(entry.results.clone());
            }
        }
        None
    }

    fn set(&self, query: &str, results: Vec<WebSearchResult>) {
        let mut entries = self.entries.lock().unwrap();
        entries.insert(
            query.to_string(),
            CacheEntry {
                results,
                timestamp: Instant::now(),
            },
        );
    }

    /// Purge expired entries.
    pub fn purge_expired(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.retain(|_, e| e.timestamp.elapsed() < self.ttl);
    }
}

// ── Search engine ──────────────────────────────────────────────────────────────

/// Execute a single web search query. Returns cached results if available.
pub async fn search(
    client: &Client,
    query: &str,
    config: &WebPrefetchConfig,
    cache: &SearchCache,
) -> Vec<WebSearchResult> {
    // Check cache first
    if let Some(cached) = cache.get(query) {
        return cached;
    }

    let results = match &config.provider {
        SearchProvider::None => Vec::new(),
        SearchProvider::Brave { api_key } => search_brave(client, query, api_key, config).await,
        SearchProvider::SearXNG { base_url } => {
            search_searxng(client, query, base_url, config).await
        }
        SearchProvider::Websurfx { base_url } => {
            search_websurfx(client, query, base_url, config).await
        }
    };

    // Cache results
    cache.set(query, results.clone());
    results
}

/// Search Brave Search API.
async fn search_brave(
    client: &Client,
    query: &str,
    api_key: &str,
    config: &WebPrefetchConfig,
) -> Vec<WebSearchResult> {
    let url = format!(
        "https://api.search.brave.com/res/v1/web/search?q={}&count={}",
        urlencoding(query),
        config.max_results
    );

    let resp = match client
        .get(&url)
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip")
        .header("X-Subscription-Token", api_key)
        .timeout(Duration::from_secs(config.timeout_secs))
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let json: BraveResponse = match resp.json().await {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let results = json.web.map(|w| w.results).unwrap_or_default();

    results
        .into_iter()
        .map(|r| WebSearchResult {
            title: truncate(&r.title, config.max_chars_per_result),
            snippet: truncate(&r.description, config.max_chars_per_result),
            url: r.url,
        })
        .collect()
}

/// Search SearXNG (self-hosted).
async fn search_searxng(
    client: &Client,
    query: &str,
    base_url: &str,
    config: &WebPrefetchConfig,
) -> Vec<WebSearchResult> {
    let url = format!(
        "{}/search?q={}&format=json&categories=general",
        base_url.trim_end_matches('/'),
        urlencoding(query)
    );

    let resp = match client
        .get(&url)
        .timeout(Duration::from_secs(config.timeout_secs))
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let json: SearXNGResponse = match resp.json().await {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    json.results
        .into_iter()
        .take(config.max_results)
        .map(|r| WebSearchResult {
            title: truncate(&r.title, config.max_chars_per_result),
            snippet: truncate(&r.content, config.max_chars_per_result),
            url: r.url,
        })
        .collect()
}

/// Search Websurfx (Rust, on-device meta-search engine).
async fn search_websurfx(
    client: &Client,
    query: &str,
    base_url: &str,
    config: &WebPrefetchConfig,
) -> Vec<WebSearchResult> {
    let url = format!(
        "{}/search?q={}&format=json",
        base_url.trim_end_matches('/'),
        urlencoding(query)
    );

    let resp = match client
        .get(&url)
        .timeout(Duration::from_secs(config.timeout_secs))
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    // Websurfx returns { "results": [...] } similar to SearXNG
    json["results"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .take(config.max_results)
        .map(|r| WebSearchResult {
            title: truncate(
                r["title"].as_str().unwrap_or(""),
                config.max_chars_per_result,
            ),
            snippet: truncate(
                r["description"].as_str().unwrap_or(""),
                config.max_chars_per_result,
            ),
            url: r["url"].as_str().unwrap_or("").to_string(),
        })
        .collect()
}

// ── Helpers ────────────────────────────────────────────────────────────────────

pub fn urlencoding(s: &str) -> String {
    s.replace(' ', "+")
        .replace('"', "%22")
        .replace('#', "%23")
        .replace('&', "%26")
}

pub fn truncate(s: &str, max_chars: usize) -> String {
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

    #[test]
    fn test_search_cache_hit() {
        let cache = SearchCache::new(60);
        let result = WebSearchResult {
            title: "Test".into(),
            snippet: "test snippet".into(),
            url: "https://example.com".into(),
        };
        cache.set("rust", vec![result]);

        let cached = cache.get("rust").unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].title, "Test");
    }

    #[test]
    fn test_search_cache_miss() {
        let cache = SearchCache::new(60);
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long() {
        let result = truncate("this is a very long string that needs truncation", 20);
        assert!(result.ends_with("..."));
        assert!(result.len() <= 23); // 20 + "..."
    }

    #[test]
    fn test_urlencoding_spaces() {
        assert_eq!(urlencoding("hello world"), "hello+world");
    }
}
