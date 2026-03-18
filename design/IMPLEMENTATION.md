# Pantheon - Implementation Plan

**Purpose**: Define the implementation phases for building Pantheon, starting with a working MVP.

---

## Philosophy

Build incrementally. Each phase produces a working system that can be used and tested. Add complexity only when the foundation is solid.

---

## Feature Document Naming Convention

For each version, create a feature document: `design/X.Y.Z_FEATURE_<name>.md`

After completing a phase:
1. Create feature document with implementation details
2. Update `README.md` with new status
3. Tag the commit with the version

---

## Phase 1: The MVP (Minimum Viable Product)

**Version**: 0.1.0  
**Feature Document**: `0.1.0_FEATURE_MVP.md`  
**Status**: Completed

**Goal**: A working chat interface connected to an LLM that you can talk to.

**What this looks like**:
- You type a message in a terminal
- It sends to an LLM (configurable provider)
- You see the response
- Basic command handling (/quit, /help)

**This is the foundation everything else builds on.**

### Phase 1 Scope

| Feature | What's Included |
|---------|-----------------|
| **TUI** | Basic chat layout, input, display responses |
| **Agent Loop** | Send prompts to LLM, receive responses |
| **Provider** | Support one provider initially (OpenRouter or Ollama) |
| **Config** | API key, provider settings in config |
| **Commands** | `/quit`, `/help` |

### Phase 1 Not Included

- No memory between sessions
- No tools
- No workers
- No Library
- No skills

### Phase 1 Deliverable

```
User: Hello
Ao: Hello! I am Ao, your assistant.
```

Simple, but it proves the core loop works.

### Phase 1 Implementation Steps

1. **Setup project structure**
   - Initialize git repository
   - Create Rust project (`cargo init`)
   - Add dependencies (tokio, reqwest, ratatui)
   - Set up logging
   - Create basic project structure:
     ```
     pantheon/
       src/
         main.rs
         lib.rs
       Cargo.toml
       .gitignore
       README.md
       design/
         ARCHITECTURE.md
         FEATURES.md
         IMPLEMENTATION.md
         0.1.0_FEATURE_MVP.md
     ```

2. **Setup design directory**
   - Move ARCHITECTURE.md, FEATURES.md, IMPLEMENTATION.md to `design/`
   - Create `0.1.0_FEATURE_MVP.md` with detailed MVP implementation plan
   - Update README.md with project status and links to documentation

3. **Configuration**

2. **Configuration**
   - Load config from `.ao/config.yaml`
   - API key from environment variable
   - Provider selection

3. **Basic TUI**
   - Render chat interface
   - Input handling
   - Display messages

4. **LLM integration**
   - Provider trait (for swapping providers)
   - OpenRouter client (or Ollama)
   - Send prompt, receive response

5. **Basic commands**
   - `/quit` - exit
   - `/help` - show commands

---

## Phase 2: Context and History

**Version**: 0.2.0  
**Feature Document**: `0.2.0_FEATURE_CONTEXT.md` (planned)  
**Status**: Not Started

**Goal**: The assistant remembers what you said in this session.

**What this looks like**:
- Conversation history is preserved during session
- Previous messages included in prompt
- `/clear` to reset history

### Phase 2 Scope

| Feature | What's Included |
|---------|-----------------|
| **Session** | Track conversation history |
| **Prompt** | Include history in LLM calls |
| **Commands** | `/clear`, `/status` |

### Phase 2 Implementation Steps

1. **Session management**
   - Store messages in memory during session
   - Include in prompt construction

2. **History compression** (basic)
   - If history too long, truncate oldest messages

3. **Commands**
   - `/clear` - reset history
   - `/status` - show session info

---

## Phase 3: Ao's Identity

**Version**: 0.3.0  
**Feature Document**: `0.3.0_FEATURE_IDENTITY.md` (planned)  
**Status**: Not Started

**Goal**: Ao has a defined identity that shapes responses.

