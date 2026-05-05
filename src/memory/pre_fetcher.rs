//! Pre-fetch pipeline — runs before each LLM call.
//! Combines web search + fact_store probe into a unified context block.
//!
//! Flow:
//! 1. Extract entities + topics from user prompt
//! 2. Web search each entity (concurrent, cached)
//! 3. fact_store.probe() for each entity  
//! 4. fact_store.reason() for entity pairs
//! 5. Assemble into InjectedContext via injector
//! 6. Return context ready for prompt injection

use crate::memory::fact_index;
use crate::memory::fact_store::FactStore;
use crate::memory::injector::{self, InjectedContext, InjectorConfig, WebSearchResult, WebSnippet};
use crate::web_prefetch::{SearchCache, SearchProvider, WebPrefetchConfig};
use reqwest::Client;
use tokio::task::JoinSet;

/// Configuration for the full pre-fetch pipeline.
pub struct PreFetchConfig {
    pub web_search_enabled: bool,
    pub web_config: WebPrefetchConfig,
    pub injector_config: InjectorConfig,
}

impl Default for PreFetchConfig {
    fn default() -> Self {
        Self {
            web_search_enabled: false,
            web_config: WebPrefetchConfig::default(),
            injector_config: InjectorConfig::default(),
        }
    }
}

/// The pre-fetch engine.
pub struct PreFetcher {
    config: PreFetchConfig,
    client: Client,
    search_cache: SearchCache,
}

impl PreFetcher {
    pub fn new(config: PreFetchConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("failed to build pre-fetch HTTP client"),
            search_cache: SearchCache::new(config.web_config.cache_ttl_secs),
            config,
        }
    }

    /// Run the full pre-fetch pipeline for a user prompt.
    pub async fn fetch(
        &self,
        store: &FactStore,
        prompt: &str,
        session_summary: Option<&str>,
    ) -> InjectedContext {
        let entities = fact_index::extract_entities(prompt);

        // Detect URLs in prompt
        let urls = extract_urls(prompt);

        // Fetch URL content (lightweight page previews)
        let url_snippets = if !urls.is_empty() {
            fetch_url_previews(&self.client, &urls).await
        } else {
            Vec::new()
        };

        // Web search (concurrent, cached)
        let web_results = if self.config.web_search_enabled {
            self.web_search_batch(&entities).await
        } else {
            Vec::new()
        };

        // Probe fact_store
        let probe_results = if !entities.is_empty() {
            fact_index::probe_all(store, prompt)
        } else {
            Vec::new()
        };

        // Convert web results to snippets
        let mut snippets: Vec<WebSnippet> = web_results
            .iter()
            .map(|r| {
                let query = entities
                    .iter()
                    .find(|e| {
                        r.title.to_lowercase().contains(&e.to_lowercase())
                            || r.snippet.to_lowercase().contains(&e.to_lowercase())
                    })
                    .map(|e| e.as_str())
                    .unwrap_or("general");
                WebSnippet {
                    query: query.to_string(),
                    snippet: format!("{}: {}", r.title, r.snippet),
                    url: r.url.clone(),
                }
            })
            .collect();

        // Prepend URL fetches (highest priority — exactly what user sent)
        snippets.splice(0..0, url_snippets);

        // Assemble context
        injector::assemble_context(
            &probe_results,
            &snippets,
            session_summary,
            &self.config.injector_config,
        )
    }

    /// Run concurrent web searches for extracted entities.
    async fn web_search_batch(&self, entities: &[String]) -> Vec<WebSearchResult> {
        if entities.is_empty() || matches!(self.config.web_config.provider, SearchProvider::None) {
            return Vec::new();
        }

        let max_queries = self.config.web_config.max_queries.min(entities.len());
        let mut set = JoinSet::new();
        let mut all_results = Vec::new();

        for entity in entities.iter().take(max_queries) {
            let query = entity.clone();
            let client = self.client.clone();
            let config = self.config.web_config.clone();

            set.spawn(async move { search_direct(&client, &query, &config).await });
        }

        while let Some(result) = set.join_next().await {
            if let Ok(results) = result {
                all_results.extend(results);
            }
        }

        // Deduplicate by URL
        let mut seen = std::collections::HashSet::new();
        all_results.retain(|r| seen.insert(r.url.clone()));
        all_results
    }

    pub fn purge_cache(&self) {
        self.search_cache.purge_expired();
    }
}

/// Direct search for a single query (no caching — used in batch mode).
async fn search_direct(
    client: &Client,
    query: &str,
    config: &WebPrefetchConfig,
) -> Vec<WebSearchResult> {
    match &config.provider {
        SearchProvider::None => Vec::new(),
        SearchProvider::Brave { api_key } => {
            search_brave_direct(client, query, api_key, config).await
        }
        SearchProvider::SearXNG { base_url } => {
            search_searxng_direct(client, query, base_url, config).await
        }
        SearchProvider::Websurfx { base_url } => {
            search_websurfx_direct(client, query, base_url, config).await
        }
    }
}

// Direct search implementations for batch mode (no caching).

