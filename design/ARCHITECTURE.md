# Pantheon - System Architecture

**Purpose**: Describe how the features work together as a cohesive personal assistant using descriptive prose.

---

## Overview

Pantheon is designed as a personal AI assistant that remembers your preferences, learns from interactions, and delivers consistent help across sessions. The architecture centers on a single permanent agent called Ao, which handles all user interactions directly while spawning temporary workers when parallel execution provides benefits.

At its core, Pantheon works by receiving your input through a chat interface, assembling relevant context from your history and memories, executing the appropriate tools or skills, and returning results while recording what it learned. Every interaction builds on previous ones, creating a continuous experience where the system remembers what matters to you.

---

## The Library: What Makes It Your Assistant

The Library is the defining feature that makes Pantheon truly your personal assistant. It is your personal knowledge base, where you store your notes, documents, code, references, and everything you want the assistant to know about.

Unlike generic AI assistants that only know public information, Pantheon knows your files. When you ask questions, it searches your Library. When you reference "that document from last month" or "my notes on API design," it can find and use those files. This is what transforms Pantheon from a generic tool into your assistant that understands your context.

The Library lives in a `library/` directory in your pantheon folder. Within this directory, you organize your knowledge however you prefer. Notes go in `library/notes/`, documents in `library/documents/`, code snippets in `library/code/`, and so on. The system does not impose structure on you; you organize your knowledge the way that makes sense to you.

Every file in the Library gets indexed. The indexer reads your files, extracts searchable text, builds full-text search indexes, and stores metadata like titles, tags, and summaries. When you add or change a file, the index updates automatically.

When you interact with Pantheon, the system searches your Library alongside everything else. Your question triggers a search through your indexed knowledge. Relevant passages get extracted and injected into the prompt alongside your identity, memories, and conversation history. The language model responds with answers grounded in your personal files, citing where it found information.

This creates the experience of working with someone who has read all your files and can discuss them intelligently. You are not just talking to an AI; you are talking to an AI that knows your documents, your notes, and your work.

---

## The User Experience

When you interact with Pantheon, your experience follows a consistent pattern. You type a message in the chat interface. Behind the scenes, the system loads your session, assembles a prompt containing your identity, short-term context from recent sessions, available tools, and conversation history, then passes this to the agent loop for execution. The agent loop decides whether to handle the task directly by calling tools, or to spawn worker agents for parallel execution when appropriate.

When you want the assistant to know about specific things from your Library or past conversations, you use the `/remember` command to inject that context. This gives you control over what the assistant knows rather than having everything automatically injected. Results are returned to you, and optionally recorded in memory if you want to remember it for future sessions.

This creates the feeling of working with an assistant where you have control over context, remembering what matters when you explicitly ask it to remember, rather than dealing with an overwhelmed prompt.

---

## Core Components and Their Roles

### The Session Manager

The session manager handles the continuity of your experience. When you start Pantheon, it loads or creates a session that tracks everything about your current interaction period. Sessions preserve your conversation history, remember which memories were relevant at the start, and maintain state across multiple message exchanges.

Each session takes a snapshot of your memories and profile when it begins. This snapshot remains stable throughout the session, which allows the system's prompts to remain consistent even as new memories are formed during the session. When you return later, a new session begins with its own fresh snapshot, but the memories from previous sessions remain accessible through the recall system.

### The Prompt System

The prompt system is the heart of how Pantheon assembles context for its language model. Rather than sending only your current message, the system builds a comprehensive prompt containing multiple layers of information.

First comes the Soul block, which defines Ao's core essence and unchanging character. This comes from the SOUL.md file and is created once, never modified. It establishes the fundamental traits that define who Ao is.

Second comes the Identity block, which defines Ao's current behavior and guidelines. This comes from the IDENTITY.md file which is user-editable. Together, Soul and Identity define who Ao is and how it should behave in every interaction.

Next, the system adds previous session summaries for cross-session continuity. At session start, it loads brief summaries of recent conversations, giving the assistant awareness of ongoing topics without overwhelming the prompt.

Then the system adds conversation history from the current session. Messages accumulate during the session and are sent with each prompt to maintain context within the session.

**Current Prompt Order:**
```
=== Ao's Soul ===
{core essence}
=== End Soul ===

=== Ao's Identity ===
{behavior guidelines}
=== End Identity ===

=== Previous Sessions Summary ===
{session summaries}
=== End Previous Sessions ===

=== Current Session History ===
{conversation history}
=== End Current History ===

User: {input}
Ao:
```

The ordering matters for efficiency. Soul and Identity are stable and placed first so language model providers can cache those sections, reducing tokens sent on each request.

