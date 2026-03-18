# Pantheon - Feature Concepts

**Purpose**: Extract core concepts from DESIGN.md to define features for implementation.

---

## Core Architecture

```
User (TUI) → Ao (active) → Workers (spawned via prompt injection) → Results → Ao → User
```

**Key Principle**: Ao is the only permanent entity. Workers are ephemeral, spawned with prompt injection.

---

## F0: Library System (User's Knowledge Base)

**Purpose**: The Library is the user's personal knowledge base that Pantheon indexes, manages, and uses to provide informed responses. This is what makes Pantheon truly the user's personal assistant.

### What the Library Contains

The Library is where the user stores their personal knowledge:

| Directory | Purpose |
|-----------|---------|
| `library/notes/` | Markdown notes |
| `library/documents/` | Books, papers, PDFs |
| `library/code/` | Code snippets, repositories |
| `library/journal/` | Daily journals, reflections |
| `library/reference/` | API docs, manuals |
| `library/media/` | Images, audio notes |

### Library Integration with Prompt

The Library is queried and injected into prompts for every relevant interaction:

```
User asks about X
       ↓
Library Search: Find relevant documents
       ↓
Extract relevant passages
       ↓
Inject into prompt context
       ↓
LLM responds with knowledge of user's files
```

### Components

| Component | Description |
|-----------|-------------|
| **Indexer** | Watches library, indexes new/changed files |
| **Search Engine** | FTS5 full-text search |
| **Metadata Store** | SQLite table with file info, tags, summaries |
| **Citation System** | Track which file provided which information |
| **Watcher** | File system watcher for automatic updates |

### Storage Schema

```sql
-- Library files metadata
CREATE TABLE library_files (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    title TEXT,
    file_type TEXT,
    tags TEXT,           -- JSON array
    summary TEXT,
    indexed_at TIMESTAMP,
    file_hash TEXT,      -- For change detection
);

-- Full-text search
CREATE VIRTUAL TABLE library_fts USING fts5(
    title,
    content,
    content=library_files,
    content_rowid=rowid
);

-- File content (for search)
CREATE TABLE library_content (
    file_id TEXT PRIMARY KEY,
    content TEXT,
    FOREIGN KEY (file_id) REFERENCES library_files(id)
);
```

### Features to Implement

- [ ] Library directory structure and discovery
- [ ] File indexer (markdown, text, code)
- [ ] FTS5 search integration
- [ ] Automatic re-indexing on file changes (file watcher)
- [ ] Metadata extraction (title, tags, summary)
- [ ] Library search tool for workers
- [ ] Prompt injection of relevant library content
- [ ] Citation/ source tracking
- [ ] Support for different file types

### Library Tools

```yaml
# Library-related tools
tools:
  - name: library_search
    description: Search user's knowledge base
    parameters:
      query: string
      limit: number
  - name: library_index
    description: Re-index the library
  - name: library_add
    description: Add file to library tracking
  - name: library_stats
    description: Show library statistics
```

### Key Behavior: Selective Memory Injection

To prevent prompt overload and give users control, memory injection works in layers:

#### Layer 1: Short-term (Automatic)
At session start, inject recent conversation subjects and key context. This is small (~200-500 tokens).

```
## Recent Context
- Working on: Rust async programming
- Current file: src/main.rs
- Last task: Researching error handling patterns
```

#### Layer 2: Long-term (Manual Command)
Users explicitly add relevant long-term memory via command:

```
/remember "API design patterns from my notes"
```

This injects specific Library passages or memories into context for the current session.

#### Layer 3: On-demand (Tool)
Use `recall` tool during conversation to pull specific memories:

```
recall("error handling patterns Rust")
```

#### Automatic Library Search (Optional)
Can be enabled/disabled per-query:

```
User: "what was that API design pattern?"  → Auto-search Library
User: @library "design patterns"           → Explicit Library search
```

### Commands

```bash
/remember <query>    # Add relevant memory to context
/forget             # Clear short-term context
/context            # Show current context
```

This approach:
- Prevents prompt overload from unlimited Library injection
- Gives users explicit control over what's in context
- Enables relevant recall without automatic bloat
- Short-term context is visible and manageable

### Token Size Considerations

User Library content can grow large. This creates challenges for prompt assembly:

| Challenge | Mitigation |
|-----------|------------|
| **Too many results** | Limit to top K results (configurable, default 5-10) |
| **Long passages** | Truncate each passage to first N characters |
| **Total context overflow** | Reserve X tokens for Library, distribute across results |
| **Relevance vs completeness** | Use relevance ranking, not completeness |

