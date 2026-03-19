use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io::{self, Stdout};

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let prefix_len = if text.starts_with("[You] ") {
        6
    } else if text.starts_with("[Ao] ") {
        5
    } else if text.starts_with("[*]  ") {
        4
    } else {
        0
    };
    let available_width = width.saturating_sub(prefix_len + 1);

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= available_width.max(20) {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

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

            let available_width = (chunks[1].width.saturating_sub(4)) as usize;
            let mut message_items: Vec<ListItem> = Vec::new();

            for msg in &self.messages {
                let (prefix, content) = match msg.role.as_str() {
                    "user" => ("[You] ", msg.content.clone()),
                    "ao" => ("[Ao] ", msg.content.clone()),
                    "system" => ("[*]  ", msg.content.clone()),
                    _ => ("[?]  ", msg.content.clone()),
                };
                let text = format!("{}{}", prefix, content);
                let wrapped_lines = wrap_text(&text, available_width);

                for line in wrapped_lines {
                    if msg.role == "system" {
                        message_items.push(ListItem::new(line.dim()));
                    } else {
                        message_items.push(ListItem::new(line));
                    }
                }
            }

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
