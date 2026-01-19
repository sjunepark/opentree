# Codex to OpenCode Migration Guide

Migration reference for replacing Codex CLI with OpenCode in automated agents.

## Command Mapping

| Codex | OpenCode | Notes |
|-------|----------|-------|
| `codex exec` | `opencode run` | Main execution command |
| `--json` | `--format json` | NDJSON streaming |
| `--output-schema` | **None** | Use tool-based enforcement |
| `--output-last-message` | Shell redirect | `> output.txt` |
| `--sandbox danger-full-access` | `permission: "allow"` | In config |
| `--skip-git-repo-check` | N/A | Not needed |
| `-c key=value` | Provider config | `opencode.json` |
| `-` (stdin) | Positional or pipe | `opencode run "prompt"` |

## Critical Gap: No `--output-schema`

OpenCode does not support forced JSON schema on model output. Use **tool-based enforcement**:

### Strategy: Schema-Enforcing Tool

1. **Define a tool** with the desired output schema
2. **Instruct model** to call the tool for final output
3. **OpenCode validates** tool inputs automatically

### Implementation Pattern

#### 1. Create Custom Tool (MCP or Plugin)

```typescript
// .opencode/plugins/submit-result.ts
import { z } from "zod";

export const submitResult = {
  name: "submit_result",
  description: "Submit the final result. MUST be called to complete the task.",
  inputSchema: z.object({
    status: z.enum(["done", "retry"]),
    summary: z.string().describe("Brief summary of what was accomplished or why retry is needed")
  }),
  execute: async (input: { status: string; summary: string }) => {
    // Tool execution just returns the input as confirmation
    return JSON.stringify(input);
  }
};
```

#### 2. Add System Instructions

```jsonc
// opencode.json
{
  "instructions": [
    "When you complete your task, you MUST call the submit_result tool.",
    "Use status='done' when successful, status='retry' if you need another attempt.",
    "Always include a summary of what you did or why you need to retry."
  ]
}
```

#### 3. Parse Tool Call from Stream

```rust
// In your executor
fn extract_result(events: &[OpenCodeEvent]) -> Option<ExecutorOutput> {
    for event in events.iter().rev() {
        if let OpenCodeEvent::ToolUse { part, .. } = event {
            if part["tool"] == "submit_result" {
                let input = &part["state"]["input"];
                return Some(ExecutorOutput {
                    status: input["status"].as_str()?.to_string(),
                    summary: input["summary"].as_str()?.to_string(),
                });
            }
        }
    }
    None
}
```

## Streaming Event Mapping

| Codex Event | OpenCode Event |
|-------------|----------------|
| `thread.started` | (session start, no event) |
| `turn.started` | `step_start` |
| `turn.completed` | `step_finish` |
| `item.started` (text) | (no equivalent) |
| `item.completed` (text) | `text` |
| `item.started` (tool) | (no equivalent) |
| `item.completed` (tool) | `tool_use` |

### Key Differences

- OpenCode wraps payloads in `part` object
- OpenCode includes `sessionID` on every event
- OpenCode `step_finish` has detailed token breakdown
- OpenCode emits only completed items (no `started` events for items)

## Config Migration

### Before (Codex CLI Flags)

```rust
let mut cmd = Command::new("codex");
cmd.arg("exec")
    .arg("-c").arg("model_reasoning_effort=medium")
    .arg("--sandbox").arg("danger-full-access")
    .arg("--skip-git-repo-check")
    .arg("--json")
    .arg("--output-schema").arg(&schema_path)
    .arg("--output-last-message").arg(&output_path)
    .arg("-")
    .stdin(Stdio::piped());
```

### After (OpenCode CLI + Config)

```rust
let mut cmd = Command::new("opencode");
cmd.arg("run")
    .arg("--format").arg("json")
    .arg(&prompt)  // prompt as positional arg
    .current_dir(&workdir)
    .stdout(Stdio::piped());
```

```jsonc
// opencode.json in workdir
{
  "$schema": "https://opencode.ai/config.json",
  "model": "anthropic/claude-sonnet-4-20250514",
  "permission": "allow",
  "provider": {
    "anthropic": {
      "options": {
        "timeout": 300000
      }
    }
  }
}
```

## Output Capture

### Codex: `--output-last-message`

Writes final message to file.

### OpenCode: Stream Parsing

Parse NDJSON stream, collect `text` events, extract tool calls:

```rust
fn collect_output(stdout: impl BufRead) -> Result<String, Error> {
    let mut final_text = String::new();

    for line in stdout.lines() {
        let line = line?;
        let event: OpenCodeEvent = serde_json::from_str(&line)?;

        match event {
            OpenCodeEvent::Text { part, .. } => {
                if let Some(text) = part["text"].as_str() {
                    final_text = text.to_string();
                }
            }
            OpenCodeEvent::ToolUse { part, .. } => {
                // Check for submit_result tool
                if part["tool"] == "submit_result" {
                    // Extract structured output
                }
            }
            _ => {}
        }
    }

    Ok(final_text)
}
```

## Permission Config for Full Automation

Replace `--sandbox danger-full-access`:

```jsonc
{
  "permission": "allow"
}
```

Or more granular:

```jsonc
{
  "permission": {
    "*": "allow",
    "external_directory": "allow",
    "doom_loop": "ask"
  }
}
```

## Environment Variables

| Codex | OpenCode |
|-------|----------|
| `ANTHROPIC_API_KEY` | `ANTHROPIC_API_KEY` (same) |
| N/A | `OPENCODE_PERMISSION` (runtime override) |
| N/A | `OPENCODE_CONFIG_CONTENT` (inline config) |

## Testing Checklist

1. **Basic execution**: `opencode run --format json "test" > out.ndjson`
2. **Permission config**: Verify no prompts with `permission: "allow"`
3. **Tool-based schema**: Confirm model calls `submit_result` tool
4. **Stream parsing**: All event types parse correctly
5. **Error handling**: `error` events captured and handled
6. **Timeout**: Provider timeout respected

## Fallback Strategy

If tool-based schema fails (model doesn't call tool):

1. **Retry with explicit instruction**: "You MUST call submit_result now."
2. **Parse text output**: Attempt JSON extraction from final `text` event
3. **Validate with schema**: Use JSON Schema validation on extracted text
4. **Retry on failure**: Re-run with stronger instructions