**Implementation strategies**:

```yaml
# Configurable limits
library:
  max_results: 5        # Max documents to include
  max_chars_per_result: 2000  # Truncate each
  reserved_tokens: 4000  # Tokens reserved for Library
```

**Passage extraction**: Rather than injecting entire files, extract the most relevant sections using:
- FTS5 snippet function for excerpts
- LLM to extract relevant paragraphs
- Keyword proximity scoring

This ensures the Library enriches responses without overwhelming the prompt.

---

## F1: Prompt System

**Purpose**: Construct prompts for Ao and workers with proper context assembly.

### Components

| Component | Description | Cached? |
|-----------|-------------|---------|
| **Identity Block** | Ao's core identity and purpose | Yes |
| **System Block** | Dynamic guidance per task | No |
| **Library Context** | Relevant passages from user's knowledge base | Per-turn |
| **Memory Snapshot** | Frozen at session start | At start |
| **User Profile** | Frozen at session start | At start |
| **Skills Index** | Available capabilities | Yes |
| **Tool Schemas** | Tool definitions | Yes |
| **Context Files** | .cursorrules, etc. | Yes |
| **History** | Conversation (with compression) | No |

### Dependencies

- **F0: Library System** - Query Library and inject relevant content into prompt
- **F5: Memory System** - Get memory context for recall

### Key Behaviors

- **Prompt Caching**: Static blocks first to preserve provider-side cache
- **Selective Injection**: Use layers (short-term auto, long-term via command)
- **Frozen Snapshots**: Memory and profile taken at session start, not mutated
- **Context Compression**: Summarize old history, preserve recent
- **Minimal History Mutation**: Append only, never rewrite
- **User Control**: `/remember` command injects specific memories

### Memory Injection Layers

| Layer | Type | Trigger | Size |
|-------|------|---------|------|
| **Short-term** | Auto | Session start | ~200-500 tokens |
| **Long-term** | Manual | `/remember` command | ~500-2000 tokens |
| **On-demand** | Tool | `recall()` during chat | As needed |

This prevents prompt bloat while allowing users to pull relevant context when needed.

### Features to Implement

- [ ] Prompt assembly pipeline (layered construction)
- [ ] Identity block loader from `.ao/ao.md`
- [ ] Context file loader (.cursorrules, etc.)
- [ ] Library query integration (search Library, extract passages)
- [ ] Prompt injection: "Relevant from your Library: {citations}\n{passages}"
- [ ] Memory snapshot at session start
- [ ] History compression (summarize old, preserve recent)
- [ ] Token estimation and limit handling
- [ ] Ordering: Library context after Identity/Skills/Tools (before History)

---

## F2: Agent Loop

**Purpose**: The core execution engine driving Ao and worker behavior.

### Responsibilities

| Responsibility | Description |
|----------------|-------------|
| **Provider Selection** | Route to configured LLM (OpenRouter, Anthropic, Ollama, etc.) |
| **Prompt Construction** | Assemble prompts from identity, Library, context, memory, tools |
| **Tool Execution** | Parse tool calls, dispatch to runtime, handle results |
| **Retries & Fallback** | Handle failures with exponential backoff, provider fallback |
| **Callbacks** | Trigger events on completion, error, or tool use |
| **Compression** | Manage context length via summarization/pruning |
| **Persistence** | Save state to SQLite after each turn |

### Dependencies

- **F0: Library System** - Agent Loop uses Prompt System which queries Library
- **F1: Prompt System** - Provides assembled prompts including Library content

### Features to Implement

- [ ] Provider abstraction (trait for LLM backends)
- [ ] Provider chain with fallback (try A, then B, then C)
- [ ] Retry logic with exponential backoff
- [ ] Circuit breaker for failing providers
- [ ] Callback system (on_tool_call, on_error, on_completion)
- [ ] Turn persistence to SQLite

---

## F3: Tool System

**Purpose**: Atomic executable functions that workers can use.

### Tool Types

| Type | Implementation | Example |
|------|---------------|---------|
| **Core** | Hardcoded in binary | `get_status`, `terminal`, `file_read` |
| **External** | Executable scripts | `.ao/tools/my_tool` |

### Risk Levels

| Level | Description | Requires Confirmation |
|-------|-------------|---------------------|
| `read` | Read-only operations | No |
| `write` | Modify data, create files | No |
| `execute` | Run processes, external calls | Yes |
| `critical` | System changes, destructive | Yes |

