//! SOUL.md bootstrap system — persona injection with cache boundary splitting.
//!
//! Bootstrap files are injected into the system prompt at session start.
//! After the first turn, subsequent turns skip bootstrap injection (token saving).
//! The cache boundary marker (<!-- CACHE_BOUNDARY -->) separates stable prefix
//! from dynamic suffix for Anthropic/DeepSeek prompt caching.
//!
//! Inspired by OpenClaw's bootstrap system with:
//! - Multiple bootstrap files (SOUL.md, IDENTITY.md, MEMORY.md)
//! - Cache boundary splitting
//! - Skip-after-first optimization
//! - File change detection (inode/mtime-based invalidation)

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// A bootstrap file loaded at session start.
#[derive(Debug, Clone)]
pub struct BootstrapFile {
    pub name: String, // e.g., "SOUL.md"
    pub path: PathBuf,
    pub content: String,
    pub max_chars: usize, // Truncate if exceeds
}

/// Status of a bootstrap injection.
#[derive(Debug)]
pub struct BootstrapContext {
    /// Whether this is the first injection (full bootstrap) or a skip.
    pub is_first_turn: bool,
    /// The full system prompt to inject.
    pub system_prompt: String,
    /// The stable prefix (safe to cache).
    pub stable_prefix: String,
    /// The dynamic suffix (changes per turn).
    pub dynamic_suffix: String,
    /// Whether the response will benefit from prompt caching.
    pub can_cache: bool,
}

/// The SOUL system manages bootstrap file loading, caching, and injection.
pub struct SoulSystem {
    /// Paths to bootstrap files to load.
    bootstrap_paths: Vec<PathBuf>,
    /// Maximum total characters across all bootstrap files.
    max_total_chars: usize,
    /// Whether the first turn has completed.
    first_turn_done: AtomicBool,
    /// Cached bootstrap file contents (path → (mtime, content)).
    cache: Mutex<HashMap<PathBuf, (u64, String)>>,
}

impl SoulSystem {
    /// Create a new SOUL system with default bootstrap paths.
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/laptop".into());

        Self {
            bootstrap_paths: vec![
                PathBuf::from(format!("{}/.hermes/persona.md", home)),
                PathBuf::from(format!("{}/.agent/MEMORY.md", home)),
            ],
            max_total_chars: 8000, // ~2K tokens
            first_turn_done: AtomicBool::new(false),
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Set custom bootstrap paths.
    pub fn with_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.bootstrap_paths = paths;
        self
    }

    /// Set max total characters for bootstrap.
    pub fn with_max_chars(mut self, max_chars: usize) -> Self {
        self.max_total_chars = max_chars;
        self
    }

    /// Load all bootstrap files, respecting file change detection.
    pub fn load_bootstrap(&self) -> Vec<BootstrapFile> {
        let mut files = Vec::new();
        let mut cache = self.cache.lock().unwrap();
        let mut total_chars = 0;

        for path in &self.bootstrap_paths {
            if !path.exists() {
                continue;
            }

            // Check file modification time for cache invalidation
            let mtime = fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                })
                .unwrap_or(0);

            let content = if let Some((cached_mtime, cached_content)) = cache.get(path) {
                if *cached_mtime == mtime {
                    cached_content.clone()
                } else {
                    let fresh = Self::read_file(path, self.max_total_chars - total_chars);
                    cache.insert(path.clone(), (mtime, fresh.clone()));
                    fresh
                }
            } else {
                let fresh = Self::read_file(path, self.max_total_chars - total_chars);
                cache.insert(path.clone(), (mtime, fresh.clone()));
                fresh
            };

            if content.is_empty() {
                continue;
            }

            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown.md".into());

            total_chars += content.len();
            if total_chars > self.max_total_chars {
                break; // Stop loading more files
            }

