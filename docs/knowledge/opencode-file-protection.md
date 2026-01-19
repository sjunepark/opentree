# OpenCode File Protection

OpenCode can prevent editing specific files via its **permission system**.

## Configuration

Set in `opencode.json` (project root) or `~/.config/opencode/opencode.json` (global).

### Deny Edits to Specific Files

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "permission": {
    "edit": {
      "*": "allow",
      "package-lock.json": "deny",
      "bun.lock": "deny",
      ".env": "deny",
      "infra/**": "deny"
    }
  }
}
```

### Allow Edits Only in Specific Folders

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "permission": {
    "edit": {
      "*": "deny",
      "src/**": "allow"
    }
  }
}
```

### Require Manual Approval for Sensitive Paths

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "permission": {
    "edit": {
      "*": "allow",
      "db/migrations/**": "ask",
      "terraform/**": "ask"
    }
  }
}
```

## Pattern Rules

- Wildcard patterns supported (`*`, `**`)
- **Last matching rule wins**
- Values: `"allow"`, `"deny"`, `"ask"`

## What `edit` Permission Covers

The `edit` permission gates these tools:

- `edit`
- `write`
- `patch`
- `multiedit`

## Per-Agent Restrictions

You can also restrict tools at the agent level in agent frontmatter:

```yaml
---
tools:
  "edit": false
  "write": false
---
```

This creates a "read-only" agent that cannot modify files.

## Ignore vs Protection

| Mechanism | Purpose | Enforced? |
|-----------|---------|-----------|
| `permission.edit` | Prevents modifications | Yes |
| `.gitignore` / `.ignore` | Hides files from search/listing | No |
| `watcher.ignore` | Reduces file watching noise | N/A |

### Ignore Patterns (Search/Listing Only)

`.gitignore` and `.ignore` affect what the agent *sees* during search, but do **not** prevent edits.

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "watcher": {
    "ignore": ["node_modules/**", "dist/**", ".git/**"]
  }
}
```

This only affects file watching (UI updates, reload triggers), not edit protection.

## Security Note

Permissions are **UX guardrails, not a security sandbox**. The agent could theoretically bypass them via bash commands.

For hard isolation, run OpenCode in a container or VM.

## Example: Comprehensive Protection Config

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "permission": {
    "edit": {
      "*": "allow",

      // Lock files - never edit
      "package-lock.json": "deny",
      "bun.lock": "deny",
      "yarn.lock": "deny",
      "pnpm-lock.yaml": "deny",

      // Secrets - never edit
      ".env": "deny",
      ".env.*": "deny",
      "**/*.pem": "deny",
      "**/*.key": "deny",

      // Infrastructure - require approval
      "terraform/**": "ask",
      "k8s/**": "ask",
      "docker-compose*.yml": "ask",

      // Database - require approval
      "db/migrations/**": "ask",
      "prisma/migrations/**": "ask"
    }
  }
}
```
