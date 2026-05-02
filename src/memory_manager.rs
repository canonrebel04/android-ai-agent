use std::fs;
use std::io;
use std::path::PathBuf;

/// Manages persistent memory stored in a MEMORY.md file.
///
/// The file is structured as:
/// ```markdown
/// # Memory
///
/// ## Persistent Facts
/// <facts>
///
/// ## Recent Context
/// <tasks>
/// ```
pub struct MemoryManager {
    path: PathBuf,
}

impl MemoryManager {
    /// Returns the default path for the MEMORY.md file.
    ///
    /// On Android, uses `$EXTERNAL_STORAGE/.agent/MEMORY.md`.
    /// Otherwise, uses `~/.agent/MEMORY.md`.
    pub fn default_path() -> PathBuf {
        #[cfg(target_os = "android")]
        {
            if let Ok(dir) = std::env::var("EXTERNAL_STORAGE") {
                return PathBuf::from(dir).join(".agent").join("MEMORY.md");
            }
            // Fallback on Android: use /sdcard
            PathBuf::from("/sdcard/.agent/MEMORY.md")
        }
        #[cfg(not(target_os = "android"))]
        {
            let home = dirs_fallback();
            home.join(".agent").join("MEMORY.md")
        }
    }

    /// Creates a new `MemoryManager` with the default path.
    pub fn new() -> Self {
        Self {
            path: Self::default_path(),
        }
    }

    /// Creates a new `MemoryManager` with a custom path.
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Reads the entire MEMORY.md file as a string.
    ///
    /// If the file does not exist, a default template is returned
    /// (but not written to disk unless something is persisted).
    pub fn read(&self) -> String {
        match fs::read_to_string(&self.path) {
            Ok(content) => content,
            Err(_) => Self::default_template(),
        }
    }

    /// Overwrites the MEMORY.md file with the given content.
    pub fn write(&self, content: &str) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, content)
    }

    /// Appends a fact line under the `## Persistent Facts` section.
    ///
    /// If the section does not exist it is created.  If the file
    /// does not exist yet it is created from the default template
    /// before the fact is appended.
    pub fn add_fact(&self, fact: &str) -> io::Result<()> {
        let mut content = self.read();
        let header = "## Persistent Facts";
        let entry = format!("- {}\n", fact.trim());

        if let Some(pos) = content.find(header) {
            // Insert the fact on the line immediately after the header.
            let insert_at = content[pos..]
                .find('\n')
                .map(|n| pos + n + 1)
                .unwrap_or(content.len());
            content.insert_str(insert_at, &entry);
        } else {
            // Section missing — append it before "## Recent Context" if it
            // exists, otherwise at the end of the file.
            if let Some(rc_pos) = content.find("## Recent Context") {
                let section = format!("## Persistent Facts\n{}\n", entry);
                content.insert_str(rc_pos, &section);
            } else {
                content.push_str(&format!("\n## Persistent Facts\n{}\n", entry));
            }
        }

        self.write(&content)
    }

    /// Updates (replaces) the content under the `## Recent Context` section.
    ///
    /// Any existing content between the header and the next `##` heading
    /// (or end-of-file) is replaced with the provided task description.
    pub fn update_last_task(&self, task: &str) -> io::Result<()> {
        let mut content = self.read();
        let header = "## Recent Context";

        if let Some(pos) = content.find(header) {
            // Find the end of the header line.
            let after_header = content[pos..]
                .find('\n')
                .map(|n| pos + n + 1)
                .unwrap_or(content.len());

            // Find the start of the next section (## heading) or EOF.
            let end_of_section = content[after_header..]
                .find("\n## ")
                .map(|n| after_header + n)
                .unwrap_or(content.len());

            // Replace everything between after_header and end_of_section.
            let new_section = format!("- {}\n", task.trim());
            content.replace_range(after_header..end_of_section, &new_section);
        } else {
            // Section missing — append at the end.
            content.push_str(&format!(
                "\n## Recent Context\n- {}\n",
                task.trim()
            ));
        }

        self.write(&content)
    }

    /// Returns the default template for a new MEMORY.md file.
    fn default_template() -> String {
        "# Memory\n\n## Persistent Facts\n\n## Recent Context\n".to_string()
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolves `~` to the home directory.  Falls back to the current
/// directory if the home directory cannot be determined.
fn dirs_fallback() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Helper: create a temporary directory that is cleaned up on drop.
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("memory_mgr_test_{}", uuid()));
            fs::create_dir_all(&dir).unwrap();
            Self { path: dir }
        }

        fn join(&self, name: &str) -> PathBuf {
            self.path.join(name)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn uuid() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as u64
    }

    // ------------------------------------------------------------------
    // Test 1 – read/write round-trip
    // ------------------------------------------------------------------
    #[test]
    fn test_read_write() {
        let tmp = TempDir::new();
        let path = tmp.join("test_read_write.md");
        let mgr = MemoryManager::with_path(&path);

        // Initial read should return default template.
        let initial = mgr.read();
        assert!(
            initial.contains("## Persistent Facts"),
            "default template missing facts header"
        );

        // Write something and read it back.
        mgr.write("hello world").unwrap();
        assert_eq!(mgr.read(), "hello world");
    }

    // ------------------------------------------------------------------
    // Test 2 – add_fact
    // ------------------------------------------------------------------
    #[test]
    fn test_add_fact() {
        let tmp = TempDir::new();
        let path = tmp.join("test_add_fact.md");
        let mgr = MemoryManager::with_path(&path);

        mgr.add_fact("Rust is awesome").unwrap();
        let content = mgr.read();
        assert!(
            content.contains("- Rust is awesome"),
            "fact not found in {content}"
        );

        // Add a second fact.
        mgr.add_fact("Hermes is fast").unwrap();
        let content = mgr.read();
        assert!(content.contains("- Rust is awesome"));
        assert!(content.contains("- Hermes is fast"));
    }

    // ------------------------------------------------------------------
    // Test 3 – empty / non-existent memory
    // ------------------------------------------------------------------
    #[test]
    fn test_empty_memory() {
        let tmp = TempDir::new();
        let path = tmp.join("nonexistent.md");
        let mgr = MemoryManager::with_path(&path);

        // Reading a non-existent file returns the template without creating it.
        let content = mgr.read();
        assert!(!path.exists(), "file should not be created by read() alone");
        assert!(content.contains("## Persistent Facts"));
        assert!(content.contains("## Recent Context"));

        // Calling write creates the file.
        mgr.write("# custom").unwrap();
        assert!(path.exists());
        assert_eq!(mgr.read(), "# custom");
    }
}
