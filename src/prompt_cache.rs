/// Marker for cacheable content boundaries in LLM requests.
/// Anthropic uses these breakpoints; OpenRouter passes them through.
#[derive(Debug, Clone)]
pub struct CacheControl {
    /// Where to set the cache breakpoint within the message sequence.
    pub breakpoint: CacheBreakpoint,
}

#[derive(Debug, Clone)]
pub enum CacheBreakpoint {
    /// Cache the system message(s) — reused across all turns.
    SystemMessages,
    /// Cache the last N messages in the conversation.
    LastMessages(usize),
    /// Custom: cache up to and including this message index.
    AtMessage(usize),
}

/// Result of applying cache markers to a request.
#[derive(Debug, Default)]
pub struct CachedRequest {
    /// Whether any cache markers were applied.
    pub cache_enabled: bool,
    /// The modified request body (if provider modified it).
    pub modified_body: Option<serde_json::Value>,
    /// Extra HTTP headers to add.
    pub extra_headers: Vec<(String, String)>,
}

/// Trait for providers that support prompt caching.
pub trait CacheableProvider {
    /// Apply cache control breakpoints to the request JSON body.
    /// Returns the modified body + any extra headers needed.
    fn apply_cache(
        &self,
        body: &serde_json::Value,
        breakpoints: &[CacheBreakpoint],
    ) -> CachedRequest;
}

/// Decide which cache breakpoints to use for a given tier.
pub fn default_breakpoints(
    cache_system: bool,
    cache_recent: Option<usize>,
) -> Vec<CacheBreakpoint> {
    let mut bp = Vec::new();
    if cache_system {
        bp.push(CacheBreakpoint::SystemMessages);
    }
    if let Some(n) = cache_recent {
        bp.push(CacheBreakpoint::LastMessages(n));
    }
    bp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_breakpoints_both() {
        let bp = default_breakpoints(true, Some(4));
        assert_eq!(bp.len(), 2);
    }

    #[test]
    fn test_default_breakpoints_none() {
        let bp = default_breakpoints(false, None);
        assert!(bp.is_empty());
    }

    #[test]
    fn test_cached_request_default() {
        let cr = CachedRequest::default();
        assert!(!cr.cache_enabled);
        assert!(cr.modified_body.is_none());
        assert!(cr.extra_headers.is_empty());
    }
}
