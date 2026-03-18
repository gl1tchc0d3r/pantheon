mod config;
mod provider;
mod agent;
mod tui;

use config::Config;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use provider::OpenRouterProvider;
use agent::AgentLoop;
use tui::Tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;

    let provider = OpenRouterProvider::new(
        config.provider.api_key,
        config.provider.model,
        config.provider.base_url,
    );

    let agent = AgentLoop::new(Box::new(provider));
    let mut tui = Tui::new();
    let mut terminal = tui::init_terminal()?;
    let mut input_buffer = String::new();
    let mut is_processing = false;

    loop {
        tui.render(&mut terminal, &input_buffer, is_processing)?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => {
                            input_buffer.push(c);
                        }
                        KeyCode::Backspace => {
                            input_buffer.pop();
                        }
                        KeyCode::Enter => {
                            let input = input_buffer.clone();
                            input_buffer.clear();

                            if !input.trim().is_empty() {
                                match tui.handle_input(&input) {
                                    tui::InputResult::Quit => {
                                        tui::restore_terminal();
                                        break;
                                    }
                                    tui::InputResult::Command(ref cmd) if cmd == "help" => {
                                        tui::restore_terminal();
                                        tui::print_help();
                                        println!("\nPress Enter to continue...");
                                        let _ = std::io::stdin().read_line(&mut String::new());
                                    }
                                    tui::InputResult::Chat(text) => {
                                        tui.add_message("user", &text);
                                        is_processing = true;
                                        tui.render(&mut terminal, &input_buffer, is_processing)?;

                                        match agent.run(&text).await {
                                            Ok(response) => {
                                                tui.add_message("ao", &response);
                                            }
                                            Err(e) => {
                                                tui.add_message("system", &format!("Error: {:?}", e));
                                            }
                                        }
                                        is_processing = false;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        KeyCode::Up => {
                            tui.scroll_up(3);
                        }
                        KeyCode::Down => {
                            tui.scroll_down(3);
                        }
                        KeyCode::Esc => {
                            tui::restore_terminal();
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
