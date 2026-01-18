# Proposal: Read-Only Local UI for Runner (MVP)

## Problem

The runner already produces high-value artifacts (`.runner/state/tree.json`, `.runner/state/run_state.json`,
and `.runner/iterations/...`), but they are currently only accessible via files and terminal output.

A UI would make it much easier to:

- Visualize the task tree and node status at a glance
- Monitor progress during `runner loop` execution
- Review iteration history (agent summaries + guard output) quickly

## MVP Goals

- Read-only UI (no mutation of `.runner/` and no command execution)
- Local-only by default (bind `127.0.0.1`)
- Shows the full tree with status indicators and node details
- Provides fast access to iteration outputs/logs
- Supports “live updates” (near-real-time refresh without manual reload)

## MVP Non-Goals

- Remote access, authentication, or multi-user support
- Editing the tree or files from the UI
- Streaming detailed agent events (beyond “something changed”)
- Packaging/distribution (desktop app, installers)
- Polished graph visualization (raw list/tree is sufficient)

## Key Constraints (Align With Project Principles)

- UI is tooling: it must not compromise runner determinism or safety (`VISION.md`, `DECISIONS.md`)
- The runner’s canonical machine state remains `.runner/state/*` (UI is a viewer)
- Avoid introducing fragile coupling to “partial” filesystem state

## Key Insight: Backend Required Regardless (But Keep It Minimal)

If the UI is a browser app, it cannot reliably watch the local filesystem directly, and it should not be given
direct write access anyway.

We still need a small local backend that:

1. Reads `.runner/` snapshots safely
2. Notifies the UI when “something changed”

## MVP Decision (Recommended)

### Architecture: Web UI + Local Server

- Frontend: Svelte + Vite (fast iteration and good DX)
- Backend: `runner serve` (Rust), **read-only**, **binds `127.0.0.1` by default**

### Update Transport: Prefer “Signal + Re-fetch”

Instead of pushing full JSON payloads over WebSocket, the server should send lightweight “changed” signals.
The UI responds by re-fetching the relevant snapshot(s) over REST.

Benefits:

- Avoids handling partial/inconsistent reads as “events”
- Keeps the server and UI simpler
- Limits payload size and reduces event storm complexity

Implementation options (in order of recommended complexity):

1. Polling (simplest fallback)
2. Server-Sent Events (SSE) for low-friction push
3. WebSocket (only when we add bidirectional commands later)

## Proposed Backend: `runner serve`

### CLI

```text
runner serve [--bind 127.0.0.1] [--port 3000] [--project-dir .]
```

Defaults:

- `--bind 127.0.0.1` (local-only)
- `--port 3000`
- `--project-dir .` (single project per server instance for MVP)

### Responsibilities

- Read `.runner/state/tree.json` and `.runner/state/run_state.json`
- Enumerate `.runner/iterations/{run-id}/{iter}/...`
- Provide a “changed” signal stream (`/events`) when state changes

### API Surface (MVP)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/tree` | GET | Current `.runner/state/tree.json` |
| `/api/run-state` | GET | Current `.runner/state/run_state.json` |
| `/api/iterations` | GET | List run IDs and iterations (basic metadata) |
| `/api/iterations/{run}/{iter}` | GET | Per-iteration bundle (meta + output + paths) |
| `/api/iterations/{run}/{iter}/guard.log` | GET | Guard log (text) if present |
| `/events` | GET | SSE stream of “changed” notifications |

### SSE Message Format (MVP)

The event payload is a hint; the UI re-fetches the relevant endpoints.

```json
{
  "type": "tree_changed" | "run_state_changed" | "iteration_added",
  "ts": "RFC3339 timestamp"
}
```

### Rust HTTP Framework Choice

Prefer `axum` for ecosystem alignment and long-term maintainability. If keeping the core runner binary lean
becomes a concern, put `serve` behind a feature flag or a separate workspace crate.

## Data Consistency: Avoid Partial Reads

The two practical failure modes are:

1. The runner writes `tree.json`/`run_state.json` while the server is reading it.
2. File watch events arrive in bursts, and the UI thrashes.

Mitigations:

- Prefer runner-side atomic writes (write temp file then rename). If not guaranteed, server should tolerate
  transient JSON parse failures by retrying with a short backoff.
- Debounce filesystem notifications before emitting a single “changed” event.
- Keep the SSE payload small; treat it as an invalidation signal only.

## Proposed Frontend: `ui/` (Svelte + Vite)

### MVP Views

1. Tree view (collapsible), with per-node indicators:
   - `passes: true` => passed
   - `attempts > 0 && !passes` => in progress / retrying
   - `attempts == max_attempts && !passes` => stuck
2. Node detail panel (goal + acceptance criteria + attempts)
3. Iteration list (per run-id) with quick access to summary + guard log
4. Raw JSON panel (developer-friendly)
5. Connection status indicator (SSE connected / polling fallback)

## Development Workflow (Local)

```bash
# Terminal 1: Backend
cargo run -- serve --bind 127.0.0.1 --port 3001

# Terminal 2: UI (proxy /api + /events to backend)
cd ui && npm run dev
```

## Open Questions (Worth Deciding Early)

1. **Event source**: should the runner also emit an append-only `.runner/state/events.jsonl` stream?
   - If yes, the UI becomes a “tail + fetch” client and we can reduce reliance on filesystem watching.
2. **Serving built UI**: do we eventually want `runner serve` to also serve static files for distribution?
3. **Command execution**: if we ever add POST endpoints, do we require an explicit `--unsafe-commands` flag?

## Next Steps

1. Decide on `events.jsonl` vs filesystem watching (or explicitly defer with rationale)
2. Write a phased implementation plan (MVP polling -> SSE -> optional WS)
3. Implement `runner serve` (read-only REST + SSE invalidation)
4. Scaffold `ui/` and implement tree + iteration views
