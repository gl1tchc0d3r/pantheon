use crate::config::Config;
use crate::provider::OpenRouterProvider;
use crate::agent::AgentLoop;
use crate::tui::Tui;
use crate::tui;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;

    let provider = OpenRouterProvider::new(
        config.provider.api_key,
        config.provider.model,
        config.provider.base_url,
    );

    let agent = AgentLoop::new(Box::new(provider));
    let mut tui_instance = Tui::new();
    let mut terminal = tui::init_terminal()?;
    let mut input_buffer = String::new();
    let mut is_processing = false;

    loop {
        tui_instance.render(&mut terminal, &input_buffer, is_processing)?;

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
                                match tui_instance.handle_input(&input) {
                                    crate::tui::InputResult::Quit => {
                                        tui::restore_terminal();
                                        break;
                                    }
                                    crate::tui::InputResult::Command(ref cmd) if cmd == "help" => {
                                        tui_instance.add_message("system", "Available commands:");
                                        tui_instance.add_message("system", "/quit  - Exit the application");
                                        tui_instance.add_message("system", "/help  - Show this help message");
                                    }
                                    crate::tui::InputResult::Chat(text) => {
                                        tui_instance.add_message("user", &text);
                                        is_processing = true;
                                        tui_instance.render(&mut terminal, &input_buffer, is_processing)?;

                                        match agent.run(&text).await {
                                            Ok(response) => {
                                                tui_instance.add_message("ao", &response);
                                            }
                                            Err(e) => {
                                                tui_instance.add_message("system", &format!("Error: {:?}", e));
                                            }
                                        }
                                        is_processing = false;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        KeyCode::Up => {
                            tui_instance.scroll_up(3);
                        }
                        KeyCode::Down => {
                            tui_instance.scroll_down(3);
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
