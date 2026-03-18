mod config;
mod provider;
mod agent;
mod tui;

use config::Config;
use provider::OpenRouterProvider;
use agent::AgentLoop;
use tui::{Tui, print_help};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = Config::load()?;

    let provider = OpenRouterProvider::new(
        config.provider.api_key,
        config.provider.model,
        config.provider.base_url,
    );

    let agent = AgentLoop::new(Box::new(provider));
    let mut tui = Tui::new();

    println!("Pantheon v0.1.0 - Type /help for commands\n");

    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match tui.handle_input(&input) {
            tui::InputResult::Quit => break,
            tui::InputResult::Command(ref cmd) if cmd == "help" => {
                print_help();
            }
            tui::InputResult::Chat(text) => {
                if text.is_empty() {
                    continue;
                }

                tui.add_message("user", &text);

                match agent.run(&text).await {
                    Ok(response) => {
                        tui.add_message("ao", &response);
                        for msg in tui.messages() {
                            println!("{}: {}", msg.role, msg.content);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
