pub trait MessageDb: Send + Sync {
    fn insert_message(&self, msg: NewMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<MessageRow, crate::ChatError>> + Send + '_>>;
    fn get_messages(&self, session_id: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<MessageRow>, crate::ChatError>> + Send + '_>>;
    fn get_sessions(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<SessionRow>, crate::ChatError>> + Send + '_>>;
    fn create_session(&self, id: &str, name: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SessionRow, crate::ChatError>> + Send + '_>>;
    fn delete_session(&self, id: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), crate::ChatError>> + Send + '_>>;
}

pub struct NewMessage {
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
}

#[derive(Clone, serde::Serialize)]
pub struct MessageRow {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub model: Option<String>,
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
    fn insert_message(&self, msg: NewMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<MessageRow, crate::ChatError>> + Send + '_>> {
        Box::pin(async move {
            let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
            let created_at = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO messages (session_id, role, content, model, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![msg.session_id, msg.role, msg.content, msg.model, created_at],
            ).map_err(|e| crate::ChatError::Db(e.to_string()))?;
            let id = conn.last_insert_rowid();
            Ok(MessageRow {
                id,
                session_id: msg.session_id,
                role: msg.role,
                content: msg.content,
                model: msg.model,
                created_at,
            })
        })
    }

    fn get_messages(&self, session_id: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<MessageRow>, crate::ChatError>> + Send + '_>> {
        let session_id = session_id.to_string();
        Box::pin(async move {
            let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
            let mut stmt = conn.prepare(
                "SELECT id, session_id, role, content, model, created_at FROM messages WHERE session_id = ?1 ORDER BY id ASC"
            ).map_err(|e| crate::ChatError::Db(e.to_string()))?;
            let rows = stmt.query_map(rusqlite::params![session_id], |row| {
                Ok(MessageRow {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    model: row.get(4)?,
                    created_at: row.get(5)?,
                })
            }).map_err(|e| crate::ChatError::Db(e.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| crate::ChatError::Db(e.to_string()))
        })
    }

    fn get_sessions(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<SessionRow>, crate::ChatError>> + Send + '_>> {
        Box::pin(async move {
            let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
            let mut stmt = conn.prepare("SELECT id, name, created_at FROM sessions ORDER BY created_at DESC")
                .map_err(|e| crate::ChatError::Db(e.to_string()))?;
            let rows = stmt.query_map([], |row| {
                Ok(SessionRow { id: row.get(0)?, name: row.get(1)?, created_at: row.get(2)? })
            }).map_err(|e| crate::ChatError::Db(e.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| crate::ChatError::Db(e.to_string()))
        })
    }

    fn create_session(&self, id: &str, name: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SessionRow, crate::ChatError>> + Send + '_>> {
        let id = id.to_string();
        let name = name.to_string();
        Box::pin(async move {
            let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
            let created_at = chrono::Utc::now().to_rfc3339();
            conn.execute("INSERT INTO sessions (id, name, created_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![id, name, created_at])
                .map_err(|e| crate::ChatError::Db(e.to_string()))?;
            Ok(SessionRow { id: id.into(), name: name.into(), created_at })
        })
    }

    fn delete_session(&self, id: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), crate::ChatError>> + Send + '_>> {
        let id = id.to_string();
        Box::pin(async move {
            let conn = self.conn.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
            conn.execute("DELETE FROM messages WHERE session_id = ?1", rusqlite::params![id])
                .map_err(|e| crate::ChatError::Db(e.to_string()))?;
            conn.execute("DELETE FROM sessions WHERE id = ?1", rusqlite::params![id])
                .map_err(|e| crate::ChatError::Db(e.to_string()))?;
            Ok(())
        })
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
    fn insert_message(&self, msg: NewMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<MessageRow, crate::ChatError>> + Send + '_>> {
        Box::pin(async move {
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
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            msgs.push(row.clone());
            Ok(row)
        })
    }

    fn get_messages(&self, session_id: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<MessageRow>, crate::ChatError>> + Send + '_>> {
        let session_id = session_id.to_string();
        Box::pin(async move {
            let msgs = self.messages.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
            Ok(msgs.iter().filter(|m| m.session_id == session_id).cloned().collect())
        })
    }

    fn get_sessions(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<SessionRow>, crate::ChatError>> + Send + '_>> {
        Box::pin(async move {
            let sessions = self.sessions.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
            let mut list = sessions.clone();
            list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            Ok(list)
        })
    }

    fn create_session(&self, id: &str, name: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SessionRow, crate::ChatError>> + Send + '_>> {
        let id = id.to_string();
        let name = name.to_string();
        Box::pin(async move {
            let mut sessions = self.sessions.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
            let row = SessionRow {
                id: id.into(),
                name: name.into(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            sessions.push(row.clone());
            Ok(row)
        })
    }

    fn delete_session(&self, id: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), crate::ChatError>> + Send + '_>> {
        let id = id.to_string();
        Box::pin(async move {
            let mut msgs = self.messages.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
            msgs.retain(|m| m.session_id != id);
            let mut sessions = self.sessions.lock().map_err(|e| crate::ChatError::Db(e.to_string()))?;
            sessions.retain(|s| s.id != id);
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDb;

    impl MessageDb for MockDb {
        fn insert_message(&self, _msg: NewMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<MessageRow, crate::ChatError>> + Send + '_>> {
            Box::pin(async { Err(crate::ChatError::Db("not implemented".into())) })
        }
        fn get_messages(&self, _session_id: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<MessageRow>, crate::ChatError>> + Send + '_>> {
            Box::pin(async { Ok(vec![]) })
        }
        fn get_sessions(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<SessionRow>, crate::ChatError>> + Send + '_>> {
            Box::pin(async { Ok(vec![]) })
        }
        fn create_session(&self, _id: &str, _name: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SessionRow, crate::ChatError>> + Send + '_>> {
            Box::pin(async { Err(crate::ChatError::Db("not implemented".into())) })
        }
        fn delete_session(&self, _id: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), crate::ChatError>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }
    }

    #[tokio::test]
    async fn test_message_db_trait_compiles() {
        let db = MockDb;
        assert!(db.insert_message(NewMessage {
            session_id: "s1".into(), role: "user".into(),
            content: "hi".into(), model: None,
        }).await.is_err());
    }

    #[tokio::test]
    async fn test_in_memory_insert_and_get() {
        let db = InMemoryMessageDb::new();
        let row = db.insert_message(NewMessage {
            session_id: "s1".into(), role: "user".into(),
            content: "hello".into(), model: None,
        }).await.unwrap();
        assert_eq!(row.id, 1);
        let msgs = db.get_messages("s1").await.unwrap();
        assert_eq!(msgs.len(), 1);
    }

    #[tokio::test]
    async fn test_in_memory_sessions() {
        let db = InMemoryMessageDb::new();
        db.create_session("s1", "Test").await.unwrap();
        assert_eq!(db.get_sessions().await.unwrap().len(), 1);
        db.delete_session("s1").await.unwrap();
        assert!(db.get_sessions().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_sqlite_create_and_get_sessions() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.create_session("s1", "Test").await.unwrap();
        assert_eq!(db.get_sessions().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_sqlite_insert_and_get_messages() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.create_session("s1", "Test").await.unwrap();
        let msg = db.insert_message(NewMessage {
            session_id: "s1".into(), role: "user".into(),
            content: "hello".into(), model: None,
        }).await.unwrap();
        assert_eq!(msg.role, "user");
        assert_eq!(db.get_messages("s1").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_sqlite_delete_session_cascades() {
        let db = SqliteDb::open_in_memory().unwrap();
        db.create_session("s1", "Test").await.unwrap();
        db.insert_message(NewMessage {
            session_id: "s1".into(), role: "user".into(),
            content: "x".into(), model: None,
        }).await.unwrap();
        db.delete_session("s1").await.unwrap();
        assert!(db.get_sessions().await.unwrap().is_empty());
    }
}
