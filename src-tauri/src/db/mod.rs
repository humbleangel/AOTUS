pub mod migrations;

pub trait MessageDb: Send + Sync {
    fn insert_message(&self, msg: NewMessage) -> Result<MessageRow, crate::ChatError>;
    fn get_messages(&self, session_id: &str) -> Result<Vec<MessageRow>, crate::ChatError>;
    fn delete_message(&self, id: i64) -> Result<(), crate::ChatError>;
    fn get_sessions(&self) -> Result<Vec<SessionRow>, crate::ChatError>;
    fn create_session(&self, id: &str, name: &str) -> Result<SessionRow, crate::ChatError>;
    fn delete_session(&self, id: &str) -> Result<(), crate::ChatError>;
    fn rename_session(&self, id: &str, name: &str) -> Result<(), crate::ChatError>;
}

pub struct NewMessage {
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub tool_calls: Option<String>,
}

#[derive(Clone, serde::Serialize)]
pub struct MessageRow {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub tool_calls: Option<String>,
    pub created_at: String,
}

use std::sync::Mutex;

#[derive(Clone, serde::Serialize)]
pub struct SessionRow {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

pub struct SqliteDb {
    conn: Mutex<rusqlite::Connection>,
}

impl SqliteDb {
    pub fn open(path: &str) -> Result<Self, crate::ChatError> {
        let conn = rusqlite::Connection::open(path)
            .map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let db = Self { conn: Mutex::new(conn) };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self, crate::ChatError> {
        let conn = rusqlite::Connection::open_in_memory()
            .map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let db = Self { conn: Mutex::new(conn) };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), crate::ChatError> {
        let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );"
        ).map_err(|e| crate::ChatError::Db(e.to_string()))?;

        let current: i64 = conn
            .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |row| row.get(0))
            .map_err(|e| crate::ChatError::Db(e.to_string()))?;

        for m in crate::db::migrations::get_migrations() {
            if m.version > current {
                conn.execute_batch(m.sql)
                    .map_err(|e| crate::ChatError::Db(format!("migration {}: {}", m.version, e)))?;
                conn.execute(
                    "INSERT INTO schema_version (version) VALUES (?1)",
                    rusqlite::params![m.version],
                ).map_err(|e| crate::ChatError::Db(e.to_string()))?;
            }
        }
        Ok(())
    }

}

impl MessageDb for SqliteDb {
    fn insert_message(&self, msg: NewMessage) -> Result<MessageRow, crate::ChatError> {
        let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let created_at = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO messages (session_id, role, content, model, tool_calls, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![msg.session_id, msg.role, msg.content, msg.model, msg.tool_calls, created_at],
        ).map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let id = conn.last_insert_rowid();
        Ok(MessageRow {
            id,
            session_id: msg.session_id,
            role: msg.role,
            content: msg.content,
            model: msg.model,
            tool_calls: msg.tool_calls,
            created_at,
        })
    }

    fn get_messages(&self, session_id: &str) -> Result<Vec<MessageRow>, crate::ChatError> {
        let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, model, tool_calls, created_at FROM messages WHERE session_id = ?1 ORDER BY id ASC"
        ).map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let rows = stmt.query_map(rusqlite::params![session_id], |row| {
            Ok(MessageRow {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                model: row.get(4)?,
                tool_calls: row.get(5)?,
                created_at: row.get(6)?,
            })
        }).map_err(|e| crate::ChatError::Db(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| crate::ChatError::Db(e.to_string()))
    }