### Tool Protocol

**Input**: JSON via stdin  
**Output**: JSON via stdout

```bash
#!/bin/bash
# .ao/tools/my_tool
read -r input
echo "$input" | jq '.param1 * 2'
```

### Features to Implement

- [ ] Tool registry (internal + external)
- [ ] JSON input/output protocol for external tools
- [ ] Risk level enforcement with confirmation prompts
- [ ] Tool schema validation
- [ ] Execution timeout handling
- [ ] Tool discovery (scan `.ao/tools/` on startup)

---

## F4: Worker System

**Purpose**: Spawn ephemeral workers for parallel execution.

### Worker Flow

```
1. Ao receives task
2. Ao selects relevant tools and skills
3. Ao constructs worker prompt via injection
4. Worker executes with its own agent loop
5. Worker returns result to Ao
6. Worker terminates
```

### Worker Configuration

```yaml
workers:
  default_count: 2    # Concurrent workers
  max_count: 5       # Max parallel
  task_timeout: 300  # Seconds
```

### Features to Implement

- [ ] Worker spawner with prompt injection
- [ ] Tool/skill injection at spawn time
- [ ] Worker pool with configurable limits
- [ ] Parallel execution (multiple workers simultaneously)
- [ ] Result aggregation from workers
- [ ] Worker timeout handling

---

## F5: Memory System

**Purpose**: Autonomous learning and persistent storage.

### Memory Types

| Type | Description | Persistence |
|------|-------------|-------------|
| **Ephemeral** | Current context, active state | Session only |
| **Persistent** | Experiences, learnings, skills | SQLite |

### Relationship with Library

The Memory System and Library System serve different purposes but work together:

- **Memory System**: Stores *experiences* - what happened, what you preferred, what was learned
- **Library System**: Stores *knowledge* - your notes, documents, files

Both are queried when assembling prompts. For user queries, the system searches both:
1. **Library first**: Your files, notes, documents
2. **Memory second**: Past interactions, learned preferences

This gives the assistant complete context: it knows your files AND remembers your conversations.

### Autonomous Learning Loop

```
1. COLLECT: Gather recent memory items
2. EVALUATE: LLM scores importance (1-10)
3. CONSOLIDATE: Merge related low-importance items
4. SUMMARIZE: Compress old items via LLM
5. INDEX: Update FTS5 search index
6. PRUNE: Remove redundant/outdated items (moderate)
```

### Cross-Session Recall

Memory recall is triggered explicitly, not automatically:

1. User types `/remember <query>` or uses `recall()` tool
2. Query FTS5 with user's query
3. Retrieve top K matches
4. Use LLM to summarize relevant context
5. Inject into current session context

This prevents automatic bloat while enabling relevant recall.

### Features to Implement

- [ ] SQLite storage for memories
- [ ] FTS5 index for search
- [ ] Importance scoring (LLM-based)
- [ ] Periodic consolidation trigger
- [ ] Memory summarization (LLM-based)
- [ ] Cross-session recall via FTS5
- [ ] Memory pruning (moderate)

---

## F6: Skills System

**Purpose**: Self-improving complex workflows.

### Skill Format

```yaml
---
name: research
description: Conduct comprehensive research
autonomy_level: high
version: 1.0.0
tools:
  - web_search
  - remember
  - recall
triggers:
  - "research <topic>"
---
# Research Skill

## Workflow
1. Search for relevant information
2. Store findings in memory
3. Recall related context
4. Synthesize and present

## Self-Improvement
- Track which queries yield best results
- Adjust strategy based on feedback
```

### Features to Implement

- [ ] Skill loader from `.ao/skills/*.md`
- [ ] YAML frontmatter parser
- [ ] Skill trigger matching
- [ ] Workflow execution engine
- [ ] Usage tracking (count, success rate)
- [ ] Self-improvement (modify skill on feedback)
- [ ] Version management

---

## F7: Session Persistence

**Purpose**: Store session state with full lineage tracking.

### Schema

```sql
-- Sessions
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    name TEXT,
    created_at TIMESTAMP,
    last_active_at TIMESTAMP,
    initial_memory_snapshot TEXT,
    initial_profile_snapshot TEXT,
);

-- Messages with lineage
CREATE TABLE messages (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    lineage_id TEXT NOT NULL,
    parent_lineage_id TEXT,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    compressed BOOLEAN DEFAULT FALSE,
    compression_summary TEXT,
);

-- Compression events
CREATE TABLE compression_events (
    session_id TEXT,
    lineage_id TEXT,
    method TEXT,
    original_tokens INTEGER,
    compressed_tokens INTEGER,
    archived_messages TEXT,
);
```

