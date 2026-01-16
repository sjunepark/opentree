# Ralph Architecture

Agent-first, repo-specific architecture notes.

## Hard Constraints

- Keep this file **≤ 250 lines** (gates enforce this).
- Replace all `RALPH_TODO` placeholders (gates enforce this).
- Put detailed rationale close to code (comments). Use this doc for the *stable* high-level model + invariants.

---

## System Overview

RALPH_TODO: What does this repo do? What’s in/out of scope? Who are the users?

## Platform & CI Constraints

RALPH_TODO: What OS/platforms are supported? What OS does CI run on? If there’s a mismatch, what is the strategy (conditional compilation/fakes/manual smoke tests)?

## Key Components

RALPH_TODO: Bullet list of major modules/services and their responsibilities.

## Data Model / Key Concepts

RALPH_TODO: The nouns that show up everywhere (entities, resources, state).

## Critical Flows

### Flow: RALPH_TODO

- Trigger:
- Steps:
- Outputs / side effects:
- Failure modes:

## Invariants / Constraints (Must Not Break)

RALPH_TODO: List of constraints that future changes must preserve.

## Where Decisions Live

- Prefer decision rationale in **code comments next to the logic**.
- If a decision spans multiple files, add a short note here with pointers (file paths + a sentence).