    fn delete_message(&self, id: i64) -> Result<(), crate::ChatError> {
        let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        conn.execute("DELETE FROM messages WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| crate::ChatError::Db(e.to_string()))?;
        Ok(())
    }

    fn get_sessions(&self) -> Result<Vec<SessionRow>, crate::ChatError> {
        let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT id, name, created_at FROM sessions ORDER BY created_at DESC")
            .map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let rows = stmt.query_map([], |row| {
            Ok(SessionRow { id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)? })
        }).map_err(|e| crate::ChatError::Db(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| crate::ChatError::Db(e.to_string()))
    }

    fn create_session(&self, id: &str, name: &str) -> Result<SessionRow, crate::ChatError> {
        let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let created_at = chrono::Utc::now().to_rfc3339();
        conn.execute("INSERT INTO sessions (id, name, created_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, name, created_at])
            .map_err(|e| crate::ChatError::Db(e.to_string()))?;
        Ok(SessionRow { id: id.into(), name: name.into(), created_at })
    }

    fn delete_session(&self, id: &str) -> Result<(), crate::ChatError> {
        let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        conn.execute("DELETE FROM messages WHERE session_id = ?1", rusqlite::params![id])
            .map_err(|e| crate::ChatError::Db(e.to_string()))?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| crate::ChatError::Db(e.to_string()))?;
        Ok(())
    }

    fn rename_session(&self, id: &str, name: &str) -> Result<(), crate::ChatError> {
        let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        conn.execute("UPDATE sessions SET name = ?1 WHERE id = ?2", rusqlite::params![name, id])
            .map_err(|e| crate::ChatError::Db(e.to_string()))?;
        Ok(())
    }
}

pub struct InMemoryMessageDb {
    messages: Mutex<Vec<MessageRow>>,
    sessions: Mutex<Vec<SessionRow>>,
    next_id: Mutex<i64>,
}

impl InMemoryMessageDb {
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(vec![]),
            sessions: Mutex::new(vec![]),
            next_id: Mutex::new(1),
        }
    }
}

