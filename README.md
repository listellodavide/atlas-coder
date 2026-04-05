# AI Coding Assistant CLI — Rust

A high-performance, memory-safe AI coding assistant built in Rust. Provides an interactive REPL and one-shot prompt interface backed by multiple LLM providers, a full tool execution engine, session persistence, MCP server management, and an autonomous multi-agent dev pipeline.

---

## Quick Start

```bash
cd rust/
cargo build --release

# Interactive REPL
./target/release/atlas

# One-shot prompt
./target/release/atlas prompt "explain this codebase"

# With a specific model
./target/release/atlas --model sonnet prompt "fix the bug in main.rs"
```

---

## Features

### 🤖 LLM Providers

- **Anthropic** — full API + real-time SSE streaming, API key and OAuth login
- **Ollama** — local model support via the OpenAI-compatible endpoint (`http://localhost:11434/v1`), switch instantly with `/ollama <model>`
- **OpenAI-compatible** — any provider speaking the `/v1/chat/completions` protocol (OpenAI, xAI, etc.)
- **Model aliases** — short names that resolve to the latest version automatically

| Alias | Resolves To |
|-------|-------------|
| `opus` | `claude-opus-4-6` |
| `sonnet` | `claude-sonnet-4-6` |
| `haiku` | `claude-haiku-4-5-20251213` |

---

### 🛠️ Tool Execution Engine

Full built-in tool suite — the agent can use all of these autonomously:

| Tool | Description |
|------|-------------|
| `Bash` | Execute shell commands with configurable permission gates |
| `ReadFile` | Read specific files or line ranges |
| `WriteFile` | Create or overwrite files |
| `EditFile` | Targeted find-and-replace edits |
| `GlobSearch` | Pattern-match files across the workspace |
| `GrepSearch` | Regex / literal content search |
| `WebSearch` | Live internet search |
| `WebFetch` | Fetch and extract content from any URL |
| `Agent` | Spawn sub-agents for isolated task execution |
| `TodoWrite` | Persistent task tracking during long runs |
| `NotebookEdit` | Jupyter notebook cell editing |
| `Skill` | Load and invoke custom skill definitions |
| `ToolSearch` | Semantic search over available tools |

---

### 💬 Interactive REPL

- **Persistent multi-turn conversation** with automatic context tracking
- **Tab completion** — slash commands, model names, permission modes, recent session IDs
- **Streaming output** with live ANSI markdown rendering (headings, code blocks, bold, italic)
- **Syntax highlighting** via `syntect`
- **Vim keybinding mode** toggle
- **Session auto-save** — every turn persisted to disk

---

### 🗂️ Session Management

- Sessions saved to `~/.claude/sessions/` as versioned JSON
- `/session list` — browse all sessions
- `/session switch <id>` — load any previous session in-place
- `/session fork [branch]` — fork the current conversation into a new branch
- `/resume <id>` — resume by ID, path, or short alias (`latest`)
- `/restore <id>` — resume and restore the model used at save time
- `/compact` — summarise and compress conversation history to free context space
- `/clear` — start a fresh session (old session remains resumable)
- `/export [path]` — export current conversation to a Markdown file

---

### 🔐 Permission System

Three permission levels with runtime switching via `/permissions`:

| Mode | Description |
|------|-------------|
| `read-only` | No file writes, no shell execution |
| `workspace-write` | Full file access, no arbitrary shell |
| `danger-full-access` | Unrestricted — bash, writes, network |

- Permission prompts on tool calls with approve/deny flow
- `/approve` / `/deny` slash commands for prompted tool gates

---

### 🔑 Authentication

- **API key** — `ANTHROPIC_API_KEY` environment variable
- **OAuth** — full PKCE device-flow login via `/login`, stored credentials, refresh handling
- `/logout` — clear stored credentials
- Provider-specific env vars: `OPENAI_API_KEY`, `XAI_API_KEY`, `OLLAMA_API_KEY`, `OLLAMA_BASE_URL`

---

### ⚙️ Configuration System

Config file hierarchy — each layer merges cleanly:

1. System-wide `/etc/claude/config.json`
2. User global `~/.claude/config.json`
3. Project `.claude.json`
4. Environment variables

- `/config [env|hooks|model|plugins]` — inspect any config section live
- `CLAUDE.md` project memory — automatically injected into system prompt
- `/memory` — show which instruction files are currently loaded

---

### 🔌 MCP (Model Context Protocol) Servers

