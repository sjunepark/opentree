# OpenCode Custom Tools

OpenCode supports extending tool calling beyond built-ins (read, edit, bash) via custom tools and MCP servers.

## Overview

Two extension mechanisms:

| Mechanism | Description |
|-----------|-------------|
| **Custom Tools** | JS/TS modules in `.opencode/tool/` that the agent can call |
| **MCP Servers** | External Model Context Protocol servers whose tools become callable |

## File Structure & Naming

### Discovery Locations

- **Project-local:** `.opencode/tool/*.{ts,js}` or `.opencode/tools/*.{ts,js}`
- **Global:** `~/.config/opencode/tool/` or `~/.config/opencode/tools/`

### Naming Convention

| Export | Tool ID |
|--------|---------|
| `export default` | filename (e.g., `github.ts` → `github`) |
| `export const search` | `filename_exportname` (e.g., `github_search`) |

### Example Structure

```text
.opencode/
├── tool/
│   ├── github.ts        → registers "github" (default) + "github_search" (named)
│   └── database.ts      → registers "database"
├── package.json         → dependencies for tools
└── opencode.jsonc       → config
```

## The `tool()` API

### Basic Example

```ts
import { tool } from "@opencode-ai/plugin"

export default tool({
  description: "Query the database",
  args: {
    query: tool.schema.string().describe("SQL query"),
    limit: tool.schema.number().default(10),
  },
  async execute(args, ctx) {
    // args is typed: { query: string, limit: number }
    return "result string"
  },
})
```

### Key Points

- `tool.schema` is Zod (`tool.schema = z`)
- `args` must be a Zod raw shape (converted to JSON Schema for the model)
- `execute()` must return `Promise<string>` — not objects

### Context Object

```ts
type ToolContext = {
  sessionID: string
  messageID: string
  agent: string
  abort: AbortSignal
  metadata(input: { title?: string; metadata?: Record<string, any> }): void
  ask(input: {
    permission: string
    patterns: string[]
    always: string[]
    metadata: Record<string, any>
  }): Promise<void>
}
```

## Async Operations

Tools are async by default. Best practices:

```ts
async execute(args, ctx) {
  // Pass abort signal to fetch for cooperative cancellation
  const res = await fetch("https://api.example.com", {
    signal: ctx.abort,
  })

  // Check abort in long loops
  for (const item of items) {
    if (ctx.abort.aborted) return "Cancelled"
    await processItem(item)
  }

  // Report progress via metadata
  ctx.metadata({ title: "Processing 50%" })

  return "Done"
}
```

## Error Handling

Two approaches:

| Style | When to Use | Example |
|-------|-------------|---------|
| **Throw** | Hard failures (auth missing, invariant violated) | `throw new Error("GITHUB_TOKEN not set")` |
| **Return string** | Soft failures (agent can recover) | `return "No results found for query"` |

```ts
async execute(args, ctx) {
  // Hard failure - throw
  if (!process.env.API_KEY) {
    throw new Error("API_KEY environment variable required")
  }

  const result = await fetch(...)

  // Soft failure - return message
  if (result.items.length === 0) {
    return "No items found matching criteria"
  }

  return JSON.stringify(result.items, null, 2)
}
```

## Permissions

Tools can request permission before sensitive operations:

```ts
async execute(args, ctx) {
  // Ask permission before reading external files
  await ctx.ask({
    permission: "read",
    patterns: [args.filepath],
    always: ["*"],  // Pattern to persist if user chooses "always allow"
    metadata: {},
  })

  // Permission granted, proceed
  const content = await Bun.file(args.filepath).text()
  return content
}
```

### Configuring Tool Access

**In agent frontmatter:**

```yaml
---
tools:
  "*": false           # Deny all tools by default
  "my-tool": true      # Allow only this tool
---
```

**In `opencode.jsonc`:**

```jsonc
{
  "tools": {
    "dangerous-tool": false,
    "safe-tool": true
  }
}
```

## Dependencies

OpenCode auto-installs dependencies in `.opencode/`.

1. Create `.opencode/package.json`:

```json
{
  "dependencies": {
    "@octokit/rest": "^20.0.0"
  }
}
```

2. OpenCode runs `bun install` at startup

3. Import in your tool:

```ts
import { Octokit } from "@octokit/rest"

export default tool({
  description: "...",
  args: { ... },
  async execute(args, ctx) {
    const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN })
    // use octokit...
  },
})
```

## Tool Patterns

### Simple Pure Function

