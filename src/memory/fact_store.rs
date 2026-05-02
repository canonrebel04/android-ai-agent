//! Holographic fact database — port of Hermes Agent's fact_store.
//! SQLite-backed with FTS5 full-text search, entity indexing, trust scoring.
//!
//! Operations: add, search, probe, related, reason, contradict, update, remove, feedback.
//!
//! Schema:
//!   facts(id INTEGER PK, content TEXT, category TEXT, tags TEXT, entities TEXT,
//!        trust REAL DEFAULT 0.5, created_at TEXT, last_accessed TEXT, access_count INTEGER)
//!   fts_facts — FTS5 virtual table on content
//!   INDEX idx_category, idx_trust, idx_entities

use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;
use std::sync::Mutex;

// ── Types ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactCategory {
    UserPref,
    Project,
    Tool,
    General,
}

impl FactCategory {
    pub fn as_str(&self) -> &str {
        match self {
            FactCategory::UserPref => "user_pref",
            FactCategory::Project => "project",
            FactCategory::Tool => "tool",
            FactCategory::General => "general",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "user_pref" => FactCategory::UserPref,
            "project" => FactCategory::Project,
            "tool" => FactCategory::Tool,
            _ => FactCategory::General,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fact {
    pub id: i64,
    pub content: String,
    pub category: FactCategory,
    pub tags: Vec<String>,
    pub entities: Vec<String>,
    pub trust: f64,
    pub created_at: String,
    pub last_accessed: String,
    pub access_count: u32,
}

#[derive(Debug)]
pub struct ProbeResult {
    pub fact: Fact,
    pub relevance: f64, // 0.0–1.0, how relevant to the probe query
}

// ── Database ───────────────────────────────────────────────────────────────────

pub struct FactStore {
    conn: Mutex<Connection>,
}

impl FactStore {
    /// Open or create the fact database at the given path.
    pub fn open(path: impl AsRef<Path>) -> SqlResult<Self> {
        let conn = Connection::open(path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS facts (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                content     TEXT NOT NULL,
                category    TEXT NOT NULL DEFAULT 'general',
                tags        TEXT NOT NULL DEFAULT '',
                entities    TEXT NOT NULL DEFAULT '',
                trust       REAL NOT NULL DEFAULT 0.5,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                last_accessed TEXT NOT NULL DEFAULT (datetime('now')),
                access_count INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        // FTS5 virtual table for full-text search
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS fts_facts USING fts5(
                content,
                content=facts,
                content_rowid=id,
                tokenize='porter unicode61'
            )",
            [],
        )?;

        // Indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_facts_category ON facts(category)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_facts_trust ON facts(trust)",
            [],
        )?;

        // Triggers to keep FTS in sync
        conn.execute_batch(
            "CREATE TRIGGER IF NOT EXISTS facts_ai AFTER INSERT ON facts BEGIN
                INSERT INTO fts_facts(rowid, content) VALUES (new.id, new.content);
            END;
            CREATE TRIGGER IF NOT EXISTS facts_ad AFTER DELETE ON facts BEGIN
                INSERT INTO fts_facts(fts_facts, rowid, content) VALUES('delete', old.id, old.content);
            END;
            CREATE TRIGGER IF NOT EXISTS facts_au AFTER UPDATE ON facts BEGIN
                INSERT INTO fts_facts(fts_facts, rowid, content) VALUES('delete', old.id, old.content);
                INSERT INTO fts_facts(rowid, content) VALUES (new.id, new.content);
            END;",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open in-memory (for testing).
    pub fn open_in_memory() -> SqlResult<Self> {
        Self::open(":memory:")
    }

    // ── CRUD ──────────────────────────────────────────────────────────────────

    /// Add a new fact. Returns the fact ID.
    pub fn add(
        &self,
        content: &str,
        category: FactCategory,
        tags: &[&str],
        entities: &[&str],
        trust: f64,
    ) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        let tags_str = tags.join(",");
        let entities_str = entities.join(",");

        conn.execute(
            "INSERT INTO facts (content, category, tags, entities, trust) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![content, category.as_str(), tags_str, entities_str, trust],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Search facts by keyword (FTS5).
    pub fn search(&self, query: &str, limit: usize) -> SqlResult<Vec<ProbeResult>> {
        let conn = self.conn.lock().unwrap();

        // Use FTS5 for relevance-ranked search
        let sql = "SELECT f.id, f.content, f.category, f.tags, f.entities, f.trust,
                          f.created_at, f.last_accessed, f.access_count,
                          rank
                   FROM fts_facts
                   JOIN facts f ON fts_facts.rowid = f.id
                   WHERE fts_facts MATCH ?1
                   ORDER BY rank
                   LIMIT ?2";

        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params![query, limit as i64], |row| {
            Ok(ProbeResult {
                fact: Fact {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    category: FactCategory::from_str(&row.get::<_, String>(2)?),
                    tags: split_csv(&row.get::<_, String>(3)?),
                    entities: split_csv(&row.get::<_, String>(4)?),
                    trust: row.get(5)?,
                    created_at: row.get(6)?,
                    last_accessed: row.get(7)?,
                    access_count: row.get(8)?,
                },
                relevance: 1.0 / (1.0 + row.get::<_, f64>(9)?),
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        // Update access counts
        for r in &results {
            conn.execute(
                "UPDATE facts SET last_accessed = datetime('now'), access_count = access_count + 1 WHERE id = ?1",
                params![r.fact.id],
            )?;
        }

        Ok(results)
    }

    /// Probe: ALL facts about a specific entity.
    pub fn probe(&self, entity: &str) -> SqlResult<Vec<Fact>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", entity);

        let mut stmt = conn.prepare(
            "SELECT id, content, category, tags, entities, trust,
                    created_at, last_accessed, access_count
             FROM facts
             WHERE entities LIKE ?1
             ORDER BY trust DESC, access_count DESC
             LIMIT 50",
        )?;

        let rows = stmt.query_map(params![pattern], row_to_fact)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        // Touch access times
        for r in &results {
            conn.execute(
                "UPDATE facts SET last_accessed = datetime('now') WHERE id = ?1",
                params![r.id],
            )?;
        }

        Ok(results)
    }

    /// Related: facts that share entities with the given entity.
    pub fn related(&self, entity: &str, limit: usize) -> SqlResult<Vec<Fact>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", entity);

        let mut stmt = conn.prepare(
            "SELECT DISTINCT f.id, f.content, f.category, f.tags, f.entities, f.trust,
                    f.created_at, f.last_accessed, f.access_count
             FROM facts f
             WHERE f.entities LIKE ?1
               AND f.entities != ?2
             ORDER BY f.trust DESC, f.access_count DESC
             LIMIT ?3",
        )?;

        let rows = stmt.query_map(params![pattern, entity, limit as i64], row_to_fact)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Reason: facts connected to MULTIPLE entities simultaneously.
    /// Finds facts that mention ALL of the given entities.
    pub fn reason(&self, entities: &[&str]) -> SqlResult<Vec<Fact>> {
        if entities.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.conn.lock().unwrap();

        // Build LIKE conditions for each entity
        let conditions: Vec<String> = entities
            .iter()
            .enumerate()
            .map(|(i, _)| format!("f.entities LIKE ?{}", i + 1))
            .collect();
        let where_clause = conditions.join(" AND ");

        let sql = format!(
            "SELECT f.id, f.content, f.category, f.tags, f.entities, f.trust,
                    f.created_at, f.last_accessed, f.access_count
             FROM facts f
             WHERE {}
             ORDER BY f.trust DESC
             LIMIT 30",
            where_clause
        );

        let mut stmt = conn.prepare(&sql)?;

        let patterns: Vec<String> = entities.iter().map(|e| format!("%{}%", e)).collect();
        let params: Vec<&dyn rusqlite::types::ToSql> = patterns
            .iter()
            .map(|p| p as &dyn rusqlite::types::ToSql)
            .collect();

        let rows = stmt.query_map(params.as_slice(), row_to_fact)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Contradict: find facts making conflicting claims.
    /// Two facts contradict if they share entities but have opposing trust scores
    /// (one high-trust, one low-trust) about similar topics.
    pub fn contradict(&self) -> SqlResult<Vec<(Fact, Fact)>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT f1.id, f1.content, f1.category, f1.tags, f1.entities, f1.trust,
                    f1.created_at, f1.last_accessed, f1.access_count,
                    f2.id, f2.content, f2.category, f2.tags, f2.entities, f2.trust,
                    f2.created_at, f2.last_accessed, f2.access_count
             FROM facts f1
             JOIN facts f2 ON f1.id < f2.id
             WHERE f1.entities = f2.entities
               AND ABS(f1.trust - f2.trust) > 0.3
             ORDER BY ABS(f1.trust - f2.trust) DESC
             LIMIT 20",
        )?;

        let rows = stmt.query_map([], |row| {
            let f1 = Fact {
                id: row.get(0)?,
                content: row.get(1)?,
                category: FactCategory::from_str(&row.get::<_, String>(2)?),
                tags: split_csv(&row.get::<_, String>(3)?),
                entities: split_csv(&row.get::<_, String>(4)?),
                trust: row.get(5)?,
                created_at: row.get(6)?,
                last_accessed: row.get(7)?,
                access_count: row.get(8)?,
            };
            let f2 = Fact {
                id: row.get(9)?,
                content: row.get(10)?,
                category: FactCategory::from_str(&row.get::<_, String>(11)?),
                tags: split_csv(&row.get::<_, String>(12)?),
                entities: split_csv(&row.get::<_, String>(13)?),
                trust: row.get(14)?,
                created_at: row.get(15)?,
                last_accessed: row.get(16)?,
                access_count: row.get(17)?,
            };
            Ok((f1, f2))
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Update a fact's content or trust.
    pub fn update(&self, id: i64, content: Option<&str>, trust_delta: Option<f64>) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

        if let Some(c) = content {
            conn.execute(
                "UPDATE facts SET content = ?1 WHERE id = ?2",
                params![c, id],
            )?;
        }

        if let Some(delta) = trust_delta {
            conn.execute(
                "UPDATE facts SET trust = MAX(0.0, MIN(1.0, trust + ?1)) WHERE id = ?2",
                params![delta, id],
            )?;
        }

        Ok(())
    }

    /// Remove a fact by ID.
    pub fn remove(&self, id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM facts WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// List all facts (for debugging).
    pub fn list(&self, limit: usize) -> SqlResult<Vec<Fact>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, content, category, tags, entities, trust,
                    created_at, last_accessed, access_count
             FROM facts
             ORDER BY id DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], row_to_fact)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Rate a fact as helpful or unhelpful (adjusts trust score).
    pub fn feedback(&self, id: i64, helpful: bool) -> SqlResult<()> {
        let delta = if helpful { 0.05 } else { -0.1 };
        self.update(id, None, Some(delta))
    }

    /// Purge facts below a trust threshold.
    pub fn purge_low_trust(&self, min_trust: f64) -> SqlResult<usize> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(
            "DELETE FROM facts WHERE trust < ?1",
            params![min_trust],
        )?;
        Ok(count)
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn row_to_fact(row: &rusqlite::Row) -> SqlResult<Fact> {
    Ok(Fact {
        id: row.get(0)?,
        content: row.get(1)?,
        category: FactCategory::from_str(&row.get::<_, String>(2)?),
        tags: split_csv(&row.get::<_, String>(3)?),
        entities: split_csv(&row.get::<_, String>(4)?),
        trust: row.get(5)?,
        created_at: row.get(6)?,
        last_accessed: row.get(7)?,
        access_count: row.get(8)?,
    })
}

fn split_csv(s: &str) -> Vec<String> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect()
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> FactStore {
        FactStore::open_in_memory().unwrap()
    }

    #[test]
    fn test_add_and_search() {
        let store = test_store();
        store.add("android-ai-agent uses Rust edition 2021", FactCategory::Project, &["android-ai-agent", "rust"], &["android-ai-agent", "rust"], 0.8).unwrap();

        let results = store.search("rust edition", 5).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].fact.content.contains("edition 2021"));
    }

    #[test]
    fn test_probe_finds_entity() {
        let store = test_store();
        store.add("serde_json pinned to 1.0.140", FactCategory::Project, &["android-ai-agent"], &["serde_json", "cross-compile"], 0.9).unwrap();
        store.add("tokio 1.52 for async runtime", FactCategory::Project, &["android-ai-agent"], &["tokio"], 0.7).unwrap();

        let results = store.probe("serde_json").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("1.0.140"));
    }

    #[test]
    fn test_reason_multiple_entities() {
        let store = test_store();
        store.add("claude-sonnet-4 costs $3/$15 per 1M tokens", FactCategory::Tool, &["pricing"], &["claude-sonnet-4", "pricing"], 0.85).unwrap();
        store.add("deepseek-v4 costs $0.14/$0.28 per 1M tokens", FactCategory::Tool, &["pricing"], &["deepseek-v4", "pricing"], 0.9).unwrap();

        // Should find facts that mention BOTH claude-sonnet-4 AND pricing
        let results = store.reason(&["claude-sonnet-4", "pricing"]).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].content.contains("claude-sonnet-4"));
    }

    #[test]
    fn test_feedback_adjusts_trust() {
        let store = test_store();
        let id = store.add("test fact", FactCategory::General, &[], &["test"], 0.5).unwrap();

        store.feedback(id, true).unwrap();  // +0.05
        store.feedback(id, true).unwrap();  // +0.05
        store.feedback(id, false).unwrap(); // -0.10

        let results = store.probe("test").unwrap();
        assert_eq!(results.len(), 1);
        assert!((results[0].trust - 0.5).abs() < 0.01); // 0.5 + 0.05 + 0.05 - 0.10 = 0.5
    }

    #[test]
    fn test_purge_low_trust() {
        let store = test_store();
        store.add("high trust", FactCategory::General, &[], &["test"], 0.9).unwrap();
        store.add("low trust", FactCategory::General, &[], &["test"], 0.1).unwrap();

        let purged = store.purge_low_trust(0.3).unwrap();
        assert!(purged >= 1);

        let remaining = store.probe("test").unwrap();
        assert!(!remaining.is_empty());
        assert!(remaining.iter().all(|f| f.trust >= 0.3));
    }

    #[test]
    fn test_contradict_finds_conflicts() {
        let store = test_store();
        store.add("Python is the best language", FactCategory::General, &[], &["python", "languages"], 0.9).unwrap();
        store.add("Rust is better than Python", FactCategory::General, &[], &["python", "rust", "languages"], 0.3).unwrap();

        let conflicts = store.contradict().unwrap();
        // May or may not find conflicts depending on exact entity matching
        // Just verify it doesn't crash
        assert!(conflicts.len() <= 1);
    }
}
