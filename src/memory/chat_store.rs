//! Chat history database.
//! SQLite-backed storage for unified chat messages.

use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;
use std::sync::Mutex;
use crate::chat_models::ChatMessage;

pub struct ChatStore {
    conn: Mutex<Connection>,
}

impl ChatStore {
    /// Open or create the chat database at the given path.
    pub fn open(path: impl AsRef<Path>) -> SqlResult<Self> {
        let conn = Connection::open(path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id          TEXT PRIMARY KEY,
                role        TEXT NOT NULL,
                content     TEXT NOT NULL,
                timestamp   INTEGER NOT NULL
            )",
            [],
        )?;

        // Index
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp)",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open in-memory (for testing).
    pub fn open_in_memory() -> SqlResult<Self> {
        Self::open(":memory:")
    }

    /// Add a new message.
    pub fn add(&self, msg: &ChatMessage) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![msg.id, msg.role, msg.content, msg.timestamp],
        )?;
        Ok(())
    }

    /// Get chat history ordered by timestamp.
    pub fn get_history(&self, limit: usize) -> SqlResult<Vec<ChatMessage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, role, content, timestamp
             FROM messages
             ORDER BY timestamp ASC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Clear all messages.
    pub fn clear(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM messages", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_history() {
        let store = ChatStore::open_in_memory().unwrap();
        let msg = ChatMessage {
            id: "1".to_string(),
            role: "user".to_string(),
            content: "hello".to_string(),
            timestamp: 100,
        };
        store.add(&msg).unwrap();

        let history = store.get_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, "hello");
    }
}