```ts
export default tool({
  description: "Calculate compound interest",
  args: {
    principal: tool.schema.number(),
    rate: tool.schema.number(),
    years: tool.schema.number(),
  },
  async execute({ principal, rate, years }) {
    const result = principal * Math.pow(1 + rate, years)
    return `Final amount: $${result.toFixed(2)}`
  },
})
```

### API Wrapper with Auth

```ts
export default tool({
  description: "Search GitHub PRs",
  args: {
    query: tool.schema.string(),
    limit: tool.schema.number().default(10),
  },
  async execute(args, ctx) {
    const token = process.env.GITHUB_TOKEN
    if (!token) throw new Error("GITHUB_TOKEN required")

    const res = await fetch(
      `https://api.github.com/search/issues?q=${encodeURIComponent(args.query)}`,
      {
        headers: { Authorization: `Bearer ${token}` },
        signal: ctx.abort,
      }
    )

    if (!res.ok) throw new Error(`GitHub API: ${res.status}`)

    const data = await res.json()
    return JSON.stringify(data.items.slice(0, args.limit), null, 2)
  },
})
```

### Multi-Tool Module

```ts
// .opencode/tool/math.ts
import { tool } from "@opencode-ai/plugin"

// Registers as "math"
export default tool({
  description: "Add numbers",
  args: { a: tool.schema.number(), b: tool.schema.number() },
  async execute({ a, b }) { return `${a + b}` },
})

// Registers as "math_multiply"
export const multiply = tool({
  description: "Multiply numbers",
  args: { a: tool.schema.number(), b: tool.schema.number() },
  async execute({ a, b }) { return `${a * b}` },
})
```

### Shell Out to Other Languages

Tool definitions must be JS/TS (Bun imports them), but can shell out to any language:

**Python:**

```ts
// .opencode/tool/python-analyzer.ts
import { tool } from "@opencode-ai/plugin"

export default tool({
  description: "Run Python analysis",
  args: {
    code: tool.schema.string(),
  },
  async execute(args) {
    const result = await Bun.$`python3 -c ${args.code}`.text()
    return result
  },
})
```

**Go:**

```ts
// .opencode/tool/go-runner.ts
export default tool({
  description: "Run Go script",
  args: { file: tool.schema.string() },
  async execute(args) {
    const result = await Bun.$`go run ${args.file}`.text()
    return result
  },
})
```

**Rust (compiled binary):**

```ts
// .opencode/tool/rust-cli.ts
export default tool({
  description: "Call Rust CLI",
  args: { input: tool.schema.string() },
  async execute(args) {
    const result = await Bun.$`./my-rust-binary ${args.input}`.text()
    return result
  },
})
```

**Summary:**

| Layer | Language |
|-------|----------|
| Tool wrapper | Must be TS/JS (Bun imports it) |
| Actual logic | Anything — Python, Go, Rust, Ruby, shell scripts, binaries |

## Agent Interaction Flow

```text
┌─────────────────────────────────────────────────────────────┐
│ 1. Discovery: OpenCode scans .opencode/tool/*.ts            │
│    └─ Imports modules, extracts tool definitions            │
├─────────────────────────────────────────────────────────────┤
│ 2. Registration: Zod schemas → JSON Schema                  │
│    └─ Provider-specific sanitization applied                │
├─────────────────────────────────────────────────────────────┤
│ 3. Model calls tool                                         │
│    └─ OpenCode builds ctx (abort, permissions, metadata)    │
├─────────────────────────────────────────────────────────────┤
│ 4. Permission check (if ctx.ask called)                     │
│    └─ May block for user approval                           │
├─────────────────────────────────────────────────────────────┤
│ 5. execute() runs                                           │
│    └─ Plugin hooks: tool.execute.before / after             │
├─────────────────────────────────────────────────────────────┤
│ 6. Output returned to agent                                 │
│    └─ Auto-truncated if too long (Truncate.output)          │
└─────────────────────────────────────────────────────────────┘
```

## MCP Servers

Alternative to custom tools — connect external MCP servers.

**Local MCP server:**

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "mcp_everything": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-everything"],
      "enabled": true
    }
  }
}
```

**Remote MCP server:**

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "my-remote-mcp": {
      "type": "remote",
      "url": "https://my-mcp-server.com",
      "enabled": true,
      "headers": {
        "Authorization": "Bearer MY_API_KEY"
      }
    }
  }
}
```

Tools from MCP servers appear automatically and can be controlled via the same permission system.
