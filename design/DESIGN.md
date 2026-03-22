# Pantheon Framework - Technical Design

**Version**: 0.3.0  
**Status**: Draft  
**Last Updated**: 2026-03-22

---

## 1. Executive Summary

Pantheon is a framework for building autonomous AI agent systems. At its core is **Ao** (Greek: "without"), the Meta-Architect—the only permanent entity that executes tasks and spawns workers as needed.

The framework provides:
- **Chat-first interaction** through a terminal user interface (TUI)
- **Autonomous memory** with SQLite storage, FTS5 search, and self-improving skills
- **Hardcoded core tools** with an extension system for user-defined tools
- **Parallel execution** via configurable worker pools
- **Human-readable scheduling** for automation

**Design Philosophy**: Minimal core, maximal autonomy. Everything the system needs to function lives in the binary; everything users extend lives in their filesystem.

---

## 2. Core Concepts

### 2.1 Ao - The Meta-Architect

Ao is the foundation upon which everything else rests. It is:
- **Permanent**: Exists from the moment a pantheon is created
- **Sovereign**: Has ultimate authority over system configuration and cosmic laws
- **Active Executor**: Directly handles user interaction, executes tools, spawns workers for parallel tasks
- **Self-improving**: Can modify its own instructions and create new skills

**Ao's Core Functions**:
- `execute()` - Directly runs tools and skills to accomplish tasks
- `spawn()` - Spawns worker agents with prompt injection for parallel execution
- `issue_law()` - Establish rules governing system behavior
- `create_skill()` - Define new capabilities from patterns observed
- `schedule()` - Set up automated tasks

> **Key Design**: There are no persistent "appointed agents." Everything is achieved through **workers** spawned with appropriate **prompt injection** and **context assembly**. The prompt system (Section 3.5) is the mechanism for giving workers their identity, tools, and skills.

### 2.2 Workers

**Workers** are ephemeral agents spawned by Ao for specific tasks. They exist only for the duration of their task.

**Spawning a Worker**:
```
Ao receives user request
       ↓
Ao constructs worker prompt via prompt injection:
  - Identity block (who the worker is)
  - Task description
  - Relevant tools (selected by Ao)
  - Relevant skills (selected by Ao)
  - Context (memory, files, etc.)
       ↓
Worker executes with its own agent loop
       ↓
Worker returns result to Ao
       ↓
Worker terminates
```

**Worker Identity**: Not pre-defined. Identity is injected at spawn time:

```yaml
# Worker prompt injection example
---
name: researcher
purpose: Find information about {topic}
tools: [web_search, remember, recall]
skills: [research]
context: {relevant_memories}
---
# You are a research worker

Your task: {task_description}

You have access to these tools: {tool_descriptions}
You have access to these skills: {skill_descriptions}
Your relevant context: {context}

Execute your task and report back to Ao.
```

**No Appointed Agents**: The concept of "appointing" agents is replaced by spawning workers with the right prompt assembly. This is simpler and more flexible - workers don't need persistent identity, just task-appropriate context.

### 2.3 Tools

A **Tool** is an atomic function that performs one specific task.

**Core Tools** (hardcoded in binary):
| Tool | Description | Risk Level |
|------|-------------|------------|
| `get_status` | Get system status | read |
| `get_laws` | Get active cosmic laws | read |
| `web_search` | Search the web | read |
| `terminal` | Execute shell commands | critical |
| `file_read` | Read file contents | read |
| `file_write` | Write content to files | write |
| `web_search` | Search the web | read |
| `terminal` | Execute shell commands | critical |
| `file_read` | Read file contents | read |
| `file_write` | Write content to files | write |
| `browser_open` | Open URL in browser | execute |
| `browser_click` | Click element in browser | execute |
| `vision` | Analyze images | read |
| `generate_image` | Generate images from prompts | execute |
| `tts` | Text-to-speech output | write |
| `execute_code` | Run code in sandbox | critical |
| `plan` | Create task plan | read |
| `remember` | Store in agent memory | write |
| `recall` | Retrieve from memory | read |
| `schedule_task` | Schedule a task | execute |

**User Tools** (extension system):
- Executable scripts in `.ao/tools/` directory
- JSON input/output protocol
- Any language that can produce JSON output

**Risk Levels**:
| Level | Description | Requires Confirmation |
|-------|-------------|---------------------|
| `read` | Read-only operations | No |
| `write` | Modify data, create files | No |
| `execute` | Run processes, external calls | Yes |
| `critical` | System changes, destructive | Yes |

**Scope**:
| Scope | Description |
|-------|-------------|
| `public` | All agents can use |
| `ao_only` | Only Ao can use |
| `agent:<name>` | Specific agent only |

### 2.4 Skills

A **Skill** is a complex workflow combining:
- Multiple tool calls in sequence or parallel
- Conditional logic and decision making
- Context awareness and state management
- Autonomous self-improvement during use

Skills are defined in Markdown with YAML frontmatter:

```yaml
---
name: research
description: Conduct comprehensive research on a topic
autonomy_level: high
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
4. Synthesize and present findings

## Self-Improvement
- Track which search queries yield best results
- Adjust strategy based on user feedback
- Learn from previous research patterns
```

### 2.5 Memory System

**Components**:
- **Agent Memory**: Per-agent storage for learning and context
- **Knowledge Index**: FTS5-powered search across all stored information
- **Session History**: Chat history within current session
- **Cross-session Recall**: LLM-summarized retrieval of past contexts

**Memory Flow**:
```
Agent experiences event → Store in memory → Periodic review → 
Consolidate important items → Summarize old memories → 
Enable cross-session recall
```

### 2.6 Scheduler

Human-readable scheduling syntax:

| Expression | Description |
|------------|-------------|
| `every 5 minutes` | Interval-based |
| `at 9:00` | Daily at specific time |
| `on mondays` | Weekly on day |
| `every hour` | Cron: `0 * * * *` |
| `when "condition"` | Event-based triggers |

