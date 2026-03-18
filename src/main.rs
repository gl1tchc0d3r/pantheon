use clap::Parser;

#[derive(Parser)]
#[command(
    name = "ao",
    about = "Pantheon - Personal AI Assistant",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<pantheon::Command>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => cmd.execute().await?,
        None => pantheon::Command::Chat.execute().await?,
    }

    Ok(())
}
