# WIP

## Now

- Completed `.plan/3` runner core-first work (classifier + immutability + state updates).
- Documented agent-declared status model in DECISIONS.md, ARCHITECTURE.md, VISION.md.

## Next

Refactor for agent-declared status model:

1. **Directory structure** — migrate `.runner/tree.json` → `.runner/state/tree.json`, add `context/`
2. **Agent output types** — add `AgentStatus` enum (`Done`/`Retry`/`Decomposed`), `AgentOutput` struct
3. **Status validator** — validate decomposed→children added, done/retry→no children
4. **Context writer** — clear and write `context/` each iteration (goal, history, failure)
5. **Update state_update.rs** — integrate agent status into transition logic
6. **Update core loop** — read agent output, validate status, conditional guard execution

Research:

- Codex CLI structured output: <https://developers.openai.com/codex/noninteractive/#create-structured-outputs-with-a-schema>
- Claude Code equivalent for structured output

## Notes

- Plan CLI needs `pyyaml` when run via `uv run --with pyyaml`.
