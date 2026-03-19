use crossterm::cursor;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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
    pub messages: Vec<Message>,
    list_state: ListState,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            list_state: ListState::default(),
        }
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
        self.scroll_to_bottom();
    }

    pub fn handle_input(&self, input: &str) -> InputResult {
        match input.trim() {
            "/quit" => InputResult::Quit,
            "/help" => InputResult::Command("help".to_string()),
            "/clear" => InputResult::Command("clear".to_string()),
            "/status" => InputResult::Command("status".to_string()),
            _ => InputResult::Chat(input.to_string()),
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        let i = self.list_state.offset().saturating_sub(lines);
        self.list_state = ListState::default().with_offset(i);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        let max_offset = self.messages.len().saturating_sub(1).max(0);
        let new_offset = (self.list_state.offset() + lines).min(max_offset);
        self.list_state = ListState::default().with_offset(new_offset);
    }

    pub fn scroll_to_bottom(&mut self) {
        if !self.messages.is_empty() {
            self.list_state.select(Some(self.messages.len() - 1));
        }
    }

    #[allow(dead_code)]
    pub fn scroll_offset(&self) -> usize {
        self.list_state.offset()
    }

    pub fn render<B: ratatui::backend::Backend>(
        &mut self,
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
                "Pantheon v0.2.0 - Processing..."
            } else {
                "Pantheon v0.2.0"
            };
            let header = Paragraph::new(title).bold();
            f.render_widget(header, chunks[0]);

            let message_items: Vec<ListItem> = self
                .messages
                .iter()
                .map(|msg| {
                    let (prefix, content) = match msg.role.as_str() {
                        "user" => ("[You] ", msg.content.clone()),
                        "ao" => ("[Ao] ", msg.content.clone()),
                        "system" => ("[*]  ", msg.content.clone()),
                        _ => ("[?]  ", msg.content.clone()),
                    };
                    let text = format!("{}{}", prefix, content);
                    if msg.role == "system" {
                        ListItem::new(text.dim())
                    } else {
                        ListItem::new(text)
                    }
                })
                .collect();

            let messages_list = List::new(message_items)
                .block(Block::default().borders(Borders::ALL).title("Messages"))
                .direction(ratatui::widgets::ListDirection::TopToBottom);

            f.render_stateful_widget(messages_list, chunks[1], &mut self.list_state);

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

            if !is_processing {
                let prompt_len = 2;
                let cursor_x = (prompt_len + input_buffer.len()).min(size.width as usize - 2);
                let _ = crossterm::execute!(
                    io::stdout(),
                    cursor::Show,
                    cursor::MoveTo(chunks[2].x + cursor_x as u16 + 1, chunks[2].y + 1)
                );
            }
        })?;
        Ok(())
    }
}

impl Default for Tui {
    fn default() -> Self {
        Self::new()
    }
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
