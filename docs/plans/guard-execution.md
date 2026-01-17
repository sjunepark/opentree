# Plan: Guard Execution Flow

**Priority:** Medium
**Target:** `docs/project/guard-execution.md`

## Why This Matters

Guards are the verification mechanism that makes the loop self-correcting. Understanding when they run and how output is handled helps debug CI failures.

## Topics to Cover

### When Guards Run

```text
agent status → guard behavior
─────────────────────────────
done         → run `just ci`
retry        → skip (save CI cycles)
decomposed   → skip (no code to verify)
```

### Execution Parameters

- **Timeout:** 30 minutes (shared with executor budget)
- **Output cap:** 1MB (`DEFAULT_OUTPUT_LIMIT_BYTES`)
- **Truncation:** preserves start, appends `[truncated N bytes]`

### Guard Outcome

| Exit Code | Outcome | Effect |
|-----------|---------|--------|
| 0 | `Pass` | `passes = true` |
| non-zero | `Fail` | `attempts += 1` |
| skipped | `Skipped` | `attempts += 1` |

### Output Capture

- stdout/stderr combined
- Written to `.runner/iterations/{run}/{iter}/guard.log`
- Truncated if > 1MB

## Source Files

- `runner/src/io/guards.rs` — `GuardRunner`, `JustGuardRunner`
- `runner/src/step.rs` — guard invocation logic
