use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation},
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
    scroll_offset: usize,
    content_height: usize,
    was_at_bottom: bool,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input_buffer: String::new(),
            cursor_position: 0,
            scroll_offset: 0,
            content_height: 0,
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
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
        self.was_at_bottom = false;
    }

    pub fn scroll_down(&mut self, lines: usize) {
        let max_offset = self.content_height.saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + lines).min(max_offset);
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.content_height.saturating_sub(1);
        self.was_at_bottom = true;
    }

    #[allow(dead_code)]
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn render<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        is_processing: bool,
    ) -> io::Result<()> {
        terminal.draw(|f| {
            let size = f.size();

            let header_height = 1u16;
            let input_height = 3u16;
            let status_height = 1u16;
            let fixed_height = header_height + input_height + status_height + 4;
            let messages_height = size.height.saturating_sub(fixed_height);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(header_height),
                    Constraint::Length(messages_height),
                    Constraint::Length(input_height),
                    Constraint::Length(status_height),
                ])
                .split(size);

            let title = if is_processing {
                " Pantheon v0.2.0 - Processing... "
            } else {
                " Pantheon v0.2.0 "
            };
            let header = Paragraph::new(title)
                .style(Style::default().fg(Color::White).bg(Color::DarkGray))
                .alignment(Alignment::Center);
            f.render_widget(header, chunks[0]);

            let available_width = (chunks[1].width.saturating_sub(2)) as usize;
            let mut all_lines: Vec<Line> = Vec::new();

            for msg in &self.messages {
                let (prefix, content, color) = match msg.role.as_str() {
                    "user" => ("[You] ", msg.content.clone(), Color::Cyan),
                    "ao" => ("[Ao] ", msg.content.clone(), Color::Green),
                    "system" => ("[*]  ", msg.content.clone(), Color::Yellow),
                    _ => ("[?]  ", msg.content.clone(), Color::Red),
                };
                let text = format!("{}{}", prefix, content);
                let wrapped_lines = wrap_text(&text, available_width);

                for line in wrapped_lines {
                    all_lines.push(line.style(Style::default().fg(color)));
                }
            }

            self.content_height = all_lines.len().max(1);

            if self.scroll_offset > self.content_height.saturating_sub(1) {
                self.scroll_offset = self.content_height.saturating_sub(1);
            }

            let messages_paragraph = Paragraph::new(all_lines)
                .scroll((self.scroll_offset as u16, 0))
                .style(Style::default().bg(Color::Rgb(30, 30, 35)));

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Messages ")
                .title_style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(Color::Rgb(30, 30, 35)));

            f.render_widget(messages_paragraph.block(block), chunks[1]);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(Color::DarkGray));
            let mut scrollbar_state = ratatui::widgets::ScrollbarState::new(self.content_height)
                .position(self.scroll_offset);
            f.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state);

            let input_display = if self.input_buffer.is_empty() {
                String::new()
            } else {
                self.input_buffer.clone()
            };

            let input_text = Paragraph::new(input_display.as_str())
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue))
                        .title(" Input ")
                        .title_style(
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        )
                        .style(Style::default().bg(Color::Rgb(20, 20, 25))),
                );

            f.render_widget(input_text, chunks[2]);

            if !is_processing {
                f.set_cursor(
                    chunks[2].x + 1 + self.cursor_position as u16,
                    chunks[2].y + 1,
                );
            }

            let status_text = format!(
                " Messages: {} | ↑↓ Scroll | /help | /quit ",
                self.messages.len() / 2,
            );
            let status_bar = Paragraph::new(status_text)
                .style(Style::default().fg(Color::White).bg(Color::DarkGray))
                .alignment(Alignment::Center);
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