### The Agent Loop

The agent loop is the execution engine that drives every interaction. It receives the assembled prompt, sends it to a language model, processes the model's response, and manages the flow until a final answer is ready.

When the model requests tool execution, the agent loop validates the tool exists, checks whether the worker has permission to use it, evaluates the risk level, and if necessary prompts you for confirmation before proceeding. The loop handles multiple tools in sequence when they depend on each other, or in parallel when they are independent.

The agent loop also manages retries. If a language model provider fails or rate limits occur, the loop automatically tries the next provider in your configured chain. It uses exponential backoff to avoid overwhelming providers that are experiencing temporary issues.

### The Worker System

Workers are temporary agents spawned by Ao when parallel execution provides benefits. Unlike the traditional model where you might "delegate" tasks to persistent agents, Pantheon takes a simpler approach: workers exist only for the duration of their task and receive their identity, tools, and context through prompt injection.

When Ao determines a task would benefit from parallel execution, such as researching two different topics simultaneously, it constructs worker prompts by selecting relevant tools, choosing appropriate skills, pulling relevant memories, and embedding all of this in a prompt that tells the worker who it is and what to do. The worker then executes independently, and when finished, returns its result to Ao which aggregates the responses and presents them to you.

Workers draw from a configurable pool. By default, two workers are available, with a maximum of five. Each worker operates asynchronously, allowing true parallel execution when multiple workers are active.

### The Memory System

Memory is what makes Pantheon feel like it knows you over time. The system stores experiences, learnings, and preferences in SQLite with full-text search, enabling quick retrieval when you need them.

Memory comes in two forms. Short-term memory lives only during a session, containing conversation history and current working context. When you start a session, the system loads a brief summary of what you were last working on. This small amount of context gives the assistant immediate awareness without overwhelming the prompt. Long-term memory persists across sessions in SQLite with full-text indexing, storing experiences and preferences you want to remember.

The prompt system uses memory at session start, loading short-term memory for continuity. When you use `/remember` or the recall tool, it searches long-term memory and injects relevant entries into your current context. This explicit approach gives you control over what the assistant knows rather than automatically injecting everything.

The memory system also connects to other parts. When you create files in your Library, memory can note what you created. When Ao spawns workers, it can include relevant memories so workers have awareness of past experiences. The remember and recall tools let the language model store and retrieve memories during conversation, giving the assistant agency to record things it learns. In the background, the memory system periodically evaluates importance, consolidates entries, summarizes older memories, and prunes what is no longer needed. This keeps your memory store organized without flooding your prompts.

### The Identity System

Ao's identity is what makes it feel like a persistent entity rather than a generic LLM wrapper. The identity system provides two files that are loaded at startup and injected into every prompt.

**Soul (SOUL.md)** is Ao's core essence—the unchanging character traits that define who Ao is. Created once and never modified, it establishes fundamental traits like curiosity, patience, and respect for user autonomy. Soul is sacred and permanent; it should never be changed even with confirmation.

**Identity (IDENTITY.md)** defines Ao's current behavior and guidelines. This is user-editable, allowing you to customize how Ao approaches tasks, what context it has access to, and how it should respond in different situations. Since Identity is intentional rather than disposable, there is no in-app reset command—edit the file directly or use version control to manage changes.

Both files are auto-created with sensible defaults on first run if they don't exist. The `/soul` and `/identity` commands let you view the current content of each file. Changes to these files take effect on the next restart.

### The Library System

The Library is where your personal knowledge lives. It is organized as a directory structure that you control, containing notes, documents, code, and any other files you want Pantheon to know about.

The Library consists of several interacting parts. The indexer reads through your files, extracts text content, and builds searchable indexes. The search engine uses full-text search to find relevant passages quickly. The metadata store tracks information about each file including when it was indexed, what tags you have assigned, and brief summaries. The file watcher monitors your Library directory and automatically re-indexes files when they change.

When you use `/remember` or the recall tool, the system searches your Library. You specify what you are looking for, it extracts keywords, searches the full-text index, retrieves the most relevant passages, and injects them into your current context. The language model then responds using knowledge from your personal files. This is fundamentally different from generic assistants because the model can reference your specific notes, documents, and code.

The Library also integrates with memory. When you create new notes or documents, the system can remember that you created them and what they contain. Later, when you ask for recall, the system can pull both from your personal memories and your Library files, giving the assistant a comprehensive view of your context.

You have tools to interact with your Library directly as well. You can search manually with the library_search tool, trigger re-indexing when you add many files at once, view statistics about your Library, and add or update metadata. The primary experience is the explicit Library search you trigger when you need it.

---

