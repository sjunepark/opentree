---
status: pending
created_at: 2026-01-18T06:07:29Z
---

# Task ii

## Focus

Add a determinism/reproducibility section that makes the `app-server` tradeoffs explicit and aligns
with “fresh context per iteration”.

## Inputs

- `PROPOSAL2.md`
- `VISION.md` (fresh context principle)
- `ARCHITECTURE.md` (runner invariants)

## Work

1. Add a new section covering determinism considerations for moving to `codex app-server`:
   - per-iteration spawn vs long-lived subprocess (and why)
   - what must be recorded for reproducibility (Codex version, flags, protocol/schema versions, etc.)
2. Update “Option B: Migrate to app-server” to reflect the chosen deterministic lifecycle (or call out
   the open decision explicitly).
3. Ensure the new text is actionable (clear guidance, not just warnings).

## Output

## Handoff