**What this looks like**:
- Ao.md defines who Ao is
- Identity injected into every prompt
- You can edit Ao's identity

### Phase 3 Scope

| Feature | What's Included |
|---------|-----------------|
| **Identity** | Load Ao.md from config |
| **Prompt** | Prepend identity to every prompt |
| **Editable** | You can modify Ao.md |

### Phase 3 Implementation Steps

1. **Load Ao.md**
   - Read from `.ao/ao.md`
   - Parse markdown (or just use as-is)

2. **Prompt assembly**
   - Identity block first in prompt
   - Followed by conversation history

3. **Test different identities**
   - Easy to change how Ao behaves

---

## Phase 4: Basic Tools

**Version**: 0.4.0  
**Feature Document**: `0.4.0_FEATURE_TOOLS.md` (planned)  
**Status**: Not Started

**Goal**: Ao can use tools to take action.

**What this looks like**:
- Ao can call tools (defined in prompt)
- Tool execution works
- Results returned to LLM

### Phase 4 Scope

| Feature | What's Included |
|---------|-----------------|
| **Tool Registry** | List of available tools |
| **Tool Executor** | Run tool, return result |
| **Prompt** | Include tool schemas in prompt |
| **Basic Tools** | `get_status`, `terminal` |

### Phase 4 Implementation Steps

1. **Tool system**
   - Tool trait/struct
   - Registry of available tools

2. **Tool executor**
   - Parse tool calls from LLM response
   - Execute tool
   - Return result to LLM

3. **First tools**
   - `get_status` - system info
   - `terminal` - run shell commands

4. **Risk levels**
   - Read tools work automatically
   - Execute/critical prompt for confirmation

---

## Phase 5: Memory System

**Version**: 0.5.0  
**Feature Document**: `0.5.0_FEATURE_MEMORY.md` (planned)  
**Status**: Not Started

**Goal**: Ao remembers things across sessions.

**What this looks like**:
- SQLite storage for memories
- `/remember` command to store things
- Memories included in prompts

### Phase 5 Scope

| Feature | What's Included |
|---------|-----------------|
| **Storage** | SQLite for memories |
| **Commands** | `/remember`, `/forget` |
| **Recall** | Inject memories into prompts |
| **Short-term** | Session summary at start |

### Phase 5 Implementation Steps

1. **SQLite setup**
   - Create database
   - Memory table

2. **Memory commands**
   - `/remember <text>` - store
   - `/forget` - clear session memory

3. **Recall in prompts**
   - Load recent memories
   - Inject into prompt

4. **Autonomous memory** (basic)
   - Optionally remember important things

---

## Phase 6: Library System

**Version**: 0.6.0  
**Feature Document**: `0.6.0_FEATURE_LIBRARY.md` (planned)  
**Status**: Not Started

**Goal**: Ao knows your files and documents.

**What this looks like**:
- Index files in `library/` directory
- Search Library when you ask
- Results injected into prompt

### Phase 6 Scope

| Feature | What's Included |
|---------|-----------------|
| **Indexer** | Read and index files |
| **Search** | Full-text search |
| **Tools** | `library_search`, `library_index` |
| **Citation** | Track which file info came from |

### Phase 6 Implementation Steps

1. **Library structure**
   - Create `library/` directories
   - File discovery

2. **Indexing**
   - Read file contents
   - Store in SQLite with FTS5

3. **Search tool**
   - Query library
   - Return relevant passages

4. **Prompt injection**
   - Include relevant Library content

---

## Phase 7: Workers

**Version**: 0.7.0  
**Feature Document**: `0.7.0_FEATURE_WORKERS.md` (planned)  
**Status**: Not Started

**Goal**: Parallel execution for complex tasks.

**What this looks like**:
- Spawn workers for parallel tasks
- Workers execute independently
- Results aggregated

### Phase 7 Scope

