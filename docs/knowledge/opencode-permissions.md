# OpenCode Permission System

Rules engine controlling tool execution: `allow`, `ask`, or `deny`.

## Core Concepts

- **Actions**: `allow` (auto-execute), `ask` (prompt user), `deny` (block)
- **Evaluation**: Last matching rule wins
- **Patterns**: Wildcard matching for tools and arguments

## Configuration Formats

### Simple (All Tools)

```jsonc
{ "permission": "allow" }  // → { "*": "allow" }
```

### Per-Tool Defaults

```jsonc
{
  "permission": {
    "*": "ask",        // default
    "read": "allow",
    "bash": "allow",
    "edit": "deny"
  }
}
```

### Granular Patterns

```jsonc
{
  "permission": {
    "bash": {
      "*": "ask",              // default for bash
      "git *": "allow",        // allow git commands
      "npm *": "allow",        // allow npm
      "rm *": "deny"           // block rm
    },
    "edit": {
      "*": "deny",
      "src/**/*.ts": "allow",  // allow TypeScript edits
      "*.env*": "deny"         // block env files
    }
  }
}
```

## Wildcard Syntax

| Pattern | Matches |
|---------|---------|
| `*` | Zero or more characters |
| `?` | Exactly one character |
| `git *` | `git`, `git status`, `git commit -m "msg"` |
| `src/**/*.ts` | `src/foo.ts`, `src/a/b/c.ts` |

Special: Pattern ending with `*` (space-asterisk) makes trailing args optional.

## Tools and Their Patterns

| Tool | Pattern Content | Example |
|------|-----------------|---------|
| `read` | Absolute file path | `/home/user/project/.env` |
| `edit` | Relative path to worktree | `src/index.ts` |
| `bash` | Command string | `git status --porcelain` |
| `glob` | Glob pattern | `**/*.rs` |
| `grep` | Regex query | `fn main` |
| `list` | Directory path | `/home/user/project` |
| `task` | Subagent type/name | `code-reviewer` |
| `skill` | Skill name | `commit` |
| `webfetch` | URL | `https://api.example.com` |

### Special Guards

| Permission | Purpose | Default |
|------------|---------|---------|
| `external_directory` | Ops outside project dir | `ask` |
| `doom_loop` | Same tool call 3x in a row | `ask` |

## Rule Evaluation

Rules evaluate in order; **last match wins**.

```jsonc
{
  "permission": {
    "bash": {
      "*": "deny",         // 1. deny all bash
      "git *": "allow",    // 2. but allow git
      "git push *": "ask"  // 3. but ask for push
    }
  }
}
```

For `git push origin main`:

1. Matches `*` → `deny`
2. Matches `git *` → `allow`
3. Matches `git push *` → `ask` ← **final result**

## Runtime Override

### Environment Variable

```bash
OPENCODE_PERMISSION='{"bash":"allow","edit":"allow"}' opencode run "..."
```

### Session-Scoped "Always"

When prompted, choosing "Always" adds temporary rules for the session (not persisted to config).

## Defaults

If no rules match, default is `ask`.

Built-in defaults:

- `read`: `allow` (except `*.env*` files → `deny`, `*.env.example` → `allow`)
- Most tools: `allow`
- `external_directory`: `ask`
- `doom_loop`: `ask`

## Examples

### Full Automation (CI/Scripting)

```jsonc
{
  "permission": "allow"
}
```

### Safe Defaults with Git Allowed

```jsonc
{
  "permission": {
    "*": "ask",
    "read": "allow",
    "glob": "allow",
    "grep": "allow",
    "list": "allow",
    "bash": {
      "*": "ask",
      "git *": "allow",
      "npm *": "allow",
      "cargo *": "allow"
    }
  }
}
```

### Locked Down (Review Everything)

```jsonc
{
  "permission": {
    "*": "ask",
    "bash": "deny",
    "edit": {
      "*": "deny",
      "docs/**/*.md": "ask"
    }
  }
}
```

## Comparison with Codex

| Codex | OpenCode | Notes |
|-------|----------|-------|
| `--sandbox danger-full-access` | `permission: "allow"` | Auto-allow all |
| `--sandbox standard` | Default config | Ask for dangerous ops |
| N/A | Granular patterns | OpenCode more flexible |
