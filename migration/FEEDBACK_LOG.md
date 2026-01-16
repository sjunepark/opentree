# Feedback Log (Runner + Agent Loop)

Append-only (or mostly append-only) notes about mistakes, failure modes, and fixes discovered during development
of the deterministic agent loop runner.

## Template

### YYYY-MM-DD — Issue title

- **Symptom:** what happened
- **Root cause:** why it happened
- **Fix:** what changed
- **Prevention:** tests/guards/invariants that would catch this next time
- **Follow-ups:** any deferred work

## Common Failure Modes to Watch

- Tree schema drift (docs say one thing; code does another).
- Non-deterministic leaf selection (unstable ordering / tie-break bugs).
- “Pass without green guards” (state update bug or guard not enforced).
- Mutation of passed nodes (immutability rule not enforced everywhere).
- Retry exhaustion handling (leaf remains stuck without rewrite/expand decision).
- Tool boundary escapes (paths outside workspace, uncontrolled commands).
- Guard entrypoint mismatch across platforms (Mac/Linux differences).