            files.push(BootstrapFile {
                name,
                path: path.clone(),
                content,
                max_chars: self.max_total_chars,
            });
        }

        files
    }

    fn read_file(path: &Path, max_chars: usize) -> String {
        let result = fs::read_to_string(path).unwrap_or_default();
        if result.len() > max_chars {
            let boundary = result[..max_chars].rfind('\n').unwrap_or(max_chars);
            result[..boundary].to_string()
        } else {
            result
        }
    }

    /// Assemble the system prompt with cache boundary.
    /// First turn: include full bootstrap. Subsequent turns: skip bootstrap.
    pub fn assemble_prompt(
        &self,
        dynamic_instructions: &str,
        active_context: Option<&str>,
    ) -> BootstrapContext {
        let is_first = !self.first_turn_done.load(Ordering::SeqCst);

        let mut prompt = String::new();

        if is_first {
            let files = self.load_bootstrap();
            for file in &files {
                prompt.push_str(&format!(
                    "<!-- BOOTSTRAP: {} -->\n{}\n",
                    file.name, file.content
                ));
            }
            prompt.push_str("<!-- CACHE_BOUNDARY -->\n");
        }

        // Dynamic suffix: instructions + active context
        prompt.push_str(dynamic_instructions);
        prompt.push_str("\n");

        if let Some(ctx) = active_context {
            prompt.push_str(ctx);
        }

        // Split at cache boundary
        let (stable, dynamic) = if let Some(pos) = prompt.find("<!-- CACHE_BOUNDARY -->") {
            let s = prompt[..pos].trim().to_string();
            let d = prompt[pos + "<!-- CACHE_BOUNDARY -->".len()..]
                .trim()
                .to_string();
            (s, d)
        } else {
            (String::new(), prompt.clone())
        };

        let can_cache = !stable.is_empty() && is_first;

        BootstrapContext {
            is_first_turn: is_first,
            system_prompt: prompt,
            stable_prefix: stable,
            dynamic_suffix: dynamic,
            can_cache,
        }
    }

    /// Mark the first turn as complete. Subsequent calls to assemble_prompt
    /// will skip bootstrap injection.
    pub fn mark_first_turn_done(&self) {
        self.first_turn_done.store(true, Ordering::SeqCst);
    }

    /// Check if this is still the first turn.
    pub fn is_first_turn(&self) -> bool {
        !self.first_turn_done.load(Ordering::SeqCst)
    }

    /// Reset for a new session.
    pub fn reset(&self) {
        self.first_turn_done.store(false, Ordering::SeqCst);
    }
}

impl Default for SoulSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_file(content: &str) -> PathBuf {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "test_soul_{}_{}.md",
            std::process::id(),
            rand_suffix()
        ));
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    fn rand_suffix() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    #[test]
    fn test_bootstrap_first_turn() {
        let path = temp_file("You are a helpful assistant.\nBe concise.");
        let soul = SoulSystem::new().with_paths(vec![path.clone()]);

        let ctx = soul.assemble_prompt("Respond to the user.", None);
        assert!(ctx.is_first_turn);
        assert!(ctx.system_prompt.contains("helpful assistant"));
        assert!(ctx.stable_prefix.contains("helpful assistant"));
        assert!(ctx.can_cache);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_bootstrap_skip_after_first() {
        let path = temp_file("Test persona");
        let soul = SoulSystem::new().with_paths(vec![path.clone()]);

        // First turn
        let ctx1 = soul.assemble_prompt("dynamic", None);
        assert!(ctx1.is_first_turn);
        assert!(ctx1.system_prompt.contains("Test persona"));

        soul.mark_first_turn_done();

        // Second turn — bootstrap skipped
        let ctx2 = soul.assemble_prompt("dynamic", None);
        assert!(!ctx2.is_first_turn);
        assert!(!ctx2.system_prompt.contains("Test persona"));

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_reset() {
        let path = temp_file("reset test");
        let soul = SoulSystem::new().with_paths(vec![path.clone()]);

        soul.mark_first_turn_done();
        assert!(!soul.is_first_turn());

        soul.reset();
        assert!(soul.is_first_turn());

        fs::remove_file(&path).ok();
    }
}
