use crate::config::Config;
use crate::provider::{LlmProvider, OpenRouterProvider};
use crate::agent::AgentLoop;
use crate::tui::Tui;
use crate::tui;
use crate::session::{SessionManager, Message};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;

    let provider = OpenRouterProvider::new(
        config.provider.api_key,
        config.provider.model,
        config.provider.base_url,
    );

    let provider_ref: Box<dyn LlmProvider> = Box::new(provider.clone());
    let agent = AgentLoop::new(provider_ref);
    let session_config = config.session.clone();
    let auto_resume = session_config.auto_resume;
    let mut session_manager = SessionManager::new(session_config).await?;

    if auto_resume {
        if let Some(_) = session_manager.resume_last_session().await? {
            tracing::info!("Resumed previous session");
        } else {
            session_manager.start_session().await?;
            tracing::info!("Started new session");
        }
    } else {
        session_manager.start_session().await?;
        tracing::info!("Started new session");
    }

    let previous_summaries = session_manager.get_previous_summaries().await?;

    let mut tui_instance = Tui::new();
    let mut terminal = tui::init_terminal()?;
    let mut is_processing = false;

    if let Some(session) = session_manager.current_session() {
        tui_instance.add_message("system", &format!("Session: {}", &session.id[..8]));
        if !previous_summaries.is_empty() {
            tui_instance.add_message("system", &format!("Loaded {} previous session summaries", previous_summaries.len()));
        }
    }

    loop {
        tui_instance.render(&mut terminal, is_processing)?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => {
                            tui_instance.insert_char(c);
                        }
                        KeyCode::Backspace => {
                            tui_instance.delete_char_before_cursor();
                        }
                        KeyCode::Left => {
                            tui_instance.move_cursor_left();
                        }
                        KeyCode::Right => {
                            tui_instance.move_cursor_right();
                        }
                        KeyCode::Enter => {
                            let input = tui_instance.input_buffer.clone();
                            tui_instance.reset_cursor();

                            if !input.trim().is_empty() {
                                match tui_instance.handle_input(&input) {
                                    crate::tui::InputResult::Quit => {
                                        tui_instance.add_message("system", "Summarizing conversation for future reference...");
                                        tui_instance.render(&mut terminal, false)?;
                                        if let Err(e) = session_manager.end_session(&provider).await {
                                            tracing::warn!("Failed to summarize session: {}", e);
                                        }
                                        tui::restore_terminal();
                                        break;
                                    }
                                    crate::tui::InputResult::Command(ref cmd) if cmd == "help" => {
                                        tui_instance.add_message("system", "Available commands:");
                                        tui_instance.add_message("system", "/quit   - Exit the application");
                                        tui_instance.add_message("system", "/help   - Show this help message");
                                        tui_instance.add_message("system", "/clear  - Clear conversation history");
                                        tui_instance.add_message("system", "/status - Show current session info");
                                    }
                                    crate::tui::InputResult::Command(ref cmd) if cmd == "clear" => {
                                        session_manager.clear_history();
                                        tui_instance.add_message("system", "History cleared.");
                                    }
                                    crate::tui::InputResult::Command(ref cmd) if cmd == "status" => {
                                        if let Some(session) = session_manager.current_session() {
                                            tui_instance.add_message("system", "═══ Session Status ═══");
                                            tui_instance.add_message("system", &format!("ID: {}", &session.id[..8]));
                                            tui_instance.add_message("system", &format!("Messages: {}", session.messages.len()));
                                            tui_instance.add_message("system", &format!("Tokens: ~{}", session.estimate_tokens()));
                                            tui_instance.add_message("system", &format!("Created: {}", session.created_at.format("%Y-%m-%d %H:%M:%S")));
                                            if let Some(ref summary) = session.summary {
                                                tui_instance.add_message("system", "─── Current Summary ───");
                                                tui_instance.add_message("system", summary);
                                            } else {
                                                tui_instance.add_message("system", "─── Summary: Not yet generated (will be created on quit) ───");
                                            }
                                            tui_instance.add_message("system", "════════════════════");
                                        } else {
                                            tui_instance.add_message("system", "No active session.");
                                        }
                                    }
                                    crate::tui::InputResult::Chat(text) => {
                                        tui_instance.add_message("user", &text);

                                        if let Err(e) = session_manager.add_message("user", &text).await {
                                            tui_instance.add_message("system", &format!("Error storing message: {}", e));
                                            continue;
                                        }

                                        is_processing = true;
                                        tui_instance.render(&mut terminal, is_processing)?;

                                        let history: Vec<&Message> = session_manager
                                            .current_session()
                                            .map(|s| s.messages.iter().collect())
                                            .unwrap_or_default();

                                        match agent.run(&text, &history, &previous_summaries).await {
                                            Ok(response) => {
                                                tui_instance.add_message("ao", &response);
                                                if let Err(e) = session_manager.add_message("ao", &response).await {
                                                    tracing::warn!("Failed to store ao message: {}", e);
                                                }
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
                            tui_instance.add_message("system", "Summarizing conversation for future reference...");
                            tui_instance.render(&mut terminal, false)?;
                            if let Err(e) = session_manager.end_session(&provider).await {
                                tracing::warn!("Failed to summarize session: {}", e);
                            }
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