| Feature | What's Included |
|---------|-----------------|
| **Worker Pool** | Configurable number |
| **Spawn** | Create worker with context |
| **Parallel** | Run multiple workers |
| **Aggregate** | Combine results |

### Phase 7 Implementation Steps

1. **Worker system**
   - Worker pool struct
   - Spawn function

2. **Worker prompts**
   - Inject context
   - Select tools/skills

3. **Parallel execution**
   - Async worker tasks
   - Await all results

---

## Phase 8: Skills

**Version**: 0.8.0  
**Feature Document**: `0.8.0_FEATURE_SKILLS.md` (planned)  
**Status**: Not Started

**Goal**: Reusable workflows that improve over time.

**What this looks like**:
- Skills defined in markdown files
- Triggered by keywords
- Self-improving based on usage

### Phase 8 Scope

| Feature | What's Included |
|---------|-----------------|
| **Skill Format** | Markdown + YAML frontmatter |
| **Loader** | Load skills from `.ao/skills/` |
| **Trigger** | Match user input to skill |
| **Execute** | Run skill workflow |

### Phase 8 Implementation Steps

1. **Skill format**
   - Define YAML frontmatter
   - Workflow steps

2. **Skill loader**
   - Scan `.ao/skills/`
   - Parse definitions

3. **Trigger matching**
   - Detect when skill should run
   - Pass to execution

4. **Execution**
   - Run steps in order
   - Handle conditionals

---

## Phase 9: Scheduler

**Version**: 0.9.0  
**Feature Document**: `0.9.0_FEATURE_SCHEDULER.md` (planned)  
**Status**: Not Started

**Goal**: Automated tasks on schedules.

**What this looks like**:
- Define jobs in config
- Run on schedule (human-readable)
- Persist across restarts

### Phase 9 Scope

| Feature | What's Included |
|---------|-----------------|
| **Parser** | Human-readable to cron |
| **Jobs** | Tool, skill, worker execution |
| **Persistence** | SQLite for job state |
| **Background** | Run scheduler async |

### Phase 9 Implementation Steps

1. **Schedule parser**
   - "every hour" → cron
   - "on mondays" → cron

2. **Job execution**
   - Trigger tool/skill/worker
   - Handle results

3. **Persistence**
   - Store jobs in SQLite
   - Track last run, next run

---

## Phase 10: Refinement

**Version**: 1.0.0  
**Feature Document**: `1.0.0_FEATURE_REFINE.md` (planned)  
**Status**: Not Started

**Goal**: Polish and optimize.

**What this looks like**:
- Error handling improvements
- Performance tuning
- User experience refinements
- Documentation

### Phase 10 Includes

- Better error messages
- Retry logic improvements
- Token optimization
- Logging and debugging tools

---

## Summary: Phase Order

| Phase | Focus | Key Deliverable |
|-------|-------|-----------------|
| 1 | MVP | Chat with LLM works |
| 2 | History | Session persistence |
| 3 | Identity | Ao has defined personality |
| 4 | Tools | Ao can take action |
| 5 | Memory | Remembers across sessions |
| 6 | Library | Knows your files |
| 7 | Workers | Parallel execution |
| 8 | Skills | Reusable workflows |
| 9 | Scheduler | Automated tasks |
| 10 | Polish | Refinement |

---

## Dependency Graph

```
Phase 1 ──► Phase 2 ──► Phase 3 ──► Phase 4 ──► Phase 5 ──► Phase 6
  │           │           │           │           │           │
  │           │           │           │           │           │
  └──────────►└──────────►└──────────►└──────────►└──────────► Phase 7 ──► Phase 8 ──► Phase 9 ──► Phase 10
```

Each phase builds on the previous. You could skip to later phases if you have different priorities, but this order ensures each piece is tested before adding complexity.

---

## Quick Start: Run Phase 1 First

If you're uncertain where to start:

1. Create the project
2. Get a single LLM call working
3. Get a basic TUI showing the response
4. That's Phase 1 complete

Everything else builds on that foundation.