### Features to Implement

- [ ] Session creation and restoration
- [ ] Message persistence with lineage
- [ ] Compression event tracking
- [ ] Session state reconstruction from lineage
- [ ] Frozen snapshot handling (memory, profile)

---

## F8: Scheduler

**Purpose**: Cron jobs as first-class tasks.

### Human-Readable Syntax

| Input | Cron Equivalent |
|-------|-----------------|
| `every 5 minutes` | `*/5 * * * *` |
| `every hour` | `0 * * * *` |
| `every day at 9:00` | `0 9 * * *` |
| `on mondays at 18:00` | `0 18 * * 1` |

### Job Types

| Type | Description |
|------|-------------|
| `tool` | Execute a tool |
| `skill` | Run a skill |
| `worker` | Spawn a worker |
| `maintenance` | Internal tasks (cleanup, health check) |

### Features to Implement

- [ ] Human-readable schedule parser
- [ ] Job types (tool, skill, worker, maintenance)
- [ ] Job persistence in SQLite
- [ ] Execution with retry logic
- [ ] Job history and status

---

## F9: TUI (Chat Interface)

**Purpose**: Primary user interaction.

### Layout

```
┌─────────────────────────────────────────┐
│  Pantheon v1.0          [Status: Ready] │
├─────────────────────────────────────────┤
│  You: Create a note about API design    │
│                                         │
│  Ao: I'll help you create that note.   │
│      Opening the editor...              │
│                                         │
│  [Tool: editor.open - success]         │
│                                         │
├─────────────────────────────────────────┤
│  > _                                    │
├─────────────────────────────────────────┤
│  [Workers: 0/2] [Memory: 42]            │
└─────────────────────────────────────────┘
```

### Message Types

| Type | Display |
|------|---------|
| `user` | Left-aligned |
| `ao` | Right-aligned, bold |
| `worker` | Right-aligned, colored |
| `tool` | Monospace, muted |
| `system` | Centered, muted |

### Commands

Commands are handled by the TUI - this is part of F9 (TUI), not a separate feature. Commands provide user control over the system.

| Command | Description |
|---------|-------------|
| `/quit` | Exit the application |
| `/help` | Show available commands |
| `/status` | Show system status |
| `/clear` | Clear chat history |
| `/remember <query>` | Add relevant memory/library to context |
| `/forget` | Clear short-term context |
| `/context` | Show current context |

### Features to Implement

- [ ] Basic TUI layout (header, messages, input, status)
- [ ] Message rendering (different styles per type)
- [ ] Input handling
- [ ] Command parsing (/prefix)
- [ ] Tool result display
- [ ] Status bar updates

---

## F10: Context Files

**Purpose**: External context loaded into prompts.

### Files

| File | Description |
|------|-------------|
| `.ao/ao.md` | Ao's identity |
| `.ao/skills/*.md` | Skill definitions |
| `.cursorrules` | Cursor IDE rules |
| `.cursor/rules/*.mdc` | Additional context |

### Features to Implement

- [ ] File discovery and loading
- [ ] Markdown + YAML parsing
- [ ] Cached vs dynamic loading
- [ ] Template variable substitution ({agent_list}, {tool_descriptions}, etc.)

---

## Summary: Feature Priority

| Priority | Feature | Description |
|----------|---------|-------------|
| 1 | F0 | Library System - user's knowledge base (CORE) |
| 2 | F2 | Agent Loop - core execution |
| 3 | F1 | Prompt System - context assembly |
| 4 | F3 | Tool System - execution |
| 5 | F9 | TUI - user interface |
| 6 | F4 | Worker System - parallel execution |
| 7 | F5 | Memory System - persistence |
| 8 | F7 | Session Persistence - state |
| 9 | F6 | Skills System - workflows |
| 10 | F8 | Scheduler - automation |
| 11 | F10 | Context Files - external loading |

---

## Open Questions

1. **Provider support**: Which LLM providers to support initially? (OpenRouter, Ollama, both?)
2. **Memory granularity**: Per-worker memory vs shared memory?
3. **Skill improvement**: Fully autonomous or user-approved?
4. **TUI framework**: Use ratatui, cursive, or custom?
5. **Database**: Single SQLite file or multiple?
