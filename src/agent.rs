use crate::provider::{LlmProvider, ProviderError};
use crate::session::{capitalize, Message, SessionSummary};

pub struct AgentLoop {
    provider: Box<dyn LlmProvider>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum AgentError {
    ProviderError(ProviderError),
}

impl From<ProviderError> for AgentError {
    fn from(err: ProviderError) -> Self {
        AgentError::ProviderError(err)
    }
}

impl AgentLoop {
    pub fn new(provider: Box<dyn LlmProvider>) -> Self {
        Self { provider }
    }

    pub async fn run(
        &self,
        input: &str,
        history: &[&Message],
        previous_summaries: &[SessionSummary],
    ) -> Result<String, AgentError> {
        let prompt = self.build_prompt(input, history, previous_summaries);
        let response = self.provider.complete(&prompt).await?;
        Ok(response)
    }

    fn build_prompt(
        &self,
        input: &str,
        history: &[&Message],
        previous_summaries: &[SessionSummary],
    ) -> String {
        let mut prompt = String::new();

        if !previous_summaries.is_empty() {
            prompt.push_str("=== Previous Sessions Summary ===\n");
            for summary in previous_summaries {
                prompt.push_str(&format!(
                    "Session {}: {}\n",
                    summary.session_date.format("%Y-%m-%d"),
                    summary.summary
                ));
            }
            prompt.push_str("=== End Previous Sessions ===\n\n");
        }

        if !history.is_empty() {
            prompt.push_str("=== Current Session History ===\n");
            for msg in history {
                prompt.push_str(&format!("{}: {}\n", capitalize(&msg.role), msg.content));
            }
            prompt.push_str("=== End Current History ===\n\n");
        }

        prompt.push_str(&format!("User: {}\nAo:", input));

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{LlmProvider, ProviderError};
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_build_prompt_with_history() {
        let mock = MockProvider::new(vec!["Response".to_string()]);
        let agent = AgentLoop::new(Box::new(mock));

        let message = Message::new("s1".to_string(), "user", "Hello");
        let history = vec![&message];
        let summaries: Vec<SessionSummary> = vec![];

        let prompt = agent.build_prompt("Test input", &history, &summaries);

        assert!(prompt.contains("=== Current Session History ==="));
        assert!(prompt.contains("User: Hello"));
        assert!(prompt.contains("Test input"));
    }

    #[test]
    fn test_build_prompt_with_summaries() {
        let mock = MockProvider::new(vec!["Response".to_string()]);
        let agent = AgentLoop::new(Box::new(mock));

        let history: Vec<&Message> = vec![];
        let summaries = vec![SessionSummary {
            session_id: "s1".to_string(),
            session_date: Utc::now(),
            summary: "Discussed Rust programming".to_string(),
            token_count: 100,
        }];

        let prompt = agent.build_prompt("Test input", &history, &summaries);

        assert!(prompt.contains("=== Previous Sessions Summary ==="));
        assert!(prompt.contains("Discussed Rust programming"));
    }

    #[test]
    fn test_build_prompt_empty() {
        let mock = MockProvider::new(vec!["Response".to_string()]);
        let agent = AgentLoop::new(Box::new(mock));

        let history: Vec<&Message> = vec![];
        let summaries: Vec<SessionSummary> = vec![];

        let prompt = agent.build_prompt("Test input", &history, &summaries);

        assert!(prompt.contains("User: Test input"));
        assert!(prompt.contains("Ao:"));
    }

    #[derive(Clone)]
    struct MockProvider {
        responses: Arc<Mutex<Vec<String>>>,
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl MockProvider {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses)),
                calls: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl LlmProvider for MockProvider {
        async fn complete(&self, prompt: &str) -> Result<String, ProviderError> {
            self.calls.lock().unwrap().push(prompt.to_string());
            self.responses
                .lock()
                .unwrap()
                .pop()
                .ok_or(ProviderError::ParseError)
        }
    }
}
