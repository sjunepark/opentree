# OpenCode CLI Reference

Command-line interface for OpenCode agent. Source: [anomalyco/opencode](https://github.com/anomalyco/opencode).

## Core Commands

### `opencode run [message]`

Non-interactive execution. Primary command for automation/scripting.

| Flag | Short | Description |
|------|-------|-------------|
| `--format` | | `default` (human) or `json` (NDJSON events) |
| `--model` | `-m` | Model ID, e.g., `anthropic/claude-sonnet-4-20250514` |
| `--agent` | | Named agent from config |
| `--file` | `-f` | Attach file (repeatable) |
| `--attach` | | Connect to running server URL |
| `--session` | `-s` | Reuse session by ID |
| `--continue` | `-c` | Continue last session |
| `--command` | | Run configured command template |
| `--title` | | Session title |
| `--port` | | Local server port |
| `--share` | | Share session to opencode.ai |

```bash
# Basic usage
opencode run "Explain this function"

# JSON streaming
opencode run --format json "Summarize repo" > events.ndjson

# With model selection
opencode run -m anthropic/claude-sonnet-4-20250514 "Fix the bug"

# Pipe input
cat file.ts | opencode run "Review this code"
```

### `opencode tui [project]`

Interactive terminal UI. Default command when no subcommand given.

### `opencode serve`

Start headless HTTP server (API only, no UI).

### `opencode web`

Start HTTP server and open web UI in browser.

### `opencode acp`

Start Agent Client Protocol server. Communicates via stdin/stdout NDJSON.

| Flag | Description |
|------|-------------|
| `--cwd` | Working directory |
| `--port` | Server port |
| `--hostname` | Server hostname |
| `--mdns` | Enable mDNS discovery |
| `--cors` | Enable CORS |

## Session Management

### `opencode session list`

List sessions. Supports `--format table|json` and `--limit N`.

### `opencode export [sessionID]`

Export session as JSON to stdout.

### `opencode import <file|url>`

Import session from JSON file or opencode.ai share URL.

## Model & Provider

### `opencode models [provider]`

List available models, optionally filtered by provider.

### `opencode stats`

Show usage statistics (tokens, cost, tools) across sessions.

## Authentication

### `opencode auth login [url]`

Add provider credentials. Interactive provider selection or "well-known" URL flow.

### `opencode auth list` / `opencode auth ls`

List stored credentials and detected env vars.

### `opencode auth logout`

Remove stored credential (interactive selection).

## MCP Servers

### `opencode mcp add`

Add MCP server to config (interactive). Supports local commands or remote URLs.

### `opencode mcp list` / `opencode mcp ls`

List configured MCP servers and connection status.

### `opencode mcp auth [name]`

OAuth authenticate to remote MCP server.

### `opencode mcp auth list` / `opencode mcp auth ls`

List OAuth-capable MCP servers and auth status.

### `opencode mcp logout [name]`

Remove OAuth credentials for MCP server.

### `opencode mcp debug <name>`

Debug OAuth connection issues.

## Agent Management

### `opencode agent create`

Generate new agent file. Interactive or CLI-driven with flags.

### `opencode agent list`

List all available agents.

## GitHub Integration

### `opencode github install`

Install GitHub agent workflow into current repo.

### `opencode github run`

Run GitHub agent (typically in GitHub Actions).

| Flag | Description |
|------|-------------|
| `--event` | Mock GitHub event JSON (local testing) |
| `--token` | GitHub PAT (with mock mode) |

### `opencode pr <number>`

Checkout PR branch via `gh`, import session from PR body if present, launch TUI.

## Maintenance

### `opencode upgrade [target]`

Upgrade to latest or specific version.

### `opencode uninstall`

Uninstall OpenCode. Optionally remove config/data.

## Debugging

### `opencode debug config`

Print resolved configuration as JSON.

### `opencode debug paths`

Show global paths (data, config, cache, state).

### `opencode debug wait`

Sleep indefinitely (for attaching debuggers).

### `opencode debug lsp diagnostics <file>`

Get LSP diagnostics for file.

### `opencode debug lsp symbols <query>`

Search workspace symbols.

### `opencode debug lsp document-symbols <uri>`

Get symbols for document.

### `opencode debug rg tree`

Show file tree via ripgrep. `--limit N` flag.

### `opencode debug rg files`

List files. `--query`, `--glob`, `--limit` flags.

### `opencode debug rg search <pattern>`

Search files. `--glob`, `--limit` flags.

### `opencode debug agent <name>`

Inspect agent config. `--tool`, `--params` flags for tool execution.

### `opencode debug skill`

List available skills as JSON.

### `opencode debug scrap`

List known projects as JSON.

### `opencode debug snapshot track|patch|diff`

Snapshot debugging utilities.

### `opencode generate`

Emit OpenAPI spec JSON.

### `opencode completion`

Generate shell completion script.

## Global Flags

| Flag | Description |
|------|-------------|
| `--print-logs` | Print logs to stderr |
| `--log-level` | `DEBUG`, `INFO`, `WARN`, `ERROR` |
