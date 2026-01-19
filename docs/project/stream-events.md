# Stream Event Inventory (UI)

This document captures the raw `stream.jsonl` event surface and how the UI should interpret it.
It is paired with fixtures in `ui/src/lib/fixtures/` for tests and regression checks.

## Fixtures

- `ui/src/lib/fixtures/stream-sample-executor.jsonl`
- `ui/src/lib/fixtures/stream-sample-planner.jsonl`

Fixtures are intentionally short but preserve event ordering. If the runner changes event formats,
update the fixtures and the mapping table below.

## Raw Event Types (Observed)

From the fixtures above:

- `thread.started`
  - fields: `thread_id`
- `turn.started`
- `item.started`
  - fields: `item.id`, `item.type`, `item.command?`, `item.status?`, `item.exit_code?`,
    `item.aggregated_output?`
- `item.completed`
  - fields: `item.id`, `item.type`, plus one of:
    - `item.text` (reasoning, agent_message)
    - `item.command` + `item.aggregated_output` + `item.exit_code` (command_execution)
- `turn.completed`
  - fields: `usage` (token counts)

Observed `item.type` values:

- `reasoning`
- `command_execution`
- `agent_message`

## Mapping Table (Raw -> UI)

| Raw event | Item type | Semantic UI event | Render behavior | Notes |
| --- | --- | --- | --- | --- |
| `thread.started` | - | None | Ignored in semantic view | Useful only in raw/debug view |
| `turn.started` | - | TurnStarted | Update status header / spinner | No transcript entry |
| `item.started` | `command_execution` | ExecCommandBegin | Start command cell with running state | Command string preview |
| `item.completed` | `command_execution` | ExecCommandEnd | Finalize command cell, show output, exit code | Output may be large; collapsible |
| `item.completed` | `reasoning` | ReasoningDelta/ReasoningSummary | Update status header; store summary for collapse | Not in transcript |
| `item.completed` | `agent_message` | AgentMessage | Render into transcript | If JSON text, show pretty+raw |
| `turn.completed` | - | TurnComplete | Stop spinner; show usage in metadata | Optional footer |

## Grouping Rules

- `item.started` + `item.completed` for the same `item.id` (type `command_execution`) form a single
  command cell. If only one is seen, still render a best-effort cell (streaming mid-flight).
- Consecutive `reasoning` items update the status header; the combined reasoning text is stored as a
  collapsed summary (if enabled).
- `agent_message` items render as the primary transcript message. If the message is JSON, render a
  formatted view with a raw toggle.

## Notes

- Codex TUI ignores low-level protocol events and only renders semantic events. UI should follow the
  same approach for the default (semantic) view. See `docs/knowledge/codex-streaming-ui.md`.
- New event types should be added to the fixtures and the mapping table before changing rendering
  logic.
