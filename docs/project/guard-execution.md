# Guard Execution Flow

Guards are the verification mechanism that makes the loop self-correcting. Understanding when they run and how output is handled helps debug CI failures.

## When Guards Run

```text
agent status → guard behavior
─────────────────────────────
done         → run guard command
retry        → skip (save CI cycles)
decomposed   → skip (no code to verify)
```

The decision happens in `run_guards_if_needed` (`runner/src/io/guards.rs:85-94`).

## Execution Parameters

| Parameter | Default | Source |
|-----------|---------|--------|
| Timeout | 30 min (shared with executor) | `iteration_timeout_secs` in config |
| Output cap | 100 KB | `guard_output_limit_bytes` in config |
| Command | `just ci` | `guard.command` in config |

The guard shares the iteration timeout budget with the executor — whatever time remains after agent execution becomes the guard timeout.

## Guard Outcome

| Exit Code | Outcome | Effect |
|-----------|---------|--------|
| 0 | `Pass` | `passes = true` |
| non-zero | `Fail` | `attempts += 1` |
| timeout | `Fail` | logs "guard timed out" |
| skipped | `Skipped` | handled by status rules (retry increments attempts; decomposed doesn't) |

## Output Capture

- stdout and stderr drained concurrently while the command runs (prevents pipe deadlocks)
- stdout and stderr combined into single log
- Written to `.runner/iterations/{run_id}/{iter}/guard.log`
- Truncated if exceeds limit: preserves start, appends `[truncated N bytes]`
- If per-stream buffers overflow, logs include `[stdout truncated N bytes]` / `[stderr truncated N bytes]`
- Log format separates streams with `=== stdout ===` and `=== stderr ===` headers

## Source Files

- `runner/src/io/guards.rs` — `GuardRunner` trait, `CommandGuardRunner`, `run_guards_if_needed`
- `runner/src/io/config.rs` — `guard_output_limit_bytes`, `guard.command`, `iteration_timeout_secs`
- `runner/src/step.rs` — guard invocation in `run_step`
