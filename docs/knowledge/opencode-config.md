# OpenCode Configuration

Layered JSON/JSONC config system with deep merge. Later sources override earlier.

## Config Locations (Merge Order)

1. **Remote org defaults** - `/.well-known/opencode` from authenticated providers
2. **Global config** - `~/.config/opencode/opencode.json`
3. **`OPENCODE_CONFIG` env** - Custom config file path
4. **Project config** - `opencode.json`/`opencode.jsonc` (searches upward to VCS root)
5. **Config directories** - `.opencode/`, `~/.opencode/`, `OPENCODE_CONFIG_DIR`
6. **`OPENCODE_CONFIG_CONTENT` env** - Inline JSON (highest precedence)

## Config Directory Contents

Directories (`.opencode/`, `~/.config/opencode/`, etc.) can contain:

| Path | Purpose |
|------|---------|
| `opencode.json(c)` | Config file |
| `agents/*.md` | Agent definitions |
| `commands/*.md` | Command templates |
| `modes/*.md` | Mode definitions (legacy, migrates to agent) |
| `plugins/*.{ts,js}` | Local plugins |

## Variable Substitution

Supported in config files before JSONC parsing:

- `{env:VAR_NAME}` - Environment variable (empty string if unset)
- `{file:path}` - File contents (trimmed). Relative to config file dir. `~/` expands.

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "anthropic": {
      "options": {
        "apiKey": "{env:ANTHROPIC_API_KEY}"
      }
    }
  },
  "instructions": ["{file:./AGENTS.md}"]
}
```

## Top-Level Config Keys

| Key | Type | Description |
|-----|------|-------------|
| `$schema` | string | `"https://opencode.ai/config.json"` |
| `theme` | string | Theme name |
| `logLevel` | string | Log level |
| `model` | string | Default model ID |
| `small_model` | string | Model for lightweight tasks |
| `default_agent` | string | Default agent name |
| `share` | string | `"manual"`, `"auto"`, `"disabled"` |
| `autoupdate` | bool/string | `true`, `false`, `"notify"` |

## Provider Configuration

```jsonc
{
  "provider": {
    "<provider-id>": {
      "npm": "@ai-sdk/openai-compatible",  // for custom providers
      "name": "Display Name",
      "options": {
        "apiKey": "...",
        "baseURL": "https://api.example.com/v1",
        "timeout": 300000,  // ms, or false to disable
        "setCacheKey": true,
        "headers": { "X-Custom": "value" }
      },
      "models": {
        "model-id": { "name": "Model Name" }
      }
    }
  }
}
```

**Built-in providers:** anthropic, openai, azure, google, google-vertex, amazon-bedrock, openrouter, xai, mistral, groq, deepinfra, cerebras, cohere, togetherai, perplexity, gitlab, github-copilot.

## Agent Configuration

```jsonc
{
  "agent": {
    "my-agent": {
      "model": "anthropic/claude-sonnet-4-20250514",
      "permission": { "bash": "allow" }
    }
  }
}
```

## Command Configuration

```jsonc
{
  "command": {
    "review": {
      "template": "Review this PR: $1",
      "description": "Code review",
      "agent": "reviewer",
      "model": "anthropic/claude-sonnet-4-20250514",
      "subtask": false
    }
  }
}
```

## MCP Server Configuration

```jsonc
{
  "mcp": {
    "server-name": {
      "command": "npx -y @modelcontextprotocol/server-filesystem",
      "args": ["/path/to/dir"],
      "env": { "KEY": "value" },
      "enabled": true,
      "timeout": 30000
    },
    "remote-server": {
      "url": "https://mcp.example.com",
      "headers": { "Authorization": "Bearer ..." },
      "oauth": true
    }
  }
}
```

## Permission Configuration

See `opencode-permissions.md` for details.

```jsonc
{
  "permission": {
    "*": "ask",
    "read": "allow",
    "bash": {
      "*": "ask",
      "git *": "allow"
    }
  }
}
```

## Other Settings

```jsonc
{
  "tui": {
    "scroll_speed": 1,
    "scroll_acceleration": { "enabled": true },
    "diff_style": "unified"
  },
  "server": {
    "port": 3000,
    "hostname": "localhost",
    "mdns": false,
    "cors": false
  },
  "watcher": {
    "ignore": ["node_modules/**", ".git/**"]
  },
  "formatter": {
    "prettier": {
      "command": "prettier --write",
      "extensions": [".ts", ".js", ".json"]
    }
  },
  "lsp": {
    "typescript": {
      "command": "typescript-language-server --stdio",
      "extensions": [".ts", ".tsx"]
    }
  },
  "compaction": {
    "auto": true,
    "prune": true
  },
  "plugin": [
    "opencode-plugin-example",
    "file://./my-plugin.ts"
  ],
  "instructions": [
    "./AGENTS.md",
    "https://example.com/instructions.md"
  ],
  "enterprise": {
    "url": "https://enterprise.example.com"
  }
}
```

## Environment Variables

### Config Sources

| Variable | Purpose |
|----------|---------|
| `OPENCODE_CONFIG` | Additional config file path |
| `OPENCODE_CONFIG_DIR` | Additional config directory |
| `OPENCODE_CONFIG_CONTENT` | Inline JSON config |
| `OPENCODE_PERMISSION` | Permission JSON override |

### Behavior Toggles

| Variable | Purpose |
|----------|---------|
| `OPENCODE_DISABLE_AUTOCOMPACT` | Disable auto compaction |
| `OPENCODE_DISABLE_PRUNE` | Disable context pruning |
| `OPENCODE_DISABLE_AUTOUPDATE` | Disable auto-update |
| `OPENCODE_DISABLE_DEFAULT_PLUGINS` | Skip default plugins |
| `OPENCODE_DISABLE_LSP_DOWNLOAD` | Skip LSP auto-download |
| `OPENCODE_DISABLE_TERMINAL_TITLE` | Don't update terminal title |
| `OPENCODE_DISABLE_MODELS_FETCH` | Don't fetch remote models |
| `OPENCODE_DISABLE_SHARE` | Disable sharing |
| `OPENCODE_AUTO_SHARE` | Enable auto-sharing |

### Claude Code Integration

| Variable | Purpose |
|----------|---------|
| `OPENCODE_DISABLE_CLAUDE_CODE` | Disable Claude Code integration |
| `OPENCODE_DISABLE_CLAUDE_CODE_PROMPT` | Skip Claude Code prompt |
| `OPENCODE_DISABLE_CLAUDE_CODE_SKILLS` | Skip Claude Code skills |

### Server Auth

| Variable | Purpose |
|----------|---------|
| `OPENCODE_SERVER_PASSWORD` | Password for serve/web |
| `OPENCODE_SERVER_USERNAME` | Username (default: `opencode`) |

### Experimental

| Variable | Purpose |
|----------|---------|
| `OPENCODE_EXPERIMENTAL` | Enable experimental features |
| `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` | Enable experimental models |
| `OPENCODE_EXPERIMENTAL_FILEWATCHER` | File watcher mode |
| `OPENCODE_EXPERIMENTAL_BASH_MAX_OUTPUT_LENGTH` | Bash output limit |
| `OPENCODE_EXPERIMENTAL_BASH_DEFAULT_TIMEOUT_MS` | Bash timeout |
| `OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX` | Max output tokens |

### Testing

| Variable | Purpose |
|----------|---------|
| `OPENCODE_TEST_HOME` | Override home for tests |
| `OPENCODE_CLIENT` | Client identifier (default: `cli`) |
| `OPENCODE_CALLER` | Caller identifier (vscode, etc.) |
