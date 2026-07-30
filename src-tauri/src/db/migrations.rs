pub struct Migration {
    pub version: i64,
    pub sql: &'static str,
}

pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            sql: "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        },
        Migration {
            version: 2,
            sql: "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                model TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        },
    ]
}
