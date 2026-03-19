use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePool, Row};
use std::path::Path;

use super::session::{Message, Session, SessionSummary};

#[derive(Debug)]
pub enum SessionError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::DatabaseError(e) => write!(f, "Database error: {}", e),
            SessionError::NotFound(id) => write!(f, "Session not found: {}", id),
            SessionError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<sqlx::Error> for SessionError {
    fn from(err: sqlx::Error) -> Self {
        SessionError::DatabaseError(err)
    }
}

impl From<std::io::Error> for SessionError {
    fn from(err: std::io::Error) -> Self {
        SessionError::IoError(err)
    }
}

pub struct SessionStore {
    pool: SqlitePool,
}

impl SessionStore {
    pub async fn new(db_path: &Path) -> Result<Self, SessionError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&database_url).await?;

        let store = Self { pool };
        store.init_schema().await?;

        Ok(store)
    }

    async fn init_schema(&self) -> Result<(), SessionError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT,
                created_at TEXT NOT NULL,
                last_active_at TEXT NOT NULL,
                summary TEXT,
                summary_tokens INTEGER DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, created_at)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_sessions_last_active ON sessions(last_active_at DESC)",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_session(&self, session: &Session) -> Result<(), SessionError> {
        sqlx::query(
            r#"
            INSERT INTO sessions (id, name, created_at, last_active_at, summary, summary_tokens)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(&session.name)
        .bind(session.created_at.to_rfc3339())
        .bind(session.last_active_at.to_rfc3339())
        .bind(&session.summary)
        .bind(session.summary_tokens as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_session(&self, id: &str) -> Result<Option<Session>, SessionError> {
        let row = sqlx::query("SELECT * FROM sessions WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let session_id: String = row.get("id");
                let messages = self.get_messages(&session_id).await?;

                Ok(Some(Session {
                    id: session_id,
                    name: row.get("name"),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                        .unwrap()
                        .with_timezone(&Utc),
                    last_active_at: DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("last_active_at"),
                    )
                    .unwrap()
                    .with_timezone(&Utc),
                    summary: row.get("summary"),
                    summary_tokens: row.get::<i64, _>("summary_tokens") as usize,
                    messages,
                    max_history_tokens: 8000,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn get_last_session(&self) -> Result<Option<Session>, SessionError> {
        let row = sqlx::query(
            "SELECT * FROM sessions ORDER BY last_active_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let session_id: String = row.get("id");
                let messages = self.get_messages(&session_id).await?;

                Ok(Some(Session {
                    id: session_id,
                    name: row.get("name"),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                        .unwrap()
                        .with_timezone(&Utc),
                    last_active_at: DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("last_active_at"),
                    )
                    .unwrap()
                    .with_timezone(&Utc),
                    summary: row.get("summary"),
                    summary_tokens: row.get::<i64, _>("summary_tokens") as usize,
                    messages,
                    max_history_tokens: 8000,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn save_message(&self, message: &Message) -> Result<(), SessionError> {
        sqlx::query(
            r#"
            INSERT INTO messages (session_id, role, content, created_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&message.session_id)
        .bind(&message.role)
        .bind(&message.content)
        .bind(message.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>, SessionError> {
        let rows = sqlx::query(
            "SELECT * FROM messages WHERE session_id = ? ORDER BY created_at ASC",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        let messages = rows
            .into_iter()
            .map(|row| Message {
                id: row.get::<i64, _>("id"),
                session_id: row.get("session_id"),
                role: row.get("role"),
                content: row.get("content"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
            })
            .collect();

        Ok(messages)
    }

    pub async fn update_last_active(&self, session_id: &str) -> Result<(), SessionError> {
        sqlx::query(
            "UPDATE sessions SET last_active_at = ? WHERE id = ?",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_summary(&self, session_id: &str, summary: &str, tokens: usize) -> Result<(), SessionError> {
        sqlx::query(
            "UPDATE sessions SET summary = ?, summary_tokens = ? WHERE id = ?",
        )
        .bind(summary)
        .bind(tokens as i64)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_recent_summaries(&self, max_tokens: usize) -> Result<Vec<SessionSummary>, SessionError> {
        let rows = sqlx::query(
            r#"
            SELECT id, created_at, summary, summary_tokens 
            FROM sessions 
            WHERE summary IS NOT NULL 
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut summaries: Vec<SessionSummary> = Vec::new();
        let mut total_tokens: usize = 0;

        for row in rows {
            let summary_text: String = row.get("summary");
            let token_count: usize = row.get::<i64, _>("summary_tokens") as usize;

            if total_tokens + token_count > max_tokens && !summaries.is_empty() {
                break;
            }

            total_tokens += token_count;

            summaries.push(SessionSummary {
                session_id: row.get("id"),
                session_date: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                summary: summary_text,
                token_count,
            });
        }

        Ok(summaries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_store() -> (SessionStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SessionStore::new(&db_path).await.unwrap();
        (store, dir)
    }

    #[tokio::test]
    async fn test_create_and_load_session() {
        let (store, _dir) = create_test_store().await;

        let session = Session::new("test-session-1".to_string(), 8000);
        store.create_session(&session).await.unwrap();

        let loaded = store.get_session("test-session-1").await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, "test-session-1");
    }

    #[tokio::test]
    async fn test_save_and_retrieve_messages() {
        let (store, _dir) = create_test_store().await;

        let session = Session::new("test-session-2".to_string(), 8000);
        store.create_session(&session).await.unwrap();

        let message = Message::new("test-session-2".to_string(), "user", "Hello!");
        store.save_message(&message).await.unwrap();

        let messages = store.get_messages("test-session-2").await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello!");
        assert_eq!(messages[0].role, "user");
    }

    #[tokio::test]
    async fn test_save_and_retrieve_summary() {
        let (store, _dir) = create_test_store().await;

        let session = Session::new("test-session-3".to_string(), 8000);
        store.create_session(&session).await.unwrap();

        store.save_summary("test-session-3", "Test summary", 100).await.unwrap();

        let loaded = store.get_session("test-session-3").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().summary, Some("Test summary".to_string()));
    }

    #[tokio::test]
    async fn test_get_recent_summaries_within_token_limit() {
        let (store, _dir) = create_test_store().await;

        for i in 0..5 {
            let session = Session::new(format!("test-session-{}", i), 8000);
            store.create_session(&session).await.unwrap();
            store.save_summary(&session.id, &format!("Summary {}", i), 100).await.unwrap();
        }

        let summaries = store.get_recent_summaries(250).await.unwrap();
        assert!(summaries.len() <= 3);
    }
}
