# Pantheon

A personal AI assistant framework built in Rust.

---

## Status

**Current Version**: 0.1.0  
**Status**: Planning / Early Development

---

## Quick Start

```bash
# Clone repository
git clone <repository-url>
cd pantheon

# Set API key
export OPENROUTER_API_KEY="your-api-key"

# Create config
mkdir -p .ao
cat > .ao/config.yaml << 'EOF'
provider:
  model: "anthropic/claude-3-haiku"
  base_url: "https://openrouter.ai/api/v1"
EOF

# Build and run
cargo build --release
./target/release/pantheon chat
```

---

## Project Structure

```
pantheon/
├── src/                    # Source code
├── design/                 # Design documents
│   ├── ARCHITECTURE.md     # System architecture
│   ├── FEATURES.md         # Feature specifications
│   ├── IMPLEMENTATION.md   # Implementation phases
│   └── 0.1.0_FEATURE_MVP.md  # MVP implementation plan
├── .ao/                    # Runtime configuration
│   └── config.yaml        # User configuration
└── README.md               # This file
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](design/ARCHITECTURE.md) | How the system works |
| [FEATURES.md](design/FEATURES.md) | Feature specifications |
| [IMPLEMENTATION.md](design/IMPLEMENTATION.md) | Implementation phases |
| [FEATURE_TEMPLATE.md](design/FEATURE_TEMPLATE.md) | Template for feature documents |

### Feature Documents

Each version has a detailed feature document:
- [0.1.0_FEATURE_MVP.md](design/0.1.0_FEATURE_MVP.md) - Current phase

Use [FEATURE_TEMPLATE.md](design/FEATURE_TEMPLATE.md) for creating new feature documents.

### Legacy (Previous Project)

The `design/` directory also contains feature documents from a previous project for reference:
- `FEATURE_0*.md` - Old feature specifications (not current)

---

## Key Design Decisions

- **Ao as Active Agent**: Ao is the primary agent users interact with, not just a coordinator
- **Workers on Demand**: Workers are ephemeral, spawned via prompt injection for parallel tasks
- **User-Controlled Memory**: Explicit `/remember` commands, not automatic injection
- **Library System**: Personal knowledge base that Ao can search
- **Selective Context**: Short-term memory at session start, long-term on demand

---

## Implementation Status

| Version | Feature | Feature Document | Status |
|---------|---------|------------------|--------|
| **0.1.0** | MVP: Chat + LLM | [0.1.0_FEATURE_MVP.md](design/0.1.0_FEATURE_MVP.md) | **Completed** |
| 0.2.0 | History + Context | [0.2.0_FEATURE_CONTEXT.md](design/0.2.0_FEATURE_CONTEXT.md) | Not Started |
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
| `ao chat` | Start the chat interface (default) |
| `ao --version` | Show version |
| `ao --help` | Show help |

### In-Chat Commands

| Command | Description |
|---------|-------------|
| `/quit` | Exit the application |
| `/help` | Show available commands |

---

## Configuration

Configuration is in `.ao/config.yaml`:

```yaml
provider:
  model: "anthropic/claude-3-haiku"
  base_url: "https://openrouter.ai/api/v1"
```

API keys are loaded from environment variables:
- `OPENROUTER_API_KEY` - Required for OpenRouter

---

## License

MIT