- Full MCP server lifecycle — start, connect, health-watch, restart
- Stdio and SSE transport
- OAuth-authenticated MCP connections
- `/mcp [list|show <server>|help]` — inspect configured servers
- Tool bridge — MCP tools appear as first-class agent tools

---

### 🧩 Plugin System

- Install, enable, disable, update, uninstall plugins
- Plugin-contributed tools automatically available in the agent
- `/plugin [list|install <path>|enable|disable|uninstall|update]`

---

### 🎓 Skills Registry

- Named, reusable prompt/tool bundles stored per-project or globally
- `/skills [list|install <path>|help]`

---

### 🧵 Autonomous Agents

#### `/rlm <task>` — Autonomous Loop Mode

Runs up to 30 plan → execute → verify iterations autonomously until the task is marked complete or the limit is reached.

#### `/squad <task>` — 5-Agent Dev Pipeline (via Ollama)

A full local multi-agent pipeline inspired by the `the-dev-squad` orchestrator. Requires a local Ollama model to be active (set with `/ollama <model>`).

**Pipeline phases:**

```
Phase 1 — Planning   Agent A researches the task and writes plan.md
Phase 2 — Review     Agent B audits; A ↔ B loop until plan is approved
Phase 3 — Coding     Agent C implements the approved plan exactly
Phase 4 — Testing    Agent D reviews code; C ↔ D loop until all pass
```

- All agent communication is **in-memory** (no polling, no subprocesses)
- Approved `plan.md` written to disk after Phase 2 for observability
- **Full activity log** → `dev-squad.log` in the working directory (timestamped, per-agent, per-phase)
- Up to 5 review rounds (A↔B) and 5 fix rounds (C↔D) with automatic stall recovery

---

### 🧠 Mematlas — Codebase Intelligence

- Index multi-repo, multi-module codebases for LLM-assisted development
- Structural + semantic knowledge graph over the workspace
- `/mematlas [learn|...]` — trigger autonomous codebase indexing

---

### 📊 Cost & Usage Tracking

- Per-turn and cumulative token usage
- Cost estimates with model-specific pricing
- Cache token breakdown (creation vs. read)
- `/cost` — show full cost breakdown
- `/usage` — detailed API usage statistics
- `/tokens` — current conversation token count
- `/cache` — prompt cache statistics

---

### 🪝 Hooks

Lifecycle hooks for tool call interception:

| Hook | Fires When |
|------|-----------|
| `PreToolUse` | Before any tool executes |
| `PostToolUse` | After any tool executes |

Hook scripts are configured per-project in `.claude/hooks/`.

---

### 🔀 Git Integration

- `/diff` — show current workspace git diff
- `/commit` — generate a commit message and create the commit
- `/pr [context]` — draft or open a pull request
- `/issue [context]` — draft or open a GitHub issue
- `/branch [name]` — create or switch branches
- `/stash [pop|list|apply]` — stash management
- `/changelog [count]` — show recent commit history

---

### 🔬 Analysis & Planning

- `/ultraplan <task>` — deep multi-step reasoning and plan generation
- `/bughunter [scope]` — autonomous bug detection across the codebase
- `/review [scope]` — AI code review on current changes
- `/security-review [scope]` — dedicated security audit pass
- `/teleport <symbol>` — jump to any file or symbol by semantic search

---

### 🎨 Display & Output

- Rich ANSI terminal rendering with adaptive color themes
- `/theme [name]` — switch terminal color theme
- `/color [scheme]` — configure color output
- `/output-style [style]` — switch between markdown, plain, json
- `/brief` — toggle concise response mode
- `/fast` — toggle speed-optimized response mode
- `/effort [low|medium|high]` — control reasoning depth

---

## CLI Flags

```
atlas [OPTIONS] [COMMAND]

Options:
  --model MODEL                    Set the active model (alias or full name)
  --dangerously-skip-permissions   Skip all permission prompts
  --permission-mode MODE           read-only | workspace-write | danger-full-access
  --allowedTools TOOLS             Comma-separated list of permitted tools
  --output-format FORMAT           text | json
  --version, -V                    Print version and build info

Commands:
  prompt <text>    One-shot non-interactive prompt
  login            OAuth authentication
  logout           Clear stored credentials
  init             Create starter CLAUDE.md for this project
  doctor           Diagnose environment and config issues
  self-update      Update to the latest release
```

---

## Slash Commands Reference