## The Tool System

Tools are atomic functions that let the language model interact with the world. Pantheon ships with core tools hardcoded for reliability, including status checks, web search, terminal command execution, file reading and writing, browser automation, image analysis and generation, text to speech, code execution, and memory operations.

You can extend the system with your own tools by placing executable scripts in the tools directory. These scripts receive JSON input through standard input and produce JSON output through standard output, following a simple protocol that works with any programming language.

Every tool has an associated risk level. Read-level tools like searching or reading files proceed automatically. Write-level tools modify data but also proceed automatically. Execute and critical level tools, which run processes or make system changes, require your confirmation before proceeding.

### The Skills System

Skills are complex workflows that combine multiple steps, potentially involving tools, memory, conditional logic, and decision making. Skills are defined as markdown files with YAML frontmatter, making them human-readable and version-controllable.

When you trigger a skill, the skill engine loads its definition, executes each step in order, handles any conditional branching, and manages the flow until completion. Skills can improve themselves over time by tracking what works and adjusting their behavior based on feedback, though this operates at a level you can configure.

### The Scheduler

The scheduler runs automated tasks on defined schedules using human-readable syntax. Instead of writing cron expressions, you can say things like "every day at 9am" or "every hour" and the system parses this into the appropriate schedule.

Scheduled tasks can execute any tool, run any skill, spawn workers, or perform maintenance operations like cleaning up old sessions or rebuilding search indexes. The scheduler runs in the background, persisting job definitions and execution history to SQLite so it continues reliably across restarts.

---

## How the Pieces Connect

The components work together through well-defined contracts. The prompt system provides assembled prompts to the agent loop. The agent loop calls tools through the tool executor and spawns workers through the worker spawner. Workers read and write memories through the memory system. Session state persists through the session manager to SQLite. The scheduler triggers background tasks by calling into the agent loop.

This modularity means each component can be tested independently and replaced with different implementations. You could add a new language model provider, create new tools, or modify how memory consolidation works without affecting other parts of the system.

---

## The Flow of a Typical Interaction

Consider when you tell Pantheon to research a topic. Your message arrives at the session manager which loads your session and history. The prompt system assembles your identity, frozen memory snapshot, relevant skills, tool schemas, and conversation history. The agent loop sends this to your configured language model.

The model might decide to use the web search tool directly. The tool executor validates the tool, checks permissions and risk level, executes the search, and returns results. The agent loop incorporates these results and continues the conversation with the language model until it produces a final answer.

Alternatively, if the task is complex, Ao might spawn workers to search multiple sources in parallel. Each worker receives a tailored prompt with appropriate tools and context. When workers complete, Ao aggregates their results and presents a unified response.

In both cases, the memory system records what happened. If you mentioned preferences or important context, these get stored with appropriate importance scores. The next time you interact, recall can surface relevant memories, creating the continuity that makes the system feel like it genuinely knows you.

---

## Error Handling and Recovery

The system anticipates failures at multiple levels. Language model providers may become unavailable or rate limited. The agent loop handles this by trying providers in your configured fallback chain with exponential backoff between attempts.

Tools may fail to execute. The tool executor catches these errors, logs them appropriately, and returns failure information to the agent loop which can decide whether to retry, try an alternative approach, or report the problem to you.

Workers may time out or produce errors. The worker spawner catches these and either retries once or reports the failure, depending on the configuration.

These recovery mechanisms ensure the system remains resilient and continues providing value even when individual components encounter problems.

---

## Configuration and Extensibility

Everything about Pantheon's behavior is configurable. You can specify how many workers should be available, which language model providers to use and in what order to try them, how aggressively to consolidate and prune memories, what timezone to use for scheduled tasks, and how the interface should display information.

You extend the system by adding tools as scripts in the tools directory, creating skills as markdown files with YAML frontmatter, writing context files like .cursorrules that get loaded into prompts, and defining scheduled tasks for automation.

---

## Summary

Pantheon achieves its goal of being a personal assistant that remembers, acts, and delivers consistent experience through the interaction of several key systems. The Library holds your personal knowledge and is searched when you explicitly ask via `/remember`. The session manager provides continuity and short-term context. The identity system (Soul and Identity) defines who Ao is and how it behaves in every interaction. The prompt system assembles context efficiently with user-controlled injection. The agent loop executes tasks and manages the flow. Workers provide parallel execution capacity when beneficial. The memory system stores experiences and enables recall when you request it. Tools and skills provide capabilities. The scheduler handles automation.

Together, these components create an experience where you control what context is in play, the system knows your files when you ask, remembers interactions you want preserved, and provides helpful assistance that is genuinely personal to you.
