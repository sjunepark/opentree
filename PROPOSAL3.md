# Proposal: Frontend UI for Runner

## Problem

The runner CLI produces structured data (task tree, iteration logs, run state) that is currently only accessible via files and terminal output. A frontend UI would enable:

- Visualizing the task tree with node statuses at a glance
- Monitoring live progress during `runner loop` execution
- Reviewing iteration history (logs, tree snapshots, guard output)
- Future: sending commands/prompts to the runner

## Requirements Gathered

| Question | Answer |
|----------|--------|
| Platform preference | Uncertain; Svelte preferred for web UI |
| Desktop concerns | Tauri layer complexity; but better filesystem access |
| Read/write | Read-only for MVP; future command execution (not direct file editing) |
| Live updates | Yes, required |
| Tree view | Show whole tree with each node's status |
| Logs/execution view | Tree graph view later; raw JSON display sufficient for MVP |

## Key Insight: Backend Required Regardless

Live updates require filesystem watching. Whether web or desktop, something must:

1. Watch `.runner/` for changes (tree.json, run_state.json, iterations/)
2. Push updates to the UI in real-time

The question is not "web vs desktop" but "where does the backend live?"

## Options Considered

### Option A: Web App + Separate Server

```text
┌──────────────────┐     WebSocket/REST     ┌──────────────────┐
│  Svelte frontend │ ◄──────────────────────► │  Rust server     │
│  (browser)       │                         │  (watches files) │
└──────────────────┘                         └──────────────────┘
```

**Pros:**

- Familiar web development workflow
- Hot reload during UI development
- Easy debugging (browser devtools)
- Server can be reused for future features (command execution)

**Cons:**

- Two processes to run (server + dev server or static serve)

### Option B: Tauri Desktop App

```text
┌─────────────────────────────────────────┐
│  Tauri app                              │
│  ┌─────────────────┐  IPC  ┌──────────┐ │
│  │ Svelte frontend │ ◄────► │ Rust     │ │
│  │ (webview)       │       │ backend  │ │
│  └─────────────────┘       └──────────┘ │
└─────────────────────────────────────────┘
```

**Pros:**

- Single app to launch
- Native filesystem access without network layer
- Native window management

**Cons:**

- Tauri learning curve
- Harder debugging (IPC layer)
- More complex build pipeline

### Option C: Hybrid (Start Web, Wrap with Tauri Later)

Build Option A architecture, but design the server API such that it can be embedded in Tauri later.

**Pros:**

- Start simple, add complexity incrementally
- Validate UI/UX before committing to desktop

**Cons:**

- Requires upfront API design discipline

## Decision: Option A (Web + Server) for MVP

**Rationale:**

1. **Runner is already Rust** — adding a server subcommand is natural
2. **Svelte is comfortable** — leverage existing skills
3. **Live reload** — faster UI iteration during development
4. **Future-proof** — command execution fits naturally as POST endpoints
5. **Tauri optional later** — if single-app UX is desired, can wrap

## Proposed Architecture

### Backend: `runner serve` Subcommand

New subcommand added to the existing runner binary.

```text
runner serve [--port 3000] [--project-dir .]
```

**Responsibilities:**

- Watch `.runner/` directory for changes (using `notify` crate)
- Serve REST API for initial data fetch
- Push updates via WebSocket on file changes

**API Surface (MVP):**

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/tree` | GET | Current tree.json |
| `/api/run-state` | GET | Current run_state.json |
| `/api/iterations` | GET | List of runs and iterations |
| `/api/iterations/{run}/{iter}` | GET | Specific iteration data (meta, output, guard log) |
| `/ws` | WebSocket | Live updates (tree, run_state changes) |

**WebSocket Message Format:**

```json
{
  "type": "tree_updated" | "run_state_updated" | "iteration_added",
  "data": { ... }
}
```

### Frontend: Svelte + Vite

Separate package (e.g., `ui/` directory) with standard Vite + Svelte setup.

**MVP Views:**

1. **Tree View** — Collapsible tree showing all nodes with status indicators
   - `passes: true` → green checkmark
   - `attempts > 0 && !passes` → yellow/orange (in progress or retrying)
   - `attempts == max_attempts` → red (stuck)
   - Expandable to show node details (goal, acceptance criteria)

2. **Raw JSON Panel** — Formatted JSON display of tree.json (developer-friendly)

3. **Connection Status** — Indicator showing WebSocket connection state

**Future Views (not MVP):**

- Iteration timeline (list of iterations with status)
- Tree graph visualization (d3 or similar)
- Command input panel (send prompts to runner)

### Directory Structure

```text
.
├── runner/              # Existing Rust crate
│   └── src/
│       ├── serve/       # New: HTTP/WebSocket server
│       │   ├── mod.rs
│       │   ├── api.rs   # REST endpoints
│       │   └── ws.rs    # WebSocket handler
│       └── ...
├── ui/                  # New: Svelte frontend
│   ├── package.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── App.svelte
│   │   ├── lib/
│   │   │   ├── api.ts       # REST client
│   │   │   ├── websocket.ts # WS connection
│   │   │   └── stores.ts    # Svelte stores for tree, run_state
│   │   └── components/
│   │       ├── TreeView.svelte
│   │       ├── NodeItem.svelte
│   │       └── JsonPanel.svelte
│   └── ...
└── ...
```

### Development Workflow

```bash
# Terminal 1: Run the server
cargo run -- serve --port 3001

# Terminal 2: Run Vite dev server (proxies API to backend)
cd ui && npm run dev
```

Vite config proxies `/api` and `/ws` to the Rust server.

### Data Flow

```text
┌─────────────────────────────────────────────────────────────────┐
│  .runner/                                                       │
│  ├── state/tree.json ──────┐                                    │
│  ├── state/run_state.json ─┼─► notify watcher ─► WebSocket push │
│  └── iterations/... ───────┘                                    │
└─────────────────────────────────────────────────────────────────┘
                                          │
                                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  Svelte frontend                                                │
│  ├── WebSocket connection (receives live updates)               │
│  ├── Svelte stores (reactive state)                             │
│  └── Components (render tree, JSON, etc.)                       │
└─────────────────────────────────────────────────────────────────┘
```

## Open Questions

1. **Port configuration** — Hardcode `localhost:3000` for MVP, or make configurable from start?

2. **Tree view style** — Collapsible tree with status badges, or just formatted JSON for MVP?

3. **Rust HTTP framework** — `axum` (async, popular) vs `warp` vs `actix-web`? Suggest `axum` for ecosystem alignment.

4. **Static file serving** — Should `runner serve` also serve the built frontend, or keep separate?

5. **Multiple project support** — MVP assumes single project directory; future may need project switching.

## Next Steps

1. Answer open questions above
2. Create implementation plan (phased)
3. Set up `ui/` scaffold with Vite + Svelte
4. Implement `runner serve` with minimal API
5. Build tree view component
6. Add WebSocket live updates