| Command | Description |
|---------|-------------|
| `/help` | Show all available slash commands |
| `/status` | Session status — model, tokens, cost, branch |
| `/model [name]` | Show or switch the active model |
| `/ollama <model>` | Switch to a local Ollama model |
| `/permissions [mode]` | Show or change the permission mode |
| `/sandbox` | Show current sandbox isolation status |
| `/cost` | Cumulative cost for this session |
| `/tokens` | Token count for the current conversation |
| `/usage` | Detailed API usage breakdown |
| `/cache` | Prompt cache statistics |
| `/compact` | Compress conversation history |
| `/clear [--confirm]` | Start a fresh session |
| `/session [list\|switch\|fork]` | Manage saved sessions |
| `/resume <id>` | Load a previous session |
| `/export [path]` | Export conversation to file |
| `/config [section]` | Inspect merged config |
| `/memory` | Show loaded CLAUDE.md files |
| `/mcp [list\|show\|help]` | MCP server inspection |
| `/plugin [...]` | Plugin management |
| `/skills [list\|install\|help]` | Skills registry |
| `/mematlas [action]` | Codebase intelligence |
| `/diff` | Git diff of current changes |
| `/commit` | Generate commit message + commit |
| `/pr [context]` | Draft or create a pull request |
| `/issue [context]` | Draft or create a GitHub issue |
| `/branch [name]` | Create or switch git branches |
| `/stash [pop\|list\|apply]` | Stash management |
| `/rlm <task>` | Autonomous 30-iteration agent loop |
| `/squad <task>` | 5-agent local dev pipeline via Ollama |
| `/ultraplan <task>` | Deep planning with multi-step reasoning |
| `/bughunter [scope]` | Autonomous bug inspection |
| `/review [scope]` | Code review pass |
| `/security-review [scope]` | Security audit pass |
| `/plan [on\|off]` | Toggle planning mode |
| `/teleport <symbol>` | Jump to file or symbol |
| `/search <query>` | Workspace file search |
| `/init` | Create CLAUDE.md for the project |
| `/doctor` | Environment health check |
| `/agents [list\|help]` | List configured agents |
| `/hooks [list\|run]` | Manage lifecycle hooks |
| `/theme [name]` | Terminal color theme |
| `/effort [low\|medium\|high]` | Reasoning depth |
| `/fast` | Toggle fast response mode |
| `/brief` | Toggle brief output mode |
| `/vim` | Toggle vim keybindings |
| `/version` | CLI version and build info |
| `/exit` | Exit the REPL |

---

## Workspace Layout

```
rust/
├── Cargo.toml
├── Cargo.lock
└── crates/
    ├── api/                    # HTTP client, SSE streaming, request/response types, auth
    ├── commands/               # Slash command registry, parser, help renderer (145 commands)
    ├── compat-harness/         # Tool/prompt manifest extraction harness
    ├── mematlas/                # Codebase intelligence indexer and retrieval
    ├── mock-anthropic-service/ # Deterministic local Anthropic-compatible mock for testing
    ├── plugins/                # Plugin lifecycle, registry, hook dispatch
    ├── runtime/                # Conversation loop, config loader, session, permissions, MCP, hooks
    ├── rusty-claude-cli/       # CLI binary: REPL, streaming display, argument parsing
    ├── squad/                  # 5-agent Ollama pipeline (roles, agent, orchestrator, events)
    ├── telemetry/              # Analytics, session tracing, cost tracking
    └── tools/                  # Tool implementations: Bash, ReadFile, WriteFile, Edit, Grep, Glob, Web, Agent, Todo...
```

---

## Testing

```bash
# Full test suite
cd rust/
cargo test

# Mock parity harness (deterministic end-to-end CLI tests)
./scripts/run_mock_parity_harness.sh

# Run mock service manually
cargo run -p mock-anthropic-service -- --bind 127.0.0.1:0
```

**Parity harness scenarios:** `streaming_text`, `read_file_roundtrip`, `grep_chunk_assembly`, `write_file_allowed`, `write_file_denied`, `multi_tool_turn_roundtrip`, `bash_stdout_roundtrip`, `bash_permission_prompt_approved`, `bash_permission_prompt_denied`, `plugin_tool_roundtrip`

---

## Stats

| | |
|--|--|
| Language | Rust (2021 edition) |
| Crates in workspace | 11 |
| Slash commands | 145 |
| Binary name | `atlas` |
| Default model | `claude-opus-4-6` |
| Default permission mode | `danger-full-access` |

---

## License

MIT