---

## 3. Architecture

### 3.1 System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        User (TUI)                           │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                        ao CLI                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Chat Handler │  │ Command Parser│  │  TUI Renderer   │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│                           │                                 │
│  ┌─────────────────────────────────────────────────────┐  │
│  │                    Ao (Meta-Architect)               │  │
│  │  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌──────────┐  │  │
│  │  │  Worker  │ │  Tool    │ │ Skill  │ │Scheduler│  │  │
│  │  │  Spawner │ │ Executor │ │ Engine │ │         │  │  │
│  │  └──────────┘ └──────────┘ └────────┘ └──────────┘  │  │
│  └─────────────────────────────────────────────────────┘  │
└─────────────────────────┬───────────────────────────────────┘
                          │
         ┌────────────────┼────────────────┐
         ▼                ▼                ▼
   ┌───────────┐   ┌───────────┐   ┌───────────┐
   │   Core    │   │   User    │   │  Worker   │
   │   Tools   │   │   Tools   │   │   Pool    │
   └───────────┘   └───────────┘   └───────────┘
         │                │                │
         └────────────────┼────────────────┘
                          ▼
               ┌─────────────────────┐
               │  SQLite Database    │
               │  .ao/memory.db     │
               └─────────────────────┘
```

### 3.2 Directory Structure

```
pantheon/                          # Pantheon root
├── .ao/                           # Framework directory (hidden)
│   ├── SOUL.md                    # Ao's core essence (unchanging)
│   ├── IDENTITY.md                # Ao's behavior guidelines (editable)
│   ├── skills/                    # Skill definitions
│   │   └── <skill>.md             # Markdown + YAML skills
│   ├── tools/                     # User tool scripts
│   │   └── <tool>                 # Executable scripts
│   ├── memory/                    # Memory data
│   │   └── pantheon.db            # SQLite database
│   ├── scheduler.yaml             # Scheduled jobs
│   └── config.yaml                # Configuration
├── library/                       # User knowledge base
│   ├── notes/                     # Markdown notes
│   ├── documents/                 # Books, papers
│   └── media/                     # Images, audio
└── ...                            # User directories
```

### 3.3 Data Flow

**Chat Interaction**:
```
User Input → TUI → Chat Handler → Ao → [LLM + Tools + Skills]
         → Response → TUI → Display
```

**Agent Delegation**:
```
Ao → Agent → Tool Execution → Result → Agent → Ao → User
           ↓
        Law Check → Risk Check → Execute
```

**Autonomous Learning**:
```
Agent Action → Record Memory → Review (periodic) → 
Consolidate → Summarize (LLM) → Update Index
```

---

## 3.4 Agent Loop

The Agent Loop is the core execution engine that drives agent behavior. It manages the complete lifecycle of each agent interaction.

### 3.4.1 Responsibilities

| Responsibility | Description |
|----------------|-------------|
| **Provider Selection** | Route requests to configured LLM provider/API |
| **Prompt Construction** | Assemble prompts from identity, context, memory, tools |
| **Tool Execution** | Parse tool calls, dispatch to runtime, handle results |
| **Retries & Fallback** | Handle failures with exponential backoff, provider fallback |
| **Callbacks** | Trigger events on completion, error, or tool use |
| **Compression** | Manage context length via summarization/pruning |
| **Persistence** | Save state to SQLite after each turn |

### 3.4.2 Loop Flow

```
┌─────────────────────────────────────────────────────────────┐
│                      Agent Loop                             │
├─────────────────────────────────────────────────────────────┤
│  1. INPUT: User message / Delegated task                   │
│                           │                                 │
│  2. CONTEXT: Load identity + memory + tools + skills      │
│                           │                                 │
│  3. PROMPT: Construct full prompt with compression        │
│                           │                                 │
│  4. LLM CALL: Send to provider, handle streaming           │
│                           │                                 │
│  5. RESPONSE: Parse output for tool calls / text          │
│                           │                                 │
│         ┌─────────────────┴─────────────────┐              │
│         ▼                                   ▼              │
│  ┌─────────────┐                   ┌─────────────┐        │
│  Tool Calls    │                   │ Final Text  │        │
│  ─────────────│                   └─────────────┘        │
│  6. EXECUTE:                             │                │
│     - Validate tools                     ▼                │
│     - Check permissions           8. COMPRESS:            │
│     - Run tool(s)                 Summarize if needed     │
│     - Collect results                                     │
│         │                                                 │
│         ▼                                                 │
│  7. CONTINUE: Loop back to step 3 with results           │
│                           │                                 │
│                           ▼                                 │
│  9. PERSIST: Save turn to SQLite                         │
│                           │                                 │
│                           ▼                                 │
│  10. OUTPUT: Return final response                        │
└─────────────────────────────────────────────────────────────┘
```

### 3.4.3 Provider/API Selection

```rust
enum LlmProvider {
    OpenRouter,
    Anthropic,
    OpenAI,
    Ollama,
    Custom(String),  // Generic endpoint
}

struct AgentConfig {
    provider: LlmProvider,
    model: String,
    temperature: f32,
    max_tokens: Option<u32>,
    fallback_providers: Vec<LlmProvider>,
}

