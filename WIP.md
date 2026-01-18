# WIP

- Enhance the node execution step thoroughly.
  - It currently does not behave as intended, for example:
    - Does not update the tree.json
    - Does not output or add logs appropriately.
  - To achieve these, we need to:
    - Add good system prompts to the executor.
    - Add deterministic guards etc.
    - Make the outside runeer or another agent perform the execution.
  - Once these are in place, we already have a agnet for working with trees, but need to check it.
  - Also, read `https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents`
