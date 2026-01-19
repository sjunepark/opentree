# OpenCode Streaming Events

NDJSON event format from `opencode run --format json`.

## Event Envelope

All events share this structure:

```jsonc
{
  "type": "text | tool_use | step_start | step_finish | error",
  "timestamp": 1730000000000,  // ms since epoch
  "sessionID": "session_..."
  // + event-specific fields
}
```

## Event Types

### `text`

Model text output. Emitted when text part completes (`time.end` present).

```jsonc
{
  "type": "text",
  "timestamp": 1730000000000,
  "sessionID": "session_...",
  "part": {
    "id": "part_...",
    "sessionID": "session_...",
    "messageID": "message_...",
    "type": "text",
    "text": "Here is the explanation...",
    "synthetic": false,
    "ignored": false,
    "time": {
      "start": 1730000000000,
      "end": 1730000001234
    },
    "metadata": {}
  }
}
```

### `tool_use`

Completed tool call. Emitted when tool state is `completed`.

```jsonc
{
  "type": "tool_use",
  "timestamp": 1730000000000,
  "sessionID": "session_...",
  "part": {
    "id": "part_...",
    "sessionID": "session_...",
    "messageID": "message_...",
    "type": "tool",
    "callID": "call_...",
    "tool": "bash",
    "metadata": {},
    "state": {
      "status": "completed",
      "input": {
        "command": "git status"
      },
      "output": "On branch main\nnothing to commit",
      "title": "git status",
      "metadata": {},
      "time": {
        "start": 1730000000000,
        "end": 1730000001234,
        "compacted": null
      },
      "attachments": []
    }
  }
}
```

**Tool states (only `completed` emitted):**

- `pending` - Waiting to execute
- `running` - Currently executing
- `completed` - Finished successfully
- `error` - Failed

**Attachments (optional):**

```jsonc
{
  "attachments": [
    {
      "id": "part_...",
      "sessionID": "session_...",
      "messageID": "message_...",
      "type": "file",
      "mime": "image/png",
      "filename": "screenshot.png",
      "url": "file:///path/to/file",
      "source": {
        "type": "file",
        "path": "src/component.tsx",
        "text": { "value": "...", "start": 1, "end": 10 }
      }
    }
  ]
}
```

### `step_start`

Step boundary marker.

```jsonc
{
  "type": "step_start",
  "timestamp": 1730000000000,
  "sessionID": "session_...",
  "part": {
    "id": "part_...",
    "sessionID": "session_...",
    "messageID": "message_...",
    "type": "step-start",
    "snapshot": "snapshot_..."
  }
}
```

### `step_finish`

Step completion with token/cost info.

```jsonc
{
  "type": "step_finish",
  "timestamp": 1730000000000,
  "sessionID": "session_...",
  "part": {
    "id": "part_...",
    "sessionID": "session_...",
    "messageID": "message_...",
    "type": "step-finish",
    "reason": "end_turn",
    "snapshot": "snapshot_...",
    "cost": 0.0025,
    "tokens": {
      "input": 1500,
      "output": 200,
      "reasoning": 0,
      "cache": {
        "read": 500,
        "write": 100
      }
    }
  }
}
```

### `error`

Error during session.

```jsonc
{
  "type": "error",
  "timestamp": 1730000000000,
  "sessionID": "session_...",
  "error": {
    "name": "ProviderAuthError",
    "data": {
      "providerID": "anthropic",
      "message": "Invalid API key"
    }
  }
}
```

**Error types:**

| Name | Data Fields |
|------|-------------|
| `ProviderAuthError` | `providerID`, `message` |
| `UnknownError` | `message` |
| `MessageOutputLengthError` | (none) |
| `MessageAbortedError` | `message` |
| `APIError` | `message`, `statusCode?`, `isRetryable`, `responseHeaders?`, `responseBody?`, `metadata?` |

## Parsing Example (Rust)

```rust
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum OpenCodeEvent {
    #[serde(rename = "text")]
    Text { timestamp: u64, sessionID: String, part: Value },
    #[serde(rename = "tool_use")]
    ToolUse { timestamp: u64, sessionID: String, part: Value },
    #[serde(rename = "step_start")]
    StepStart { timestamp: u64, sessionID: String, part: Value },
    #[serde(rename = "step_finish")]
    StepFinish { timestamp: u64, sessionID: String, part: Value },
    #[serde(rename = "error")]
    Error { timestamp: u64, sessionID: String, error: Value },
}

fn parse_line(line: &str) -> Result<OpenCodeEvent, serde_json::Error> {
    serde_json::from_str(line)
}
```

## Comparison with Codex Events

| OpenCode | Codex | Notes |
|----------|-------|-------|
| `step_start` | `turn.started` | Session/turn boundary |
| `step_finish` | `turn.completed` | With token counts |
| `text` | `item.completed` (text) | Final text output |
| `tool_use` | `item.completed` (tool) | Tool call result |
| `error` | (various) | Error handling |

Key differences:

- OpenCode uses `part` wrapper for payloads
- OpenCode includes `tokens` breakdown in `step_finish`
- OpenCode has explicit `sessionID` on envelope
- No `thread.started` equivalent in OpenCode