impl MessageDb for InMemoryMessageDb {
    fn insert_message(&self, msg: NewMessage) -> Result<MessageRow, crate::ChatError> {
        let mut msgs = self.messages.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let mut id_counter = self.next_id.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let id = *id_counter;
        *id_counter += 1;
        let row = MessageRow {
            id,
            session_id: msg.session_id,
            role: msg.role,
            content: msg.content,
            model: msg.model,
            tool_calls: msg.tool_calls,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        msgs.push(row.clone());
        Ok(row)
    }

    fn get_messages(&self, session_id: &str) -> Result<Vec<MessageRow>, crate::ChatError> {
        let msgs = self.messages.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        Ok(msgs.iter().filter(|m| m.session_id == session_id).cloned().collect())
    }

    fn delete_message(&self, id: i64) -> Result<(), crate::ChatError> {
        let mut msgs = self.messages.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        msgs.retain(|m| m.id != id);
        Ok(())
    }

    fn get_sessions(&self) -> Result<Vec<SessionRow>, crate::ChatError> {
        let sessions = self.sessions.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let mut list = sessions.clone();
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(list)
    }

    fn create_session(&self, id: &str, name: &str) -> Result<SessionRow, crate::ChatError> {
        let mut sessions = self.sessions.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        let row = SessionRow {
            id: id.into(),
            name: name.into(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        sessions.push(row.clone());
        Ok(row)
    }

    fn delete_session(&self, id: &str) -> Result<(), crate::ChatError> {
        let mut msgs = self.messages.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        msgs.retain(|m| m.session_id != id);
        let mut sessions = self.sessions.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        sessions.retain(|s| s.id != id);
        Ok(())
    }

    fn rename_session(&self, id: &str, name: &str) -> Result<(), crate::ChatError> {
        let mut sessions = self.sessions.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
        if let Some(s) = sessions.iter_mut().find(|s| s.id == id) {
            s.name = name.to_string();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDb;

    impl MessageDb for MockDb {
        fn insert_message(&self, _msg: NewMessage) -> Result<MessageRow, crate::ChatError> {
            Err(crate::ChatError::Db("not implemented".into()))
        }
        fn get_messages(&self, _session_id: &str) -> Result<Vec<MessageRow>, crate::ChatError> {
            Ok(vec![])
        }
        fn delete_message(&self, _id: i64) -> Result<(), crate::ChatError> {
            Ok(())
        }
        fn get_sessions(&self) -> Result<Vec<SessionRow>, crate::ChatError> {
            Ok(vec![])
        }
        fn create_session(&self, _id: &str, _name: &str) -> Result<SessionRow, crate::ChatError> {
            Err(crate::ChatError::Db("not implemented".into()))
        }
        fn delete_session(&self, _id: &str) -> Result<(), crate::ChatError> {
            Ok(())
        }
        fn rename_session(&self, _id: &str, _name: &str) -> Result<(), crate::ChatError> {
            Ok(())
        }
    }

    #[test]
    fn test_message_db_trait_compiles() {
        let db = MockDb;
        let msg = NewMessage {
            session_id: "s1".into(),
            role: "user".into(),
            content: "hi".into(),
            model: None,
            tool_calls: None,
        };
        assert!(db.insert_message(msg).is_err());
        assert!(db.get_messages("s1").unwrap().is_empty());
        assert!(db.delete_message(1).is_ok());
    }

    #[test]
    fn test_in_memory_insert_and_get() {
        let db = InMemoryMessageDb::new();
        let msg = NewMessage {
            session_id: "s1".into(),
            role: "user".into(),
            content: "hello".into(),
            model: None,
            tool_calls: None,
        };
        let row = db.insert_message(msg).unwrap();
        assert_eq!(row.id, 1);
        assert_eq!(row.role, "user");

        let msgs = db.get_messages("s1").unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "hello");
    }

    #[test]
    fn test_in_memory_get_empty_session() {
        let db = InMemoryMessageDb::new();
        let msgs = db.get_messages("nonexistent").unwrap();
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_in_memory_delete() {
        let db = InMemoryMessageDb::new();
        let row = db.insert_message(NewMessage {
            session_id: "s1".into(), role: "user".into(),
            content: "x".into(), model: None, tool_calls: None,
        }).unwrap();

        db.delete_message(row.id).unwrap();
        assert!(db.get_messages("s1").unwrap().is_empty());
    }

    #[test]
    fn test_sqlite_migration_creates_tables() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.create_session("s1", "Test").unwrap();
        db.insert_message(NewMessage {
            session_id: "s1".into(), role: "user".into(),
            content: "hi".into(), model: None, tool_calls: None,
        }).unwrap();
        assert_eq!(db.get_sessions().unwrap().len(), 1);
        assert_eq!(db.get_messages("s1").unwrap().len(), 1);
    }

    #[test]
    fn test_sqlite_create_and_get_sessions() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.create_session("s1", "Test Session").unwrap();
        let sessions = db.get_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "Test Session");
    }

    #[test]
    fn test_sqlite_insert_and_get_messages() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.create_session("s1", "Test").unwrap();

        let msg = db.insert_message(NewMessage {
            session_id: "s1".into(),
            role: "user".into(),
            content: "hello".into(),
            model: None,
            tool_calls: None,
        }).unwrap();
        assert_eq!(msg.role, "user");

        let msgs = db.get_messages("s1").unwrap();
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn test_sqlite_delete_session_cascades() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.create_session("s1", "Test").unwrap();
        db.insert_message(NewMessage {
            session_id: "s1".into(), role: "user".into(),
            content: "x".into(), model: None, tool_calls: None,
        }).unwrap();

        db.delete_session("s1").unwrap();
        assert!(db.get_sessions().unwrap().is_empty());
        assert!(db.get_messages("s1").unwrap().is_empty());
    }

    #[test]
    fn test_sqlite_migration_idempotent() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.run_migrations().unwrap();
        let sessions = db.get_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_in_memory_auto_increment() {
        let db = InMemoryMessageDb::new();
        let a = db.insert_message(NewMessage {
            session_id: "s1".into(), role: "user".into(),
            content: "a".into(), model: None, tool_calls: None,
        }).unwrap();
        let b = db.insert_message(NewMessage {
            session_id: "s1".into(), role: "assistant".into(),
            content: "b".into(), model: None, tool_calls: None,
        }).unwrap();
        assert_eq!(a.id, 1);
        assert_eq!(b.id, 2);
    }
}
