# Glossary

Key terms used throughout the reboot docs. These should remain stable to avoid subtle spec drift.

- **Runner**: Deterministic CLI that selects nodes, enforces schemas/invariants, runs guards, and orchestrates agent runs.
- **Agent**: The LLM tool invoked by the runner (fresh session per iteration).
- **Task Tree**: Ordered node tree that represents the plan; source of truth for progress and selection.
- **Node**: A single goal-oriented unit in the task tree.
- **Leaf**: A node with no children (at the moment) that is selected for decomposition or execution.
- **DECOMPOSE**: Iteration mode where the agent expands a leaf into ordered children.
- **EXECUTE**: Iteration mode where the agent performs the work for a single-context leaf.
- **Guards / Gates**: Deterministic validation entrypoint(s) (format/lint/test/etc) owned by the runner.
