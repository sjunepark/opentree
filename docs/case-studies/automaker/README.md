# Automaker Case Study

Automaker is an "autonomous AI development studio" - a desktop/web app for managing feature cards (Kanban-style) where AI agents implement features in your codebase.

## Key Insights

### 1. Provider Abstraction Pattern

**Problem**: Support multiple model backends (Claude SDK, Codex CLI, Cursor, OpenCode) with a unified interface.

**Solution**: Provider interface with routing factory.

```text
ExecuteOptions → ProviderFactory → Provider.executeQuery() → AsyncGenerator<ProviderMessage>
```

Key design choices:

- **Bare model IDs**: Routing prefixes (`codex-`, `cursor-`) stripped before provider execution
- **Unified message format**: All providers emit `ProviderMessage` with `ContentBlock[]` (text, thinking, tool_use, tool_result)
- **Registry pattern**: `canHandleModel()` + priority ordering for routing decisions
- **CLI adapters normalize events**: Codex/Cursor CLI JSONL events are transformed to match SDK message shapes

**Takeaway**: Abstract at the message/event level, not the API level. Let providers handle protocol differences internally.

### 2. State Machine for Feature Execution

**Problem**: Features move through many states with interruption/resume needs.

**Solution**: File-based persistence + in-memory tracking.

States: `pending` → `in_progress` → `pipeline_<step>` → `verified`/`waiting_approval` → `committed`

Key design choices:

- **In-memory map guards concurrency**: `runningFeatures.has(featureId)` check-then-add before any async work
- **File-based persistence**: `.automaker/features/<id>/feature.json` survives restarts
- **Context file enables resume**: `agent-output.md` existence determines resume vs restart
- **Explicit timestamp for UI hints**: `justFinishedAt` for short-lived "just completed" badges

**Takeaway**: Separate "is running" (memory) from "state/progress" (disk). Use context files as resume eligibility markers.

### 3. Event Streaming Architecture

**Problem**: Real-time UI updates for agent progress, tool use, terminal output.

**Solution**: Single broadcast WebSocket + scoped filtering in UI.

```text
Service.emit(type, payload) → EventBus → WebSocket fanout → UI filters by sessionId/projectPath
```

Key design choices:

- **No server-side routing**: All events broadcast; UI handles scoping
- **Typed event channels**: `agent:stream`, `auto-mode:event`, `feature:*`, etc.
- **Nested discriminator pattern**: `{ type: "auto-mode:event", payload: { type: "feature_start", ... } }`
- **Separate terminal channel**: PTY streaming via dedicated WebSocket endpoint

**Takeaway**: Broadcast + client filter is simpler than server-side pub/sub when client count is low.

### 4. Tool System Design

**Problem**: Expose different tool sets to different execution contexts.

**Solution**: Configurable tool presets + provider-specific execution.

Key design choices:

- **Tool availability via allowedTools[]**: Not hardcoded; passed per execution
- **Presets in one place**: `TOOL_PRESETS` defines read-only vs full access sets
- **Tools as content blocks**: `tool_use` and `tool_result` are first-class message content
- **MCP integration**: External tool servers configurable, written to provider config files

**Takeaway**: Tool availability should be a runtime configuration, not compiled-in.

### 5. Context/Prompt Building

**Problem**: Build system prompts from multiple sources (base prompt, context files, memory).

**Solution**: Layered assembly with smart deduplication.

Layers:

1. Project context files (`.automaker/context/*.md`)
2. Memory files (`.automaker/memory/*.md` with relevance scoring)
3. Base system prompt from settings

Key design choices:

- **Relevance-based memory selection**: YAML frontmatter (tags, importance) + task keyword matching
- **Always include gotchas.md**: Special handling for common pitfalls
- **CLAUDE.md deduplication**: When SDK auto-loads it, remove from manual context
- **Hot-reloadable prompts**: Settings loaded fresh each run

**Takeaway**: Memory should be relevance-scored, not just dumped. Deduplicate across loading mechanisms.

### 6. Planning Mode Architecture

**Problem**: Support different planning depths (none, lite, detailed spec) with optional approval gates.

**Solution**: Marker-based detection + structured parsing.

Modes: `skip` | `lite` | `spec` | `full`

Key design choices:

- **Markers in output**: `[SPEC_GENERATED]` triggers plan detection
- **Structured task format**: `- [ ] T001: Do thing | File: path/to/file`
- **Approval timeout**: 30-minute timeout prevents hanging state
- **Server restart recovery**: Approval can be resolved based on persisted `planSpec.status`
- **Post-approval multi-agent**: Each parsed task gets its own focused agent call

**Takeaway**: Use explicit markers for state transitions. Plan approval is feature state, not just in-memory promise.

### 7. Worktree Isolation Pattern

**Problem**: Isolate feature work without polluting main branch.

**Solution**: Git worktrees as alternate working directories.

Key design choices:

- **Branch-based lookup**: Parse `git worktree list --porcelain` to find worktree by branch
- **Metadata stays in main project**: `.automaker/` always under project root, not worktree
- **Creation in routes, not service**: Worktree creation is API concern, execution assumes it exists

**Takeaway**: Use worktrees for isolation but keep metadata/state centralized.

### 8. Error Recovery Patterns

**Problem**: Handle failures gracefully without losing progress.

**Solution**: Resume-first approach with failure tracking.

Key design choices:

- **Resume over retry**: Check for context file → resume if exists, else restart
- **Failure classification**: Track error types (quota, rate limit, general)
- **Auto-pause on repeated failures**: 3 failures in 60s or quota exhausted → pause auto-loop
- **Status demotion on failure**: Non-abort failures → `backlog` status

**Takeaway**: Resume is safer than retry; it preserves context. Track failure patterns to prevent runaway loops.

### 9. Session Continuity

**Problem**: Maintain conversation state across restarts and provider calls.

**Solution**: Dual-layer persistence.

Layers:

1. App-level: `session.messages[]` persisted to `${sessionId}.json`
2. Provider-level: `sdkSessionId` captured and stored for resume

Key design choices:

- **Capture provider session ID from stream**: `msg.session_id` stored for later resume
- **Replay + resume**: Both app history and provider thread ID used together

**Takeaway**: Persist both your conversation state AND provider's session handle.

## Patterns to Consider Adopting

1. **Provider abstraction with bare model IDs** - Clean separation of routing vs execution
2. **Event streaming with nested discriminators** - Type-safe event channels
3. **File-based resume eligibility** - Context file existence as "can resume" signal
4. **Relevance-scored memory selection** - Don't dump all context; score and select
5. **Marker-based state detection** - Explicit markers in output for state transitions
6. **Failure tracking with auto-pause** - Prevent runaway on rate limits
7. **Dual-layer session continuity** - Your state + provider's state
