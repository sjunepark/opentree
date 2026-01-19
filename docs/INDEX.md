# Documentation Index

## Directory Structure

```text
docs/
├── project/    ← Implementation details (must be accurate)
├── knowledge/  ← General reference (not project-specific)
├── plans/      ← Documentation roadmap (what to write next)
└── case-studies/ ← Analysis of other projects for insights
```

## project/

**Project-specific implementation details.** Must stay accurate and current.

| Document | Description |
|----------|-------------|
| `agents.md` | Agent module boundaries and contracts |
| `context-preparation.md` | Prompt building, ephemeral context files, budget enforcement |
| `run-id-lifecycle.md` | Run ID generation, start/step flows, identity enforcement |

## knowledge/

**General reference material.** Not specific to this codebase; reusable patterns and external concepts.

| Document | Description |
|----------|-------------|
| `agents-md-guide.md` | How to write AGENTS.md files |
| `prd-and-specs.md` | PRD and specification guidance |
| `ralph-*.md` | Ralph agent loop reference |

## plans/

**Documentation roadmap.** Plans for docs to write, prioritized by value.

| Plan | Priority | Target |
|------|----------|--------|
| `state-updates.md` | High | How runner-owned fields are managed |
| `guard-execution.md` | Medium | Guard execution flow and output handling |
| `tree-validation.md` | Medium | Invariants and immutability rules |
| `iteration-logs.md` | Low | Log directory structure |

## case-studies/

**Analysis of other projects.** Insights, patterns, and takeaways to consider for our project.

| Directory | Description |
|-----------|-------------|
| `loom/` | Loom runtime analysis and lessons learned |
| `automaker/` | Autonomous AI dev studio - provider abstraction, state machines, event streaming |