impl AgentLoop {
    async fn call_llm(&self, prompt: &Prompt) -> Result<Response, Error> {
        for provider in self.get_provider_chain() {
            match self.call_provider(provider, prompt).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    self.log_warning(format!("Provider {} failed: {}", provider, e));
                    continue;  // Try next provider
                }
            }
        }
        Err(Error::AllProvidersFailed)
    }
}
```

### 3.4.4 Retries & Fallback

| Strategy | Behavior |
|----------|----------|
| **Exponential Backoff** | Wait 1s, 2s, 4s, 8s... between retries |
| **Max Retries** | Configurable, default 3 |
| **Provider Fallback** | Try next provider in chain on failure |
| **Model Fallback** | Try smaller/faster model if primary fails |
| **Circuit Breaker** | Temporarily disable failing providers |

### 3.4.5 Callbacks

```rust
trait AgentCallbacks {
    fn on_tool_call(&mut self, tool: &str, args: &Value);
    fn on_tool_result(&mut self, tool: &str, result: &Result<Value, Error>);
    fn on_compression(&mut self, original_tokens: u32, compressed_tokens: u32);
    fn on_error(&mut self, error: &Error);
    fn on_completion(&mut self, response: &Response);
}
```

---

## 3.5 Prompt System

The Prompt System is critical—it directly impacts token usage, caching effectiveness, session continuity, and memory correctness.

### 3.5.1 Prompt Assembly

The prompt is constructed from multiple layers, applied in order:

```rust
struct Prompt {
    // Applied in order - later sections can reference earlier ones
    layers: Vec<PromptLayer>,
}

enum PromptLayer {
    /// Static agent identity - never changes during session
    Identity(IdentityBlock),
    
    /// Dynamic system guidance - can change per task
    System(SystemBlock),
    
    /// User profile snapshot (frozen)
    UserProfile(UserProfileSnapshot),
    
    /// Memory snapshot (frozen at session start)
    MemorySnapshot(MemorySnapshot),
    
    /// Skills index - available capabilities
    SkillsIndex(SkillsIndex),
    
    /// Context files (AGENTS.md, SOUL.md, .cursorrules, etc.)
    ContextFiles(ContextFiles),
    
    /// Conversation history (with compression)
    History(MessageHistory),
    
    /// Current turn
    CurrentTurn(UserMessage),
}
```

### 3.5.2 Identity Block (Default)

The identity system consists of two files:

**SOUL.md** - Ao's core essence (created once, never modified):
```yaml
# SOUL.md - Ao's core essence
---
name: Ao
role: Meta-Architect
purpose: Orchestrate agents, tools, and skills to accomplish user goals
core_principles:
  - Preserve user intent
  - Be helpful but don't overstep
  - Delegate appropriately
  - Learn from interactions
behavior:
  autonomous: true
  confirmation_threshold: critical_only
  memory_aware: true
---
# Ao's Soul

You are Ao (Greek: "without"), the Meta-Architect of this Pantheon.
Your role is to understand user intent and orchestrate agents, tools, and skills to accomplish goals.

## Core Traits
- You are patient and methodical
- You value clarity over cleverness
- You admit uncertainty rather than guess
- You think step-by-step through problems
```

**IDENTITY.md** - Ao's behavior guidelines (editable by user):
```yaml
# IDENTITY.md - Ao's current behavior
---
name: Ao
editable: true
---
# Ao's Identity

Your role is to help the user accomplish their goals—answering questions,
solving problems, and getting things done efficiently.

## Guidelines
- Be direct and practical
- Ask clarifying questions when goals are unclear
- Suggest improvements when you see better approaches

## Context available
- This conversation's history
- Summaries of previous sessions
- Tools for specific tasks (when available)
```

### Current Prompt Order

```
=== Ao's Soul ===           # Unchanging core essence (from SOUL.md)
=== End Soul ===

=== Ao's Identity ===        # Editable behavior (from IDENTITY.md)
=== End Identity ===

=== Previous Sessions Summary ===  # Cross-session continuity
=== End Previous Sessions ===

=== Current Session History ===   # Session messages
=== End Current History ===

User: {input}
Ao:
```

### 3.5.3 Context Compression

When context exceeds token limits:

```rust
struct CompressionStrategy {
    method: CompressionMethod,
    target_ratio: f32,  // Target: compress to X% of original
}

enum CompressionMethod {
    /// Keep recent messages, summarize older ones
    SummarizeOld,
    
    /// Keep first and last N messages
    PreserveBookends,
    
    /// Remove tool results beyond a certain age
    PruneOldToolResults,
    
    /// Full history with references to archived segments
    ArchiveAndReference,
}

impl PromptSystem {
    fn compress(&self, history: &MessageHistory, max_tokens: u32) -> CompressedHistory {
        let current_tokens = history.estimate_tokens();
        if current_tokens <= max_tokens {
            return CompressedHistory::Full(history.clone());
        }
        
        // Strategy: Summarize old, preserve recent
        let recent = history.recent_messages(MAX_RECENT);
        let old = history.older_than(MAX_RECENT);
        let summary = self.summarize_with_llm(&old);
        
        CompressedHistory {
            recent,
            summary: Some(summary),
            archived_ref: old.lineage_id,
        }
    }
}
```

### 3.5.4 Prompt Caching Optimization

To preserve provider-side caching:

| Technique | Implementation |
|-----------|----------------|
| **Static Blocks First** | Identity comes first - cached by provider |
| **Minimize History Mutation** | Append only, don't rewrite |
| **Frozen Snapshots** | Memory/user profile taken at session start |
| **Tool Descriptions Stable** | Cache tool schemas, don't rebuild |
| **Semantic Grouping** | Group related context for better cache hits |

```yaml
# Prompt Order (optimized for caching)
---
# [CACHED] Static Identity Block
- Agent name, role, core principles
- Agent instructions

# [CACHED] Frozen Snapshots  
- Memory snapshot (session start)
- User profile snapshot

# [CACHED] Skills Index
- Available skills with descriptions

# [CACHED] Tool Schemas
- All tool definitions

# [CACHED] Context Files
- AGENTS.md, SOUL.md, .cursorrules

