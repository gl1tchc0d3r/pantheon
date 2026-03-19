# Pantheon

A personal AI assistant framework built in Rust with persistent conversation history and context.

---

## Status

**Current Version**: 0.2.0  
**Status**: Active Development  
**Current Phase**: Context and History - Session Management

---

## Quick Start

```bash
# Clone repository
git clone <repository-url>
cd pantheon

# Set API key
export OPENROUTER_API_KEY="your-api-key"

# Build and run
cargo build --release
./target/release/ao
```

---

## Features

- **Persistent Sessions**: Conversations are stored in SQLite and persist across restarts
- **Cross-Session Context**: Previous session summaries are loaded at startup for continuity
- **Session Summarization**: Automatic LLM-generated summaries when sessions end
- **Token Budget Control**: Configurable `max_history_tokens` for controlling context size
- **TUI Interface**: Colorful terminal UI with scrolling support

---

## Project Structure

```
pantheon/
├── src/                    # Source code
│   ├── main.rs            # Entry point
│   ├── lib.rs             # Library exports
│   ├── chat.rs            # Chat loop and session management
│   ├── agent.rs           # LLM prompt building
│   ├── config.rs          # Configuration loading
│   ├── tui.rs             # Terminal UI rendering
│   ├── provider/          # LLM provider implementations
│   └── session/           # Session management module
├── design/                 # Design documents
├── .ao/                    # Runtime configuration
│   ├── config.yaml        # User configuration
│   └── sessions.db        # SQLite database (auto-created)
└── README.md              # This file
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](design/ARCHITECTURE.md) | How the system works |
| [FEATURES.md](design/FEATURES.md) | Feature specifications |
| [IMPLEMENTATION.md](design/IMPLEMENTATION.md) | Implementation phases |

### Feature Documents

Each version has a detailed feature document:
- [0.1.0_FEATURE_MVP.md](design/0.1.0_FEATURE_MVP.md) - MVP: Basic Chat + LLM
- [0.2.0_FEATURE_CONTEXT.md](design/0.2.0_FEATURE_CONTEXT.md) - **Current**: Session Management + Context

Use [FEATURE_TEMPLATE.md](design/FEATURE_TEMPLATE.md) for creating new feature documents.

---

## Implementation Status

| Version | Feature | Feature Document | Status |
|---------|---------|------------------|--------|
| **0.1.0** | MVP: Chat + LLM | [0.1.0_FEATURE_MVP.md](design/0.1.0_FEATURE_MVP.md) | Completed |
| **0.2.0** | History + Context | [0.2.0_FEATURE_CONTEXT.md](design/0.2.0_FEATURE_CONTEXT.md) | **Completed** |
| 0.3.0 | Ao's Identity | [0.3.0_FEATURE_IDENTITY.md](design/0.3.0_FEATURE_IDENTITY.md) | Not Started |
| 0.4.0 | Basic Tools | [0.4.0_FEATURE_TOOLS.md](design/0.4.0_FEATURE_TOOLS.md) | Not Started |
| 0.5.0 | Memory System | [0.5.0_FEATURE_MEMORY.md](design/0.5.0_FEATURE_MEMORY.md) | Not Started |
| 0.6.0 | Library System | [0.6.0_FEATURE_LIBRARY.md](design/0.6.0_FEATURE_LIBRARY.md) | Not Started |
| 0.7.0 | Workers | [0.7.0_FEATURE_WORKERS.md](design/0.7.0_FEATURE_WORKERS.md) | Not Started |
| 0.8.0 | Skills | [0.8.0_FEATURE_SKILLS.md](design/0.8.0_FEATURE_SKILLS.md) | Not Started |
| 0.9.0 | Scheduler | [0.9.0_FEATURE_SCHEDULER.md](design/0.9.0_FEATURE_SCHEDULER.md) | Not Started |
| 1.0.0 | Refinement | [1.0.0_FEATURE_REFINE.md](design/1.0.0_FEATURE_REFINE.md) | Not Started |

---

## CLI Commands

The `ao` CLI provides the following commands:

| Command | Description |
|---------|-------------|
| `ao` | Start the chat interface (default) |
| `ao --version` | Show version |
| `ao --help` | Show help |

### In-Chat Commands

| Command | Description |
|---------|-------------|
| `/quit` | Exit the application (generates session summary) |
| `/help` | Show available commands |
| `/clear` | Clear current conversation history |
| `/status` | Show session info and previous summaries |

---

## Configuration

Configuration is in `.ao/config.yaml`:

```yaml
provider:
  api_key: ""  # Loaded from OPENROUTER_API_KEY env var
  model: "anthropic/claude-3-haiku"
  base_url: "https://openrouter.ai/api/v1"

session:
  max_history_tokens: 8000      # Token budget for previous session summaries
  db_path: ".ao/sessions.db"    # SQLite database path
  auto_resume: true             # Resume last session on startup
  summarize_on_close: true      # Generate summary when session ends
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `max_history_tokens` | usize | 8000 | Token budget for previous session summaries at session start |
| `db_path` | String | `.ao/sessions.db` | SQLite database path |
| `auto_resume` | bool | true | Resume last session on startup |
| `summarize_on_close` | bool | true | Generate summary when session ends |

---

## Architecture Highlights

### Data Persistence

- **Messages**: All messages stored PERMANENTLY in SQLite (never deleted)
- **Summaries**: LLM-generated session summaries stored for cross-session context
- **Token Budget**: Applies only to previous session summaries at session start

### Prompt Structure

```
=== Previous Sessions Summary ===
Session 2024-01-14: Discussed Rust async programming...
=== End Previous Sessions ===

=== Current Session History ===
User: Hello
Ao: Hi there!
=== End Current History ===

User: What did we discuss yesterday?
Ao:
```

---

## License

MIT
