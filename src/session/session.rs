use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub summary: Option<String>,
    pub summary_tokens: usize,
    pub messages: Vec<Message>,
    pub max_history_tokens: usize,
}

impl Session {
    pub fn new(id: String, max_history_tokens: usize) -> Self {
        let now = Utc::now();
        Self {
            id,
            name: None,
            created_at: now,
            last_active_at: now,
            summary: None,
            summary_tokens: 0,
            messages: Vec::new(),
            max_history_tokens,
        }
    }

    pub fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.last_active_at = Utc::now();
        let message = Message::new(self.id.clone(), role, content);
        self.messages.push(message);
    }

    pub fn clear_history(&mut self) {
        self.messages.clear();
        self.last_active_at = Utc::now();
    }

    pub fn estimate_tokens(&self) -> usize {
        self.messages.iter().map(|m| m.estimate_tokens()).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Message {
    pub fn new(session_id: String, role: &str, content: &str) -> Self {
        Self {
            id: 0,
            session_id,
            role: role.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
        }
    }

    pub fn estimate_tokens(&self) -> usize {
        estimate_tokens(&self.content)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub session_date: DateTime<Utc>,
    pub summary: String,
    pub token_count: usize,
}

impl SessionSummary {
    pub fn new(session: &Session, summary_text: String) -> Self {
        Self {
            session_id: session.id.clone(),
            session_date: session.created_at,
            summary: summary_text,
            token_count: 0,
        }
    }

    pub fn estimate_tokens(&self) -> usize {
        estimate_tokens(&self.summary)
    }
}

pub fn estimate_tokens(text: &str) -> usize {
    let word_count: usize = text.split_whitespace().count();
    let line_count: usize = text.lines().count();
    (word_count as f32 * 1.3) as usize + (line_count * 4)
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let session = Session::new("test-id".to_string(), 8000);
        assert!(session.messages.is_empty());
        assert_eq!(session.id, "test-id");
        assert_eq!(session.max_history_tokens, 8000);
    }

    #[test]
    fn test_add_message() {
        let mut session = Session::new("test".to_string(), 8000);
        session.add_message("user", "Hello");
        session.add_message("ao", "Hi there!");
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].role, "user");
        assert_eq!(session.messages[1].role, "ao");
    }

    #[test]
    fn test_token_estimation() {
        let msg = Message::new("s1".to_string(), "user", "Hello world");
        let tokens = msg.estimate_tokens();
        assert!(tokens > 0);
    }

    #[test]
    fn test_clear_history() {
        let mut session = Session::new("test".to_string(), 8000);
        session.add_message("user", "Hello");
        session.clear_history();
        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_session_generate_id() {
        let id1 = Session::generate_id();
        let id2 = Session::generate_id();
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36);
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("user"), "User");
        assert_eq!(capitalize("ao"), "Ao");
        assert_eq!(capitalize(""), "");
    }
}
