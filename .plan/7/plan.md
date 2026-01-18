---
status: complete
created_at: 2026-01-18T04:47:16Z
title: "Evaluation framework: eval crate + cases"
---

# Plan 7

## Objective

Implement the `eval/` framework to run the real deterministic `runner` loop (built from this repo)
against declarative cases, capture artifacts, judge success, and report aggregate stats across runs.

## Success Criteria

- [x] Add `eval/` crate as a workspace member with an `eval` CLI (`list`, `run`, `report`, `clean`).
- [x] Support declarative case files under `eval/cases/*.toml` (goal, config overrides, checks).
- [x] `eval run <case-id>` creates an isolated git workspace under `eval/workspaces/` and can run:
      `runner start` then `runner loop` as subprocesses.
- [x] Guard strategy works with the runner default (`just ci`) in workspaces (no “always-fail” guards).
- [x] Persist run artifacts under `eval/results/<case-id>/<run-id>/` (meta + tree/run_state + checks +
      copied `.runner/iterations/` logs).
- [x] Judge checks deterministically (at least: `file_exists`, `command_succeeds`, `runner_completed`).
- [x] `eval report <case-id>` aggregates stats across runs (success rate, avg iterations/time, per-check
      pass rate).
- [x] `eval clean <case-id>` deletes `eval/workspaces/` + `eval/results/` for the case.
- [x] `just ci` passes.

## Sub-plan Index

- a/ — Case schema + guard strategy
- b/ — Workspace harness + runner subprocess
- c/ — Judge + results layout
- d/ — CLI + reporting + docs

## Unresolved Questions

- Should the workspace `justfile` be generated from `checks` (default) or required explicitly per case?
- When toolchains are missing (ex: `go`), should a run be `failed` or `skipped`?
- Should `eval run --runs N` create N workspaces or reuse/reset one workspace per run?

## Plan Summary

- Implemented `eval` crate with case schema, workspace harness, subprocess runner execution, results capture, checks, outcomes, reporting, and CLI.
- Added example case, docs, and `just` helpers for evaluation workflows.
