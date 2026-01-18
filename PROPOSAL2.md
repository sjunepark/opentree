# Proposal: Improved Codex Integration

## Problem Statement

### Current Approach

The runner integrates with Codex via CLI subprocess (`runner/src/io/executor.rs:61-77`):

```rust
let mut cmd = Command::new("codex");
cmd.arg("exec")
    .arg("--output-schema").arg(&request.output_schema_path)
    .arg("--output-last-message").arg(&request.output_path)
    .arg("-")
    .stdin(Stdio::piped());
```

This works but has limitations:

1. **No streaming** - We wait for full completion, no progress visibility
2. **Coarse error handling** - Only exit code + stderr; no structured error types
3. **Limited control** - Can't interrupt, pause, or observe intermediate state
4. **Schema-only output** - Must parse JSON from file; no event stream

### Why Revisit Now

- Codex now offers richer integration surfaces (investigated via btca query)
- Future features may need streaming progress, cancellation, or multi-turn conversations
- Understanding options now prevents costly rewrites later

## Investigation Findings

Queried the [openai/codex](https://github.com/openai/codex) repository. Three official integration patterns exist:

### 1. `codex app-server` (JSON-RPC over stdio)

**What it is**: The backend for rich UIs like the VS Code extension.

**Protocol**:

- Bidirectional JSON-RPC 2.0 over stdin/stdout
- Line-delimited JSONL framing
- Streaming notifications: `item/*`, `turn/completed`, etc.

**Lifecycle**:

1. Spawn `codex app-server`
2. Send `initialize` request
3. Send `initialized` notification
4. Use `thread/start`, `turn/start` for conversations
5. Consume streaming events

**Schema generation**:

```bash
codex app-server generate-json-schema --out ./schemas/
codex app-server generate-ts --out ./types/
```

**Pros**:

- Richest control surface
- Streaming progress events
- Proper request/response correlation
- Type generation for client code

**Cons**:

- More complex to implement
- Must manage connection lifecycle
- Still subprocess-based (not in-process)

### 2. `codex mcp-server` (MCP protocol)

**What it is**: Codex as an MCP tool server for other agents.

**Protocol**:

- Standard MCP over stdio (JSON-RPC 2.0, line-delimited)
- Methods: `newConversation`, `sendUserMessage`, `interruptConversation`

**Usage**:

```bash
codex mcp-server | your_mcp_client
```

**Pros**:

- Standard MCP protocol
- Good if already using MCP elsewhere
- Clean tool-based abstraction

**Cons**:

- Designed for "Codex as a tool" pattern
- May be overkill for direct orchestration

### 3. `@openai/codex-sdk` (Node.js)

**What it is**: npm package wrapping `codex exec --experimental-json`.

**Mechanism**: Spawns CLI, exchanges JSONL over stdio.

**Pros**:

- Simple for Node.js projects
- Official SDK

**Cons**:

- Node.js only (we're Rust)
- Still subprocess-based underneath

### What's NOT Available

- **No in-process Rust library** - `codex-rs/core` is described as "hoped to be a library crate" in the future; not yet a stable embedding surface.

## Options

### Option A: Keep Current Approach

**Change**: None.

**When appropriate**:

- Current features sufficient
- No need for streaming/interruption
- Minimize complexity

**Risk**: May need rewrite if requirements change.

### Option B: Migrate to `app-server`

**Change**: Replace `codex exec` with `codex app-server` JSON-RPC.

**Implementation sketch**:

```rust
pub struct AppServerExecutor {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    request_id: AtomicU64,
}

impl AppServerExecutor {
    pub fn spawn() -> Result<Self> { /* spawn + initialize handshake */ }
    pub fn start_turn(&mut self, prompt: &str) -> Result<TurnHandle> { /* ... */ }
}

pub struct TurnHandle { /* streams events until turn/completed */ }
```

**Benefits**:

- Streaming progress (can log/display as agent works)
- Structured events instead of file parsing
- Interruption support
- Request correlation (parallel turns theoretically possible)

**Costs**:

- Implement JSON-RPC client
- Manage long-lived connection
- Handle reconnection on crash
- More complex error paths

### Option C: Migrate to `mcp-server`

**Change**: Use Codex as MCP tool.

**When appropriate**:

- Runner itself becomes an MCP client
- Want uniform tool abstraction across multiple agents

**Not recommended now**: Adds MCP client complexity without clear benefit.

## Recommendation

**Short term**: Stay with Option A (current `codex exec`).

**Reasoning**:

- Current approach works and is simple
- No immediate need for streaming or interruption
- `app-server` adds complexity we don't yet need

**Prepare for Option B**: If any of these become true, migrate to `app-server`:

- Need progress streaming (long tasks, user feedback)
- Need cancellation (user interrupt, timeout with cleanup)
- Need multi-turn conversations within single agent session
- Need richer error diagnostics

**Action items** (optional, low priority):

1. Generate JSON-RPC schemas: `codex app-server generate-json-schema`
2. Prototype `AppServerExecutor` in a branch
3. Validate streaming works as documented

## Appendix: Current Integration Details

### Files Involved

| File | Role |
|------|------|
| `runner/src/io/executor.rs` | `Executor` trait + `CodexExecutor` impl |
| `runner/src/io/process.rs` | `run_command_with_timeout` helper |
| `runner/schemas/agent_output.schema.json` | Output schema for `--output-schema` |

### Current Flow

```text
CodexExecutor::exec()
    │
    ├─ Validate schema file exists
    ├─ Create output directory
    ├─ Spawn: codex exec --output-schema X --output-last-message Y -
    ├─ Write prompt to stdin
    ├─ Wait with timeout
    ├─ Write stdout/stderr to executor log
    └─ Check exit status

execute_and_load()
    │
    ├─ Call executor.exec()
    ├─ Read output JSON file
    └─ Parse into AgentOutput { status, summary }
```

### Trait Abstraction

The `Executor` trait already abstracts the backend:

```rust
pub trait Executor {
    fn exec(&self, request: &ExecRequest) -> Result<()>;
}
```

This means switching to `app-server` only requires a new impl—no changes to callers.