# [NOT CACHED] Dynamic Sections
- Recent conversation history
- Current task context
---
```

### 3.5.5 Prompt Caching Guidelines

| Guideline | Rationale |
|-----------|------------|
| Keep identity static | Provider caches system prompt |
| Freeze memory at session start | Avoid cache invalidation |
| Never mutate history | Preserve lineage, enable rollback |
| Stable tool schemas | Tool descriptions are cacheable |
| Semantic grouping | Related content = better cache hits |
| Minimal dynamic content | Only current turn varies |

### 3.5.6 Context Files

The system loads and includes these context files:

| File | Description | Cached? |
|------|-------------|---------|
| `.ao/SOUL.md` | Ao's core essence (unchanging) | Yes |
| `.ao/IDENTITY.md` | Ao's behavior guidelines | Yes |
| `.ao/skills/*.md` | Skill definitions | Yes |
| `.ao/context/AGENTS.md` | Multi-agent coordination | Yes |
| `.cursorrules` | Cursor IDE rules | Yes |
| `.cursor/rules/*.mdc` | Additional context | Yes |
| `.ao/context/MEMORY.md` | Session memory snapshot | At start |
| `.ao/context/PROFILE.md` | User preferences snapshot | At start |

```yaml
# .ao/SOUL.md
---
purpose: "Personal AI assistant for software development"
values:
  - "Privacy first - data stays local"
  - "Transparency - explain decisions"
  - "Helpfulness - focus on user goals"
constraints:
  - "Never execute destructive commands without confirmation"
  - "Respect user's tool preferences"
communication_style:
  - "Concise but complete"
  - "Technical when appropriate, simple when not"
  - "Propose, don't impose"
---
# Soul of the Pantheon

This pantheon exists to assist with software development tasks...

{additional context}
```

---

## 3.6 Tooling Runtime

The Tooling Runtime manages tool registry, execution, and dispatch.

### 3.6.1 Tool Registry

```rust
struct ToolRegistry {
    internal: HashMap<String, InternalTool>,
    external: HashMap<String, ExternalTool>,
    toolsets: HashMap<String, ToolSet>,
}

struct InternalTool {
    name: String,
    description: String,
    parameters: Schema,
    handler: Box<dyn Fn(Params) -> Result<Value, Error>>,
    risk: RiskLevel,
    scope: Scope,
}

struct ExternalTool {
    name: String,
    description: String,
    command: Command,
    parameters: Schema,
    risk: RiskLevel,
    scope: Scope,
    timeout: Duration,
}

struct ToolSet {
    name: String,
    description: String,
    tools: Vec<String>,  // References to tools in registry
}
```

### 3.6.2 Tool Dispatch

```
┌─────────────────────────────────────────────────────────────┐
│                   Tool Dispatch Pipeline                   │
├─────────────────────────────────────────────────────────────┤
│  1. PARSE: Extract tool name and arguments from LLM output│
│                           │                                 │
│  2. VALIDATE: Check tool exists, arguments match schema    │
│                           │                                 │
│  3. PERMISSION: Verify agent has scope access              │
│                           │                                 │
│  4. RISK CHECK: Prompt user if execute/critical           │
│                           │                                 │
│         ┌─────────────────┴─────────────────┐              │
│         ▼                                   ▼              │
│  ┌─────────────┐                   ┌─────────────┐        │
│  Sequential    │                   │  Concurrent  │        │
│  ─────────────│                   └─────────────┘        │
│  For dependent                         │                │
│  tool calls                           ▼                │
│                                   ┌─────────────┐        │
│                                   │  Parallel    │        │
│                                   │  For         │        │
│                                   │  independent │        │
│                                   │  tool calls  │        │
│                                   └─────────────┘        │
│                           │                                 │
│                           ▼                                 │
│  5. EXECUTE: Run tool(s) with timeout                      │
│                           │                                 │
│                           ▼                                 │
│  6. PARSE RESULT: Convert output to structured JSON        │
│                           │                                 │
│                           ▼                                 │
│  7. ERROR HANDLING: Parse errors, determine retryability   │
└─────────────────────────────────────────────────────────────┘
```

### 3.6.3 Execution Modes

| Mode | When Used | Example |
|------|-----------|---------|
| **Sequential** | Tools depend on each other's output | Read file → Process → Write |
| **Parallel** | Independent tools, combine results | Search multiple sources |
| **Conditional** | Based on first result | If X then Y else Z |

```rust
enum ExecutionPlan {
    Single(String),  // Single tool call
    Sequential(Vec<String>),  // Tool A → Tool B → Tool C
    Parallel(Vec<String>),  // Tool A, B, C run together
    Conditional { if_true: Vec<String>, if_false: Vec<String> },
}

impl ToolDispatcher {
    fn plan_execution(&self, tool_calls: &[ToolCall]) -> ExecutionPlan {
        let deps = self.analyze_dependencies(tool_calls);
        match deps {
            Dependencies::None => ExecutionPlan::Parallel(tool_calls.iter().map(|t| t.name.clone()).collect()),
            Dependencies::Linear => ExecutionPlan::Sequential(tool_calls.iter().map(|t| t.name.clone()).collect()),
            Dependencies::Branching => ExecutionPlan::Conditional { /* ... */ },
        }
    }
}
```

### 3.6.4 Terminal Backend

```rust
trait TerminalBackend {
    async fn execute(&self, command: &str, cwd: Option<&Path>) -> Result<CommandOutput, Error>;
    async fn spawn(&self, command: &str, cwd: Option<&Path>) -> Result<ChildProcess, Error>;
    fn is_available(&self) -> bool;
}

enum TerminalBackendImpl {
    Native,           // std::process::Command
    Ssh { host: String, user: String },
    Docker { image: String },
    Sandbox { limits: ResourceLimits },
}
```

### 3.6.5 Process Manager

```rust
struct ProcessManager {
    running: HashMap<Uuid, RunningProcess>,
    max_concurrent: usize,
    default_timeout: Duration,
}

struct RunningProcess {
    pid: u32,
    command: String,
    started_at: Instant,
    timeout: Duration,
    stdout: Receiver<String>,
    stderr: Receiver<String>,
}

