# Codex CLI Streaming UI Rendering

How Codex transforms raw API streaming events into polished terminal UI.

## Core Principle

**Raw JSON events are NOT shown to users.** Codex parses SSE streams into `EventMsg` types, then the TUI selectively renders only high-level events while ignoring low-level protocol details.

## Event Dispatch Logic

From `codex-rs/tui2/src/chatwidget.rs`:

```rust
fn dispatch_event_msg(&mut self, id: Option<String>, msg: EventMsg, from_replay: bool) {
    match msg {
        // RENDERED: High-level UI events
        EventMsg::TurnStarted(_) => self.on_task_started(),
        EventMsg::TurnComplete(ev) => self.on_task_complete(ev.last_agent_message),
        EventMsg::AgentMessageDelta(ev) => self.on_agent_message_delta(ev.delta),
        EventMsg::AgentMessage(ev) => self.on_agent_message(ev.message),
        EventMsg::AgentReasoningDelta(ev) => self.on_agent_reasoning_delta(ev.delta),
        EventMsg::ExecCommandBegin(ev) => self.on_exec_command_begin(ev),
        EventMsg::ExecCommandOutputDelta(delta) => self.on_exec_command_output_delta(delta),
        EventMsg::ExecCommandEnd(ev) => self.on_exec_command_end(ev),
        EventMsg::PlanUpdate(update) => self.on_plan_update(update),

        // IGNORED: Low-level streaming protocol events
        EventMsg::RawResponseItem(_)
        | EventMsg::ItemStarted(_)
        | EventMsg::ItemCompleted(_)
        | EventMsg::AgentMessageContentDelta(_)
        | EventMsg::ReasoningContentDelta(_)
        | EventMsg::ReasoningRawContentDelta(_) => {}
    }
}
```

## Event Categories

### Rendered Events (User-Facing)

| Event | UI Behavior |
|-------|-------------|
| `TurnStarted` | Shows spinner/status indicator |
| `TurnComplete` | Finalizes turn, hides spinner |
| `AgentMessageDelta` | Streams text into transcript with animation |
| `AgentMessage` | Commits final message to transcript |
| `AgentReasoningDelta` | Updates status header ("Working...") |
| `ExecCommandBegin` | Shows command cell with running state |
| `ExecCommandOutputDelta` | Streams command output |
| `ExecCommandEnd` | Shows exit code, finalizes command cell |
| `PlanUpdate` | Updates plan/todo display |

### Ignored Events (Protocol-Level)

| Event | Why Ignored |
|-------|-------------|
| `ItemStarted` | Internal protocol; no user-facing meaning |
| `ItemCompleted` | Redundant with higher-level completion events |
| `RawResponseItem` | Raw API payload; not for display |
| `AgentMessageContentDelta` | Subsumed by `AgentMessageDelta` |

## Streaming Text Rendering

### Architecture

```text
AgentMessageDelta events
        ↓
  StreamController (buffers, newline-gates)
        ↓
  Commit animation (1 logical line per tick)
        ↓
  AgentMessageCell (transcript gutter + markdown wrapping)
```

### Newline-Gated Buffering

Deltas accumulate until a newline arrives, then complete lines are committed:

```rust
// codex-rs/tui2/src/streaming/controller.rs
pub(crate) fn push(&mut self, delta: &str) -> bool {
    self.state.collector.push_delta(delta);

    if delta.contains('\n') {
        let newly_completed = self.state.collector.commit_complete_lines();
        if !newly_completed.is_empty() {
            self.state.enqueue(newly_completed);
            return true; // triggers commit animation
        }
    }
    false
}
```

### Commit Animation

Releases at most one logical line per tick for smooth visual effect:

```rust
pub(crate) fn on_commit_tick(&mut self) -> (Option<Box<dyn HistoryCell>>, bool) {
    let step = self.state.step(); // at most 1 logical line
    (self.emit(step), self.state.is_idle())
}
```

## Transcript Formatting

### Gutter Style

- First line of message: `•` (dim bullet)
- Continuation lines: `` (two-space indent)

```rust
// codex-rs/tui2/src/history_cell.rs
let gutter_first_visual_line: Line = if at_cell_start && self.is_first_line {
    "• ".dim().into()
} else {
    "  ".into()
};
```

### Width-Aware Wrapping

Wrapping happens at render time based on terminal width, so resize reflows text:

```rust
let opts = RtOptions::new(width as usize)
    .initial_indent(compose_indent(&gutter_first_visual_line, &logical.initial_indent))
    .subsequent_indent(compose_indent(&gutter_continuation, &logical.subsequent_indent));

let (wrapped, wrapped_joiners) =
    crate::wrapping::word_wrap_line_with_joiners(&logical.content, opts);
```

## Reasoning Display

Reasoning is NOT streamed into the chat transcript. Instead:

1. First bold segment updates the status header ("Working...")
2. Full reasoning saved to buffer
3. On completion, rendered as a collapsed summary block

```rust
fn on_agent_reasoning_delta(&mut self, delta: String) {
    self.reasoning_buffer.push_str(&delta);

    if let Some(header) = extract_first_bold(&self.reasoning_buffer) {
        self.set_status_header(header); // updates spinner header
    }
    self.request_redraw();
}
```

## Command Execution Display

Commands have three phases:

1. **Begin**: Show command text, "running" indicator
2. **Output deltas**: Stream stdout/stderr into collapsible section
3. **End**: Show exit code (green=0, red=non-zero), finalize cell

## Design Principles

1. **Transform, don't echo**: Never show raw JSON to users
2. **Semantic grouping**: Group related events into logical UI cells
3. **Progressive disclosure**: Stream incrementally, animate commits
4. **Status over noise**: Reasoning updates status header, not transcript
5. **Width-responsive**: Reflow on terminal resize

## Implications for Our Runner

When processing Codex output for display:

1. Parse raw events into semantic categories
2. Buffer text until logical boundaries (newlines, event completion)
3. Transform `item.started`/`item.completed` into status indicators, not raw JSON
4. Group command execution phases into cohesive UI cells
5. Show reasoning as status updates during execution, summary after

## UI Fixtures and Mapping (Web UI)

The web UI keeps a small set of NDJSON fixtures and a raw-to-semantic mapping table to track
stream formats and rendering behavior:

- Mapping + taxonomy: `docs/project/stream-events.md`
- Fixtures: `ui/src/lib/fixtures/stream-sample-executor.jsonl`,
  `ui/src/lib/fixtures/stream-sample-planner.jsonl`

Update these whenever new event types appear or rendering logic changes.