/// Extract URLs from text using regex.
fn extract_urls(text: &str) -> Vec<String> {
    // Match http:// or https:// followed by non-whitespace chars
    let re = regex::Regex::new(r"https?://\S+").unwrap();
    re.find_iter(text)
        .map(|m| {
            m.as_str()
                .trim_end_matches(&['.', ',', ')', ']', '}', '"', '\''][..])
                .to_string()
        })
        .collect()
}

/// Fetch lightweight page previews for URLs.
async fn fetch_url_previews(client: &Client, urls: &[String]) -> Vec<WebSnippet> {
    let mut snippets = Vec::new();

    for url in urls.iter().take(3) {
        // Fetch first 4KB of page
        let resp = match client
            .get(url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Read up to 4KB
        let body = match resp.text().await {
            Ok(b) => b,
            Err(_) => continue,
        };

        // Extract title + first meaningful text
        let title = extract_html_title(&body).unwrap_or_else(|| url.to_string());
        let text = strip_html_tags(&body);
        let preview = truncate_url_preview(&text, 200);

        snippets.push(WebSnippet {
            query: "url_fetch".to_string(),
            snippet: format!("[URL] {} - {}", title, preview),
            url: url.clone(),
        });
    }

    snippets
}

fn extract_html_title(html: &str) -> Option<String> {
    let re = regex::Regex::new(r"<title[^>]*>(.*?)</title>").unwrap();
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
}

fn strip_html_tags(html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    let stripped = re.replace_all(html, " ");
    let re_ws = regex::Regex::new(r"\s+").unwrap();
    re_ws.replace_all(&stripped, " ").trim().to_string()
}

fn truncate_url_preview(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        text.to_string()
    } else {
        let boundary = text[..max_chars].rfind(' ').unwrap_or(max_chars);
        format!("{}...", &text[..boundary])
    }
}

async fn search_brave_direct(
    client: &Client,
    query: &str,
    api_key: &str,
    config: &WebPrefetchConfig,
) -> Vec<WebSearchResult> {
    let url = format!(
        "https://api.search.brave.com/res/v1/web/search?q={}&count={}",
        crate::web_prefetch::urlencoding(query),
        config.max_results
    );
    let resp = match client
        .get(&url)
        .header("Accept", "application/json")
        .header("X-Subscription-Token", api_key)
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
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
    json["web"]["results"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|r| WebSearchResult {
            title: crate::web_prefetch::truncate(
                r["title"].as_str().unwrap_or(""),
                config.max_chars_per_result,
            ),
            snippet: crate::web_prefetch::truncate(
                r["description"].as_str().unwrap_or(""),
                config.max_chars_per_result,
            ),
            url: r["url"].as_str().unwrap_or("").to_string(),
        })
        .collect()
}

async fn search_searxng_direct(
    client: &Client,
    query: &str,
    base_url: &str,
    config: &WebPrefetchConfig,
) -> Vec<WebSearchResult> {
    let url = format!(
        "{}/search?q={}&format=json",
        base_url.trim_end_matches('/'),
        crate::web_prefetch::urlencoding(query)
    );
    let resp = match client
        .get(&url)
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
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
    json["results"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .take(config.max_results)
        .map(|r| WebSearchResult {
            title: crate::web_prefetch::truncate(
                r["title"].as_str().unwrap_or(""),
                config.max_chars_per_result,
            ),
            snippet: crate::web_prefetch::truncate(
                r["content"].as_str().unwrap_or(""),
                config.max_chars_per_result,
            ),
            url: r["url"].as_str().unwrap_or("").to_string(),
        })
        .collect()
}

async fn search_websurfx_direct(
    client: &Client,
    query: &str,
    base_url: &str,
    config: &WebPrefetchConfig,
) -> Vec<WebSearchResult> {
    let url = format!(
        "{}/search?q={}&format=json",
        base_url.trim_end_matches('/'),
        crate::web_prefetch::urlencoding(query)
    );
    let resp = match client
        .get(&url)
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
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
    json["results"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .take(config.max_results)
        .map(|r| WebSearchResult {
            title: crate::web_prefetch::truncate(
                r["title"].as_str().unwrap_or(""),
                config.max_chars_per_result,
            ),
            snippet: crate::web_prefetch::truncate(
                r["description"].as_str().unwrap_or(""),
                config.max_chars_per_result,
            ),
            url: r["url"].as_str().unwrap_or("").to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::fact_store::FactCategory;

    #[test]
    fn test_prefetch_no_web_search_returns_facts() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let store = FactStore::open_in_memory().unwrap();
        store
            .add(
                "Tokio 1.52 is the async runtime for Rust",
                FactCategory::Project,
                &[],
                &["Tokio", "rust"],
                0.8,
            )
            .unwrap();

        let fetcher = PreFetcher::new(PreFetchConfig::default());
        let ctx = rt.block_on(fetcher.fetch(&store, "tell me about Tokio", None));

        assert!(ctx.xml_block.contains("Tokio"));
        assert!(!ctx.has_web_results);
    }
}
