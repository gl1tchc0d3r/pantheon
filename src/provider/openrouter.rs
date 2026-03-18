use super::{LlmProvider, ProviderError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenRouterProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: Client,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String, model: String, base_url: String) -> Self {
        Self {
            api_key,
            model,
            base_url,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenRouterProvider {
    async fn complete(&self, prompt: &str) -> Result<String, ProviderError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(ProviderError::HttpError)?;

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|_| ProviderError::ParseError)?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or(ProviderError::ParseError)
    }
}
