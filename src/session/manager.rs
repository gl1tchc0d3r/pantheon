use super::session::{Session, SessionSummary};
use super::store::{SessionError, SessionStore};
use crate::provider::LlmProvider;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SessionConfig {
    #[serde(default = "default_max_history_tokens")]
    pub max_history_tokens: usize,

    #[serde(default = "default_db_path")]
    pub db_path: String,

    #[serde(default = "default_auto_resume")]
    pub auto_resume: bool,

    #[serde(default = "default_summarize_on_close")]
    pub summarize_on_close: bool,
}

fn default_max_history_tokens() -> usize {
    8000
}

fn default_db_path() -> String {
    ".ao/sessions.db".to_string()
}

fn default_auto_resume() -> bool {
    true
}

fn default_summarize_on_close() -> bool {
    true
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_history_tokens: default_max_history_tokens(),
            db_path: default_db_path(),
            auto_resume: default_auto_resume(),
            summarize_on_close: default_summarize_on_close(),
        }
    }
}

pub struct SessionManager {
    store: SessionStore,
    current_session: Option<Session>,
    config: SessionConfig,
}

impl SessionManager {
    pub async fn new(config: SessionConfig) -> Result<Self, SessionError> {
        let db_path = if config.db_path.starts_with('/') {
            PathBuf::from(&config.db_path)
        } else {
            std::env::current_dir()?.join(&config.db_path)
        };

        let store = SessionStore::new(&db_path).await?;

        Ok(Self {
            store,
            current_session: None,
            config,
        })
    }

    pub fn current_session(&self) -> Option<&Session> {
        self.current_session.as_ref()
    }

    pub fn current_session_mut(&mut self) -> Option<&mut Session> {
        self.current_session.as_mut()
    }

    pub async fn start_session(&mut self) -> Result<&Session, SessionError> {
        let session = Session::new(Session::generate_id(), self.config.max_history_tokens);
        self.store.create_session(&session).await?;
        self.current_session = Some(session);
        Ok(self.current_session.as_ref().unwrap())
    }

    pub async fn resume_last_session(&mut self) -> Result<Option<&Session>, SessionError> {
        if let Some(mut session) = self.store.get_last_session().await? {
            session.max_history_tokens = self.config.max_history_tokens;
            self.current_session = Some(session);
            Ok(self.current_session.as_ref())
        } else {
            Ok(None)
        }
    }

    pub async fn add_message(&mut self, role: &str, content: &str) -> Result<(), SessionError> {
        if let Some(ref mut session) = self.current_session {
            session.add_message(role, content);

            let messages = &session.messages;
            if let Some(msg) = messages.last() {
                self.store.save_message(msg).await?;
            }
        }
        Ok(())
    }

    pub fn clear_history(&mut self) {
        if let Some(ref mut session) = self.current_session {
            session.clear_history();
        }
    }

    pub async fn end_session(&mut self, llm: &dyn LlmProvider) -> Result<Option<String>, SessionError> {
        if let Some(ref mut session) = self.current_session {
            if self.config.summarize_on_close && !session.messages.is_empty() {
                let transcript = session
                    .messages
                    .iter()
                    .map(|m| {
                        let role = super::session::capitalize(&m.role);
                        format!("{}: {}", role, m.content)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                let prompt = format!(
                    "Summarize the following conversation in 2-3 paragraphs. \
                    Focus on: main topics discussed, key decisions made, and any \
                    pending or unfinished tasks.\n\n\
                    === Conversation Transcript ===\n\
                    {}\n\
                    === End Transcript ===\n\n\
                    Summary:",
                    transcript
                );

                match llm.complete(&prompt).await {
                    Ok(summary) => {
                        let summary = summary.trim().to_string();
                        let token_count = super::session::estimate_tokens(&summary);
                        session.summary = Some(summary.clone());
                        session.summary_tokens = token_count;
                        self.store.save_summary(&session.id, &summary, token_count).await?;
                        return Ok(Some(summary));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to generate session summary: {:?}", e);
                    }
                }
            }
        }
        self.current_session = None;
        Ok(None)
    }

    pub async fn get_previous_summaries(&self) -> Result<Vec<SessionSummary>, SessionError> {
        self.store
            .get_recent_summaries(self.config.max_history_tokens)
            .await
    }

    pub fn get_config(&self) -> &SessionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{LlmProvider, ProviderError};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    struct MockLlm {
        response: String,
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl MockLlm {
        fn new(response: &str) -> Self {
            Self {
                response: response.to_string(),
                calls: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl LlmProvider for MockLlm {
        async fn complete(&self, prompt: &str) -> Result<String, ProviderError> {
            self.calls.lock().unwrap().push(prompt.to_string());
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn test_start_session() {
        let config = SessionConfig::default();
        let mut manager = SessionManager::new(config).await.unwrap();

        let session = manager.start_session().await.unwrap();
        assert!(!session.id.is_empty());
        assert_eq!(session.messages.len(), 0);
    }

    #[tokio::test]
    async fn test_add_message() {
        let config = SessionConfig::default();
        let mut manager = SessionManager::new(config).await.unwrap();
        manager.start_session().await.unwrap();

        manager.add_message("user", "Hello").await.unwrap();
        manager.add_message("ao", "Hi there!").await.unwrap();

        let session = manager.current_session().unwrap();
        assert_eq!(session.messages.len(), 2);
    }

    #[tokio::test]
    async fn test_clear_history() {
        let config = SessionConfig::default();
        let mut manager = SessionManager::new(config).await.unwrap();
        manager.start_session().await.unwrap();

        manager.add_message("user", "Hello").await.unwrap();
        manager.clear_history();

        let session = manager.current_session().unwrap();
        assert_eq!(session.messages.len(), 0);
    }

    #[tokio::test]
    async fn test_end_session_with_summarization() {
        let config = SessionConfig {
            summarize_on_close: true,
            ..Default::default()
        };
        let mut manager = SessionManager::new(config).await.unwrap();
        manager.start_session().await.unwrap();

        manager.add_message("user", "Hello").await.unwrap();
        manager.add_message("ao", "Hi there!").await.unwrap();

        let mock_llm = MockLlm::new("Test summary of the conversation.");
        let summary = manager.end_session(&mock_llm).await.unwrap();

        assert!(summary.is_some());
        assert_eq!(summary.unwrap(), "Test summary of the conversation.");
    }

    #[tokio::test]
    async fn test_end_session_without_summarization() {
        let config = SessionConfig {
            summarize_on_close: false,
            ..Default::default()
        };
        let mut manager = SessionManager::new(config).await.unwrap();
        manager.start_session().await.unwrap();

        manager.add_message("user", "Hello").await.unwrap();

        let mock_llm = MockLlm::new("Should not be called");
        let summary = manager.end_session(&mock_llm).await.unwrap();

        assert!(summary.is_none());
    }
}