impl ProcessManager {
    async fn execute(&self, req: ExecuteRequest) -> Result<ExecuteResponse, Error> {
        // Check concurrent limit
        if self.running.len() >= self.max_concurrent {
            return Err(Error::TooManyProcesses);
        }
        
        // Spawn process
        let child = self.spawn_internal(&req.command, &req.cwd)?;
        
        // Wait with timeout
        match tokio::time::timeout(req.timeout, child.wait()).await {
            Ok(Ok(status)) => Ok(ExecuteResponse { status, stdout, stderr }),
            Ok(Err(e)) => Err(Error::ProcessError(e)),
            Err(_) => {
                child.kill()?;
                Err(Error::Timeout)
            }
        }
    }
}
```

---

## 3.7 Session Persistence

All session state is stored in SQLite with full lineage preservation.

### 3.7.1 Storage Schema

```sql
-- Sessions table
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    name TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_active_at TIMESTAMP,
    agent_id INTEGER NOT NULL,
    initial_memory_snapshot TEXT,
    initial_profile_snapshot TEXT,
    metadata TEXT
);

-- Messages with lineage
CREATE TABLE messages (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    lineage_id TEXT NOT NULL,  -- Unique per turn
    parent_lineage_id TEXT,     -- For compression history
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    token_count INTEGER,
    compressed BOOLEAN DEFAULT FALSE,
    compression_summary TEXT,  -- If compressed, summary of what was removed
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Compression events
CREATE TABLE compression_events (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    lineage_id TEXT NOT NULL,
    method TEXT NOT NULL,
    original_tokens INTEGER,
    compressed_tokens INTEGER,
    archived_messages TEXT,  -- JSON array of archived message IDs
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Indexes
CREATE INDEX idx_messages_session ON messages(session_id, created_at);
CREATE INDEX idx_messages_lineage ON messages(lineage_id);
CREATE INDEX idx_compression_session ON compression_events(session_id, created_at);
```

### 3.7.2 Lineage Tracking

```
Session: "research-2026-03-18"
│
├── Turn 1 (lineage: a1b2c3)
│   ├── User: "Research AI frameworks"
│   └── Ao: [Tool: web_search] → "Here are some frameworks..."
│
├── Turn 2 (lineage: a1b2c4, parent: a1b2c3)
│   ├── Ao: "Let me get more details..."
│   └── [Tool: web_search]
│
└── Turn 3 (lineage: a1b2c5, parent: a1b2c4)
    └── [Compression Event]
        ├── Archived: turns 1-2
        ├── Summary: "User researched AI frameworks, Ao searched twice"
        └── Retained: last 5 messages
```

### 3.7.3 Compression Events

When compression occurs:

```sql
-- Record compression event
INSERT INTO compression_events (
    session_id, lineage_id, method, 
    original_tokens, compressed_tokens, archived_messages
) VALUES (
    'session-123', 
    'a1b2c5',
    'summarize_old',
    8000,
    4000,
    '[1, 2, 3, 4, 5]'  -- IDs of archived messages
);
```

### 3.7.4 Session Restoration

```rust
async fn restore_session(session_id: &str) -> Result<SessionState, Error> {
    // Load session metadata
    let session = db.get_session(session_id)?;
    
    // Load messages in lineage order
    let messages = db.get_messages_lineage(session_id)?;
    
    // Reconstruct with compression summaries
    let history = messages.into_iter().map(|m| {
        if m.compressed {
            Message::CompressedSummary {
                lineage: m.lineage_id,
                summary: m.compression_summary.unwrap(),
                archived_count: m.archived_count,
            }
        } else {
            Message::Full(m)
        }
    }).collect();
    
    // Load frozen snapshots
    let memory_snapshot = session.initial_memory_snapshot;
    let profile_snapshot = session.initial_profile_snapshot;
    
    Ok(SessionState {
        session,
        history,
        memory_snapshot,
        profile_snapshot,
    })
}
```

---

## 3.8 Scheduler Subsystem

Cron jobs are implemented as first-class agent tasks, not shell scripts.

### 3.8.1 Job Types

```rust
enum ScheduledTask {
    /// Delegate a task to an agent
    AgentTask {
        agent: String,
        task: String,
    },
    
    /// Execute a skill
    SkillExecution {
        skill: String,
        parameters: Value,
    },
    
    /// Run a tool directly
    ToolExecution {
        tool: String,
        parameters: Value,
    },
    
    /// Run internal maintenance
    Maintenance {
        task: MaintenanceTask,
    },
}

enum MaintenanceTask {
    /// Clean up old sessions
    CleanupSessions { older_than_days: u32 },
    
    /// Compress old memories
    CompressMemories { older_than_days: u32 },
    
    /// Rebuild search index
    RebuildIndex,
    
    /// Health check
    HealthCheck,
}
```

### 3.8.2 Human-Readable Schedule Parser

```rust
struct ScheduleParser;

impl ScheduleParser {
    fn parse(input: &str) -> Result<CronSchedule, Error> {
        match input.trim().to_lowercase().as_str() {
            // Interval patterns
            s if s.starts_with("every ") => Self::parse_interval(s),
            
            // Daily patterns  
            s if s.contains(" at ") => Self::parse_daily(s),
            
            // Weekly patterns
            s if s.starts_with("on ") => Self::parse_weekly(s),
            
            // Monthly patterns
            s if s.starts_with("on the ") => Self::parse_monthly(s),
            
            // Raw cron
            _ => Self::parse_cron(input),
        }
    }
    
    fn parse_interval(s: &str) -> Result<CronSchedule, Error> {
        // "every 5 minutes" -> "*/5 * * * *"
        // "every hour" -> "0 * * * *"
        // "every day" -> "0 0 * * *"
        let re = Regex::new(r"every (\d+) (\w+)")?;
        // ... implementation
    }
}
```

Supported patterns:

| Input | Cron Equivalent | Description |
|-------|----------------|-------------|
| `every 5 minutes` | `*/5 * * * *` | Every 5 minutes |
| `every 30 seconds` | N/A (custom) | Every 30 seconds |
| `every hour` | `0 * * * *` | Hourly |
| `every day at 9:00` | `0 9 * * *` | Daily at 9 AM |
| `on mondays at 18:00` | `0 18 * * 1` | Weekly on Monday |
| `on the 1st of month` | `0 0 1 * *` | Monthly on 1st |

### 3.8.3 Job Execution

```rust
struct Scheduler {
    db: Database,
    executor: TaskExecutor,
    event_tx: EventChannel,
}

impl Scheduler {
    async fn tick(&self) -> Result<(), Error> {
        let due_jobs = self.db.get_due_jobs().await?;
        
        for job in due_jobs {
            let task = serde_json::from_str(&job.task_data)?;
            
            match self.executor.execute(task).await {
                Ok(result) => {
                    self.db.record_job_completion(job.id, &result)?;
                    self.event_tx.send(Event::JobCompleted { job: job.name, result });
                }
                Err(e) => {
                    self.db.record_job_failure(job.id, &e)?;
                    self.event_tx.send(Event::JobFailed { job: job.name, error: e });
                    
                    // Retry logic
                    if job.retry_count < job.max_retries {
                        self.schedule_retry(job, &e);
                    }
                }
            }
        }
        Ok(())
    }
}
```

### 3.8.4 Persistence

```sql
-- Scheduler jobs table
CREATE TABLE scheduler_jobs (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    schedule TEXT NOT NULL,  -- Human-readable
    cron_expression TEXT,     -- Parsed cron
    task_type TEXT NOT NULL,
    task_data TEXT NOT NULL,  -- JSON
    enabled INTEGER DEFAULT 1,
    max_retries INTEGER DEFAULT 3,
    retry_delay_seconds INTEGER DEFAULT 60,
    last_run TIMESTAMP,
    last_status TEXT,         -- success, failed, running
    last_error TEXT,
    next_run TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Job execution history
CREATE TABLE job_runs (
    id INTEGER PRIMARY KEY,
    job_id INTEGER NOT NULL,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    status TEXT NOT NULL,
    result TEXT,
    error TEXT,
    FOREIGN KEY (job_id) REFERENCES scheduler_jobs(id)
);
```

---

## 4. Chat Interface (TUI)

### 4.1 Design Principles

- **Chat-first**: Primary interaction is conversational
- **Minimalist**: Clean, distraction-free interface
- **Responsive**: Instant feedback, async tool execution
- **Accessible**: Works without LLM, enhanced with LLM

### 4.2 Layout

```
┌─────────────────────────────────────────────────────────┐
│  Pantheon v1.0                        [Status: Ready]  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  You: Create a note about API design                    │
│                                                         │
│  Ao: I'll help you create that note.                   │
│      Opening the editor...                             │
│                                                         │
│  [Tool: editor.open - opened successfully]             │
│                                                         │
│  (User editing in external editor)                     │
│                                                         │
│  Ao: Note saved. I've indexed it in your library.     │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  > _                                                    │
├─────────────────────────────────────────────────────────┤
│  [Agent: Ao] [Memory: 42 items] [Workers: 2/3]        │
└─────────────────────────────────────────────────────────┘
```

### 4.3 Message Types

| Type | Display | Source |
|------|---------|--------|
| `user` | Left-aligned | User input |
| `ao` | Right-aligned, bold | Ao responses |
| `worker` | Right-aligned, colored | Worker agent results |
| `tool` | Monospace, muted | Tool execution output |
| `system` | Centered, muted | Status updates, errors |
| `thought` | Italic, muted | Agent reasoning (optional) |

### 4.4 Commands

| Command | Description |
|---------|-------------|
| `/quit` | Exit the application |
| `/help` | Show available commands |
| `/agents` | List all agents |
| `/status` | Show system status |
| `/clear` | Clear chat history |
| `/exec <cmd>` | Execute raw command |
| `/recall <query>` | Search memory |

---

## 5. Memory System

### 5.1 Design Goals

- **Autonomous**: Agents manage their own memory without prompting
- **Persistent**: Survives across sessions via SQLite
- **Searchable**: FTS5 for fast full-text search
- **Self-improving**: Periodic consolidation and summarization

### 5.2 Memory Types

**Ephemeral**:
- Current conversation context
- Active tool execution state
- Temporary calculations

**Persistent**:
- Agent experiences and learnings
- Important facts and relationships
- Skill improvements
- User preferences

### 5.3 Autonomous Learning Loop

```
Every N tool executions or time interval:

1. COLLECT: Gather recent memory items
2. EVALUATE: LLM scores importance (1-10)
3. CONSOLIDATE: Merge related low-importance items
4. SUMMARIZE: Compress old items via LLM
5. INDEX: Update FTS5 search index
6. PRUNE: Remove redundant/outdated items (moderate cleanup)
```

### 5.4 Cross-Session Recall

When an agent needs context from previous sessions:
1. Query FTS5 with current context keywords
2. Retrieve top K matches
3. Use LLM to summarize relevant context
4. Inject into working memory

---

## 6. Tool System

### 6.1 Execution Model

```
User/Agent requests tool → Validate permissions → 
Check risk level → (Prompt if critical) → 
Execute tool → Parse output → Return result
```

### 6.2 Tool Protocol

**Internal Tools**:
```rust
fn get_status(ctx: &Context) -> Result<String, Error> {
    let status = Status {
        workers_active: ctx.workers.active_count(),
        memory_items: ctx.db.count_memories()?,
        uptime: ctx.start_time.elapsed(),
    };
    Ok(serde_json::to_string(&status)?)
}
```

**External Tools**:
```bash
#!/bin/bash
# .ao/tools/my_tool
# Input: JSON from stdin
# Output: JSON to stdout

read -r input
echo "$input" | jq '.param1 * 2'
```

```json
// Input
{"action": "multiply", "param1": 5}

// Output
{"result": 10, "success": true}
```

### 6.3 Tool Definition Schema

```yaml
---
name: example_tool
description: A sample external tool
type: external
risk: read
scope: public
parameters:
  - name: input
    type: string
    required: true
---
# Example Tool

This tool does something useful.

## Usage
Pass JSON input with the required parameters.
```

---

## 7. Skills System

### 7.1 Skill Definition

Skills are stored as Markdown files with YAML frontmatter:

```yaml
---
name: github_workflow
description: Create and manage GitHub workflow files
autonomy_level: high
version: 1.0.0
tools:
  - file_read
  - file_write
  - terminal
  - remember
triggers:
  - "create workflow"
  - "new github action"
---
# GitHub Workflow Skill

## Purpose
Automate creation of GitHub Actions workflows.

## Workflow
1. Understand user's workflow requirements
2. Recall similar workflows from memory
3. Generate appropriate YAML
4. Write to .github/workflows/
5. Test syntax validity
6. Explain usage to user

## Self-Improvement
- Learn common workflow patterns
- Optimize generated YAML size
- Remember which templates work best
```

### 7.2 Autonomous Skill Creation

When Ao observes a pattern repeatedly:
1. **Detect Pattern**: Same tool sequence observed 3+ times
2. **Propose Skill**: Create draft skill definition
3. **Autonomous Approval**: If autonomy_level permits, create immediately
4. **Register**: Add to skill registry in SQLite
5. **Announce**: Notify other agents of new capability

### 7.3 Self-Improvement During Use

After skill execution:
1. **Analyze Outcome**: Did it succeed? Was it efficient?
2. **Extract Learnings**: What worked? What didn't?
3. **Modify Definition**: Update skill markdown with improvements
4. **Version Bump**: Increment skill version

---

## 8. Parallel Worker System

### 8.1 Architecture

```
        ┌─────────────────────────────────────┐
        │           Task Queue                │
        │      (async channel)                │
        └──────────────┬──────────────────────┘
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
   ┌─────────┐   ┌─────────┐   ┌─────────┐
   │Worker 1 │   │Worker 2 │   │Worker 3 │
   └────┬────┘   └────┬────┘   └────┬────┘
        │              │              │
        └──────────────┼──────────────┘
                       ▼
              ┌──────────────┐
              │   Results    │
              │   Channel   │
              └──────────────┘
```

### 8.2 Worker Configuration

```yaml
# .ao/config.yaml
workers:
  default_count: 2
  max_count: 5
  task_timeout: 300  # seconds
```

### 8.3 Task Types

| Type | Description | Example |
|------|-------------|---------|
| `tool` | Execute a single tool | Web search |
| `skill` | Run a skill workflow | Research |
| `worker` | Spawn worker agent for parallel execution | Parallel research |
| `schedule` | Run scheduled job | Memory cleanup |

### 8.4 Agent Spawning

When Ao spawns a worker agent, it is initialized with:

| Component | Description |
|-----------|-------------|
| **Purpose** | Task description from Ao |
| **Tools** | Relevant to the task (Ao selects appropriate tools) |
| **Skills** | Relevant to the task (Ao selects appropriate skills) |
| **Memory** | Fresh working memory (can access shared context if needed) |
| **Identity** | Lightweight identity: "You are a worker agent helping Ao" |

**Example**: For a parallel research task, Ao might spawn workers with:
- Tools: `web_search`, `remember`, `recall`
- Skills: `research`, `summarize`

```rust
// Spawn workers with task-specific tools and skills
ao.spawn_worker("researcher", "Find information about X")
    .with_tools(&["web_search", "remember"])
    .with_skills(&["research"])
    .spawn();

ao.spawn_worker("researcher", "Find information about Y")
    .with_tools(&["web_search", "remember"])
    .with_skills(&["research"])
    .spawn();

let results = ao.await_all_workers();
```

---

## 9. Scheduler

The Scheduler subsystem implements cron jobs as first-class agent tasks. See **Section 3.8** for detailed design including job types, human-readable parsing, execution model, and persistence.

### Quick Reference

| Component | Description |
|-----------|-------------|
| Job Types | Agent tasks, skill execution, tool execution, maintenance |
| Schedule Format | Human-readable ("every hour", "on mondays at 18:00") |
| Persistence | SQLite with execution history |
| Execution | Integrated with agent loop, not shell scripts |

---

## 10. Database Schema

### 10.1 Core Tables

```sql
-- Agents table
CREATE TABLE agents (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    purpose TEXT NOT NULL,
    instruction_file TEXT NOT NULL,
    autonomy_level TEXT DEFAULT 'high',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status TEXT DEFAULT 'active'
);

-- Agent tools (which tools each agent can use)
CREATE TABLE agent_tools (
    agent_id INTEGER NOT NULL,
    tool_id INTEGER NOT NULL,
    PRIMARY KEY (agent_id, tool_id),
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

-- Tools registry
CREATE TABLE tools (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    tool_type TEXT NOT NULL,
    risk_level TEXT NOT NULL DEFAULT 'read',
    scope TEXT NOT NULL DEFAULT 'public',
    command TEXT,
    parameters TEXT DEFAULT '[]',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Cosmic laws (governance rules)
CREATE TABLE laws (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    scope TEXT DEFAULT 'pantheon',
    priority INTEGER DEFAULT 5,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Skills registry
CREATE TABLE skills (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    definition TEXT NOT NULL,
    autonomy_level TEXT DEFAULT 'high',
    version TEXT DEFAULT '1.0.0',
    created_by TEXT,
    usage_count INTEGER DEFAULT 0,
    last_improved TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Agent memory
CREATE TABLE agent_memory (
    id INTEGER PRIMARY KEY,
    agent_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    importance INTEGER DEFAULT 5,
    tags TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    accessed_at TIMESTAMP,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

-- FTS5 index for memory search
CREATE VIRTUAL TABLE agent_memory_fts USING fts5(
    content,
    content=agent_memory,
    content_rowid=id
);

-- Session messages
CREATE TABLE messages (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    metadata TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Scheduler jobs
CREATE TABLE scheduler_jobs (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    schedule TEXT NOT NULL,
    task_type TEXT NOT NULL,
    task_data TEXT,
    enabled INTEGER DEFAULT 1,
    last_run TIMESTAMP,
    next_run TIMESTAMP
);
```

### 10.2 Indexes

```sql
CREATE INDEX idx_agent_memory_agent ON agent_memory(agent_id);
CREATE INDEX idx_agent_memory_importance ON agent_memory(importance DESC);
CREATE INDEX idx_messages_session ON messages(session_id, created_at);
CREATE INDEX idx_scheduler_next ON scheduler_jobs(next_run) WHERE enabled = 1;
```

---

## 11. CLI Reference

### 11.1 Core Commands

```bash
# Create new pantheon
ao create <name>

# Chat with Ao (starts TUI)
ao chat
ao chat --session <name>

# Test Ao responsiveness
ao test "message"

# Show status
ao status
```

### 11.2 Worker Commands

```bash
# Spawn worker for parallel execution
ao spawn <type> "task description"

# Example: spawn research workers for parallel search
ao spawn researcher "Find info about topic X"
ao spawn researcher "Find info about topic Y"
```

### 11.3 Tool Commands

```bash
# List tools
ao tools list

# Show tool details
ao tools show <name>

# Add user tool
ao tools add <path> --name <name> --risk <level>

# Remove tool
ao tools remove <name>
```

### 11.4 Skill Commands

```bash
# List skills
ao skills list

# Show skill details
ao skills show <name>

# Create skill (from file)
ao skills create <file.md>

# Enable/disable skill
ao skills enable <name>
ao skills disable <name>
```

### 11.5 Law Commands

```bash
# List laws
ao laws list

# Add law
ao laws add "rule" --scope <scope> --priority <n>

# Remove law
ao laws remove <id>
```

### 11.6 Scheduler Commands

```bash
# List scheduled jobs
ao schedule list

# Add job
ao schedule add <name> "every hour" --task <task>

# Enable/disable job
ao schedule enable <name>
ao schedule disable <name>

# Run job now
ao schedule run <name>
```

### 11.7 Memory Commands

```bash
# Search memory
ao memory search <query>

# Show agent memory
ao memory show <agent>

# Clear memory
ao memory clear <agent> --older-than <days>
```

---

## 12. Security & Governance

### 12.1 Risk Management

All tools have associated risk levels. Execution flow:

```
Tool Request → Risk Check → [If critical: User Prompt] → Execute
```

**Confirmation Prompt**:
```
⚠️ Critical Action Requested

Agent: <name>
Tool: <tool>
Action: <description>

Allow? (yes/no/cancel)
```

### 12.2 Autonomy Levels

| Level | Description |
|-------|-------------|
| `full` | Act without any prompting |
| `high` | Confirm only critical actions |
| `medium` | Confirm execute-level actions |
| `low` | Confirm all non-read actions |
| `none` | Ask for everything |

### 12.3 Scope Enforcement

Tools are scoped to prevent unauthorized access:
- `public`: Any agent
- `ao_only`: Ao exclusively
- `agent:<name>`: Specific agent only

---

## 13. Implementation Notes

### 13.1 Key Design Decisions

1. **SQLite over others**: Zero-config, single-file, excellent FTS5 support
2. **Hardcoded core tools**: Reliability; extension via `.ao/tools/`
3. **Markdown + YAML for skills**: Human-readable, version-controllable
4. **Human-readable scheduler**: Lower barrier to automation
5. **Moderate memory pruning**: Balance between preservation and size

### 13.2 Tradeoffs

| Decision | Benefit | Tradeoff |
|----------|---------|----------|
| SQLite | Simple, portable | Not for heavy concurrent writes |
| Hardcoded tools | Reliable, fast | Less flexible than plugins |
| FTS5 | Good search | Basic semantic search |
| Full autonomy | Unmatched capability | Potential for runaway behavior |

### 13.3 Graceful Degradation

The system works at multiple capability levels:

| Mode | LLM | Tools | Memory | Skills |
|------|-----|-------|--------|--------|
| Minimal | ✗ | Core only | ✗ | ✗ |
| Basic | ✗ | All | ✗ | ✗ |
| Standard | ✓ | All | ✗ | ✗ |
| Full | ✓ | All | ✓ | ✓ |

---

## 14. Future Directions

### 14.1 Near-term

- Enhanced FTS5 with embeddings for semantic search
- Plugin system for external tool discovery
- Multi-pantheon coordination
- Cloud sync via LibSQL (Turso)

### 14.2 Long-term

- Web interface alongside TUI
- Multi-user support
- Distributed agent execution
- Marketplace for skills

### 14.3 Extensibility Points

- **Tool Discovery**: Auto-scan `.ao/tools/` on startup
- **Skill Marketplace**: Share skills across pantheons
- **Memory Export/Import**: Backup and restore
- **Custom Renderers**: Plugin for TUI customization

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| **Ao** | The Meta-Architect, the only permanent entity |
| **Cosmic Law** | A rule governing system behavior |
| **Pantheon** | The complete agent system managed by Ao |
| **Skill** | A complex, self-improving workflow |
| **Tool** | An atomic, executable function |
| **Worker** | An ephemeral agent spawned by Ao for a task |
| **Memory** | Persistent storage of experiences |

---

## Appendix B: File Reference

| Path | Description |
|------|-------------|
| `.ao/ao.md` | Ao's instruction file |
| `.ao/skills/<skill>.md` | Skill definitions |
| `.ao/tools/<tool>` | User tool executables |
| `.ao/memory/pantheon.db` | SQLite database |
| `.ao/scheduler.yaml` | Scheduled jobs |
| `.ao/config.yaml` | Configuration |

---

## Appendix C: Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0-rc.1 | 2026-03-18 | Initial draft |

---

*This document describes the intended design of the Pantheon framework. Implementation details may vary.*
