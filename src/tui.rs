use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io::{self, Stdout};

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub enum InputResult {
    Quit,
    Command(String),
    Chat(String),
}

pub struct Tui {
    messages: Vec<Message>,
    scroll_offset: usize,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll_offset: 0,
        }
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
    }

    pub fn handle_input(&self, input: &str) -> InputResult {
        match input.trim() {
            "/quit" => InputResult::Quit,
            "/help" => InputResult::Command("help".to_string()),
            _ => InputResult::Chat(input.to_string()),
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        let max_offset = self.messages.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + lines).min(max_offset);
    }

    pub fn render<B: ratatui::backend::Backend>(
        &self,
        terminal: &mut Terminal<B>,
        input_buffer: &str,
        is_processing: bool,
    ) -> io::Result<()> {
        terminal.draw(|f| {
            let size = f.size();

            let input_height = 3u16;
            let status_height = 1u16;
            let header_height = 1u16;

            let main_height = size
                .height
                .saturating_sub(input_height + status_height + header_height + 4);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(header_height),
                    Constraint::Length(main_height),
                    Constraint::Length(input_height),
                    Constraint::Length(status_height),
                ])
                .split(size);

            let title = if is_processing {
                "Pantheon v0.1.0 - Processing..."
            } else {
                "Pantheon v0.1.0"
            };
            let header = Paragraph::new(title).bold();
            f.render_widget(header, chunks[0]);

            let message_items: Vec<ListItem> = self
                .messages
                .iter()
                .skip(self.scroll_offset)
                .map(|msg| {
                    let prefix = if msg.role == "user" {
                        "[You] "
                    } else {
                        "[Ao] "
                    };
                    ListItem::new(format!("{}{}", prefix, msg.content))
                })
                .collect();

            let messages_list = List::new(message_items)
                .block(Block::default().borders(Borders::ALL).title("Messages"));

            f.render_widget(messages_list, chunks[1]);

            let input_prompt = if is_processing { "> ... " } else { "> " };
            let input_text = Paragraph::new(format!("{}{}", input_prompt, input_buffer))
                .block(Block::default().borders(Borders::ALL).title(" Input "));

            f.render_widget(input_text, chunks[2]);

            let status_text = format!(
                "Messages: {} | ↑↓ Scroll | /quit Exit",
                self.messages.len() / 2
            );
            let status_bar = Paragraph::new(status_text);
            f.render_widget(status_bar, chunks[3]);
        })?;
        Ok(())
    }
}

impl Default for Tui {
    fn default() -> Self {
        Self::new()
    }
}

pub fn print_help() {
    println!("Available commands:");
    println!("  /quit  - Exit the application");
    println!("  /help  - Show this help message");
}

pub fn restore_terminal() {
    crossterm::terminal::disable_raw_mode().ok();
    crossterm::execute!(
        io::stdout(),
        crossterm::cursor::Show,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    )
    .ok();
}

pub fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::cursor::Hide,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    )?;
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}
