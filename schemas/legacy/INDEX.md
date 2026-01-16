# Legacy Schemas Index

Schemas copied from the pre-pivot repo. These are **legacy** but useful as reference patterns for:

- strict schema validation as a first-class constraint,
- versioning/migration discipline,
- documenting file contracts.

## Files

- [`board.schema.json`](board.schema.json)
  - Source: `schemas/board.schema.json` (also duplicated historically under `docs/blueprint/schemas/`).
  - Purpose (legacy): schema for a Kanban “board” model used by the prior Ralph/PRD runner.
  - Relevance to new repo: not directly reusable, but the schema style and validation expectations map to the
    new **task tree schema** requirement in `VISION.md`.
