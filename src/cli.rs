use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Start the chat interface
    Chat,
}

impl Command {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Command::Chat => crate::chat::run().await,
        }
    }
}
