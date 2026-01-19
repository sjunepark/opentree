# Agent Abstractions

The runner wraps agent interactions in explicit structs so orchestration stays focused on policy and state.

## Modules

```text
runner/src/agents/
├── decomposer.rs ← DecomposerAgent: produce child specs
├── executor.rs ← ExecutorAgent: perform work on selected node
└── mod.rs      ← shared helpers (schema writing)
```

## Responsibilities

### DecomposerAgent

- **Side effects:** none (`allows_side_effects() == false`)
- **Output:** `DecompositionOutput`
- **Schema:** `.runner/state/decomposer_output.schema.json`
- **Artifacts:** writes `planner_output.json`, `planner_executor.log`, `planner_stream.jsonl`

### ExecutorAgent

- **Side effects:** allowed (`allows_side_effects() == true`)
- **Output:** `AgentOutput`
- **Schema:** `.runner/state/executor_output.schema.json`
- **Artifacts:** writes `output.json`, `executor.log`, `stream.jsonl`

## Interface

Both agents expose a `run()` method that:

1. Writes its schema file into `.runner/state/`
2. Builds a prompt via `PromptBuilder` using shared `PromptInputs`
3. Executes the request via the `Executor` trait
4. Parses and returns the typed output

The orchestration layer (`runner/src/step.rs`) owns state transitions and guard policy, but agent execution details live in these wrappers.

## Testing

- Unit tests live next to each agent module and use a fake `Executor` to verify:
  - schema writing
  - prompt construction
  - output parsing
- Integration tests (e.g., `runner/tests/harness_lifecycle.rs`) continue to validate end-to-end step behavior using `ScriptedExecutor`.

## Source Files

- `runner/src/agents/decomposer.rs`
- `runner/src/agents/executor.rs`
- `runner/src/io/prompt.rs`
- `runner/src/step.rs`
