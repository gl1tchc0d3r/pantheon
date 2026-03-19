pub mod agent;
pub mod chat;
pub mod cli;
pub mod config;
pub mod provider;
pub mod session;
pub mod tui;

pub use agent::AgentLoop;
pub use chat::run;
pub use cli::Command;
pub use config::Config;
pub use provider::{LlmProvider, ProviderError, OpenRouterProvider};
pub use tui::Tui;

#[cfg(test)]
mod tests {
    use crate::tui::{InputResult, Tui};
    use crate::agent::AgentLoop;
    use crate::provider::{LlmProvider, ProviderError};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_command_parsing_quit() {
        let tui = Tui::new();
        assert!(matches!(tui.handle_input("/quit"), InputResult::Quit));
    }

    #[test]
    fn test_command_parsing_help() {
        let tui = Tui::new();
        match tui.handle_input("/help") {
            InputResult::Command(cmd) => assert_eq!(cmd, "help"),
            _ => panic!("Expected Command"),
        }
    }

    #[test]
    fn test_chat_input() {
        let tui = Tui::new();
        match tui.handle_input("Hello world") {
            InputResult::Chat(text) => assert_eq!(text, "Hello world"),
            _ => panic!("Expected Chat"),
        }
    }

    #[test]
    fn test_tui_add_message() {
        let mut tui = Tui::new();
        tui.add_message("user", "Hello");
        tui.add_message("ao", "Hi there!");

        assert_eq!(tui.messages.len(), 2);
        assert_eq!(tui.messages[0].role, "user");
        assert_eq!(tui.messages[0].content, "Hello");
        assert_eq!(tui.messages[1].role, "ao");
        assert_eq!(tui.messages[1].content, "Hi there!");
    }

    #[test]
    fn test_input_trimming() {
        let tui = Tui::new();
        assert!(matches!(tui.handle_input("  /quit  "), InputResult::Quit));
    }

    #[test]
    fn test_empty_input_parsed_as_chat() {
        let tui = Tui::new();
        assert!(matches!(tui.handle_input(""), InputResult::Chat(ref s) if s.is_empty()));
    }

    #[test]
    fn test_scroll_offset() {
        let mut tui = Tui::new();
        for i in 0..10 {
            tui.add_message("user", &format!("Message {}", i));
        }

        tui.scroll_up(3);
        tui.scroll_down(5);

        let max_offset = tui.messages.len().saturating_sub(1);
        assert!(tui.scroll_offset() <= max_offset);
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

        fn get_calls(&self) -> Vec<String> {
            self.calls.lock().unwrap().clone()
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

    #[tokio::test]
    async fn test_agent_calls_provider() {
        let mock = MockProvider::new(vec!["Hello from agent!".to_string()]);
        let agent = AgentLoop::new(Box::new(mock.clone()));

        let result = agent.run("Hi").await.unwrap();
        assert_eq!(result, "Hello from agent!");

        let calls = mock.get_calls();
        assert!(!calls.is_empty());
        assert!(calls[0].contains("Hi"));
    }

    #[tokio::test]
    async fn test_agent_prompt_format() {
        let mock = MockProvider::new(vec!["Response".to_string()]);
        let agent = AgentLoop::new(Box::new(mock.clone()));

        let _ = agent.run("Test message").await;

        let calls = mock.get_calls();
        assert!(calls[0].contains("User:"));
        assert!(calls[0].contains("Test message"));
        assert!(calls[0].contains("Ao:"));
    }

    #[tokio::test]
    async fn test_agent_error_handling() {
        let mock = MockProvider::new(vec![]);
        let agent = AgentLoop::new(Box::new(mock));

        let result = agent.run("Test").await;
        assert!(result.is_err());
    }
}
