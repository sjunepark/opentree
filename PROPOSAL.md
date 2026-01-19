# Proposal: Agent Abstraction for Runner

## Problem

The runner currently has two agents (tree agent, executor agent) with more planned (reviewer, error handler). Agent logic is implicit in `step.rs` — prompts, schemas, timeouts, and orchestration are interleaved.

Pain points:

1. **Testing** — Testing individual agent behavior requires verbose setup; no `agent.run()` API
2. **Scalability** — Adding agents means growing `step.rs` with more inline logic
3. **No clear contract** — Agent responsibilities are implicit in code flow

## Proposed Solution

Create explicit agent structs that encapsulate:

- **Input**: Prompt inputs, context
- **Config**: Timeout, schema, prompt builder
- **Output**: Typed, schema-validated result
- **Side effects**: Whether the agent may modify files

Each agent becomes a first-class object with a `run()` method.

## Requirements

1. Agents own their configuration (timeout, schema)
2. Agents have independent timeout budgets
3. Agents can be chained (output of one feeds into another)
4. Testing: `agent.run(input)` should work in isolation
5. Executor trait remains the underlying execution mechanism

## Agents to Support

| Agent | Side Effects | Output |
|-------|--------------|--------|
| Tree Agent | None | `TreeDecision` |
| Executor Agent | Yes | `AgentOutput` |
| (Future) Reviewer | None | Review result |
| (Future) Error Handler | TBD | Recovery action |

## Non-Goals

- No trait required unless polymorphism is needed
- No changes to the `Executor` trait itself
- No changes to schemas or prompt templates (just how they're organized)

## Success Criteria

- [ ] `TreeAgent::run()` and `ExecutorAgent::run()` exist with clean APIs
- [ ] `step.rs` orchestrates agents without building prompts/schemas inline
- [ ] Individual agent tests don't require full `run_step` setup
- [ ] Adding a new agent type doesn't require modifying `step.rs` core logic

## Plan

- [x] Decide module layout (`runner/src/agents/`, filenames, public API)
- [x] Define per-agent contract: inputs, config (timeout/schema), output type, side effects flag
- [x] Extract `TreeAgent` (no side effects): prompt build + schema write + `execute_and_load_json`
- [x] Extract `ExecutorAgent` (side effects): prompt build + schema write + `execute_and_load`
- [x] Refactor `runner/src/step.rs` to: build shared inputs once, call agents, keep state/guard policy unchanged
- [x] Add unit tests for each agent `run()` (fake executor, fixed outputs, budget/timeouts exercised)
- [x] Add/adjust integration tests to ensure step orchestration unchanged but `step.rs` slimmer (existing harness coverage; no edits needed)
- [x] Update docs (`docs/project/`) to describe agent module boundaries and testing approach
- [x] Run `just ci`

### Unresolved Questions

1. Where should agents live? (`runner/src/agents/` module?)
   1. Make your best decision
2. Should prompt builders move into agent modules or stay in `io/prompt.rs`?
   1. Make your best decision for clean design and architecture
3. How to handle iteration directory paths — passed in or agent-managed?
   1. Make your best decision for clean design and architecture
