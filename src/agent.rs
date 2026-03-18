use crate::provider::{LlmProvider, ProviderError};

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

    pub async fn run(&self, input: &str) -> Result<String, AgentError> {
        let prompt = format!("User: {}\nAo:", input);
        let response = self.provider.complete(&prompt).await?;
        Ok(response)
    }
}
