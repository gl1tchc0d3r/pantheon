use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io::{self, Stdout};

fn wrap_text(text: &str, width: usize) -> Vec<Line<'static>> {
    let prefix_len = if text.starts_with("[You] ") {
        6
    } else if text.starts_with("[Ao] ") {
        5
    } else if text.starts_with("[*]  ") {
        4
    } else {
        0
    };
    let available_width = width.saturating_sub(prefix_len + 2).max(20);

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= available_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(Line::from(std::mem::take(&mut current_line)));
        }
    }
    if !current_line.is_empty() {
        lines.push(Line::from(current_line));
    }
    if lines.is_empty() {
        lines.push(Line::from(""));
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
    pub input_buffer: String,
    pub cursor_position: usize,
    list_state: ListState,
    total_lines: usize,
    was_at_bottom: bool,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input_buffer: String::new(),
            cursor_position: 0,
            list_state: ListState::default(),
            total_lines: 0,
            was_at_bottom: true,
        }
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
        if self.was_at_bottom {
            self.scroll_to_bottom();
        }
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

    pub fn insert_char(&mut self, c: char) {
        let pos = self.cursor_position.min(self.input_buffer.len());
        self.input_buffer.insert(pos, c);
        self.cursor_position += 1;
    }

    pub fn delete_char_before_cursor(&mut self) {
        if self.cursor_position > 0 && !self.input_buffer.is_empty() {
            let pos = (self.cursor_position - 1).min(self.input_buffer.len());
            self.input_buffer.remove(pos);
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_position = 0;
        self.input_buffer.clear();
    }

    pub fn scroll_up(&mut self, lines: usize) {
        let offset = self.list_state.offset().saturating_sub(lines);
        self.list_state = ListState::default().with_offset(offset);
        self.was_at_bottom = false;
    }

    pub fn scroll_down(&mut self, lines: usize) {
        let current_offset = self.list_state.offset();
        let max_offset = self.total_lines.saturating_sub(1);
        let new_offset = (current_offset + lines).min(max_offset);
        self.list_state = ListState::default().with_offset(new_offset);
    }

    fn scroll_to_bottom(&mut self) {
        if self.total_lines > 0 {
            let offset = self.total_lines.saturating_sub(1);
            self.list_state = ListState::default().with_offset(offset);
        }
    }

    #[allow(dead_code)]
    pub fn scroll_offset(&self) -> usize {
        self.list_state.offset()
    }

    pub fn render<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
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
            let mut list_items: Vec<ListItem> = Vec::new();

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
                        list_items.push(ListItem::new(line.dim()));
                    } else {
                        list_items.push(ListItem::new(line));
                    }
                }
            }

            self.total_lines = list_items.len();
            self.list_state = ListState::default().with_offset(self.list_state.offset());

            let messages_list = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title("Messages"));

            f.render_stateful_widget(messages_list, chunks[1], &mut self.list_state);

            let input_display = if self.input_buffer.is_empty() {
                String::new()
            } else {
                self.input_buffer.clone()
            };

            let input_text = Paragraph::new(input_display.as_str())
                .block(Block::default().borders(Borders::ALL).title(" Input "));

            f.render_widget(input_text, chunks[2]);

            if !is_processing {
                f.set_cursor(
                    chunks[2].x + 1 + self.cursor_position as u16,
                    chunks[2].y + 1,
                );
            }

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
