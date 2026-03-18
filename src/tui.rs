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
    input_mode: bool,
    messages: Vec<Message>,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            input_mode: true,
            messages: Vec::new(),
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

    pub fn messages(&self) -> &[Message] {
        &self.messages
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
