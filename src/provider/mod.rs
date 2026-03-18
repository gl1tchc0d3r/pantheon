pub mod openrouter;

pub use openrouter::OpenRouterProvider;

use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String, ProviderError>;
}

#[derive(Debug)]
pub enum ProviderError {
    HttpError(reqwest::Error),
    ApiError(String),
    ParseError,
}
