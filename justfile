set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
set dotenv-load
set dotenv-filename := ".env.development"

default:
  @just --list

mdfmt:
  rumdl fmt .

mdcheck:
  rumdl check .

rustfmt:
  cargo fmt --all

rustfmt-check:
  cargo fmt --all -- --check

clippy:
  cargo clippy --workspace --all-targets -- -D warnings

test:
  cargo test --workspace

# Run UI tests
# Usage: just test-ui [filter]
#   just test-ui               - run all UI tests
#   just test-ui tree-utils    - run tree-utils tests
#   just test-ui AncestryTree  - run AncestryTreeView tests
test-ui filter="":
  cd ui && bun run test -- {{filter}}

# Run UI tests with browser UI
test-ui-browser:
  cd ui && bun run test:ui

# Run investigation tests (ignored by default, require external deps)
# TEST_LOG=1 in .env enables tracing; --nocapture shows output for passing tests
#
# LLM/Codex:
# Usage: just investigate-llm [filter]
#   just investigate-llm             - run all (ignored) LLM tests
#   just investigate-llm tree_agent  - run only tree-agent tests
#   just investigate-llm codex       - run only codex CLI tests
#
# DB:
# Usage: just investigate-db [filter]
#   just investigate-db              - run all (ignored) DB tests
investigate-llm filter="":
  cargo test -p runner --test investigation_llm {{filter}} -- --ignored --nocapture
investigate-db filter="":
  cargo test -p runner --test investigation_db {{filter}} -- --ignored --nocapture

# Back-compat alias (runs LLM investigation tests)
investigate filter="":
  cargo test -p runner --test investigation_llm {{filter}} -- --ignored --nocapture

fmt: mdfmt rustfmt
check: mdcheck rustfmt-check clippy test
ci: check

eval-list:
  RUST_LOG=eval=info cargo run -p eval -- list

eval-run CASE:
  RUST_LOG=eval=info cargo run -p eval -- run {{CASE}}

eval-run-debug CASE:
  RUST_LOG=eval=debug cargo run -p eval -- run {{CASE}}

eval-continue CASE:
  RUST_LOG=eval=info cargo run -p eval -- run {{CASE}} --continue

eval-continue-debug CASE:
  RUST_LOG=eval=debug cargo run -p eval -- run {{CASE}} --continue

eval-report CASE:
  RUST_LOG=eval=info cargo run -p eval -- report {{CASE}}

eval-clean CASE:
  RUST_LOG=eval=info cargo run -p eval -- clean {{CASE}}

# Runner UI commands
ui-install:
  cd ui && bun install

ui-dev:
  cd ui && bun run dev

ui-build:
  cd ui && bun run build

ui-server PROJECT_DIR=".":
  RUST_LOG=runner_ui=info cargo run -p runner-ui -- --project-dir {{PROJECT_DIR}}

# Run eval with UI monitoring (runs eval, backend, and frontend together)
# Usage: just eval-with-ui calculator-go
#        just eval-with-ui calculator-go --continue
#        Then open http://localhost:5173
eval-with-ui CASE *FLAGS:
  #!/usr/bin/env bash
  set -euo pipefail
  WORKSPACE_LINK="eval/workspaces/{{CASE}}_latest"
  CONTINUE_MODE=false
  [[ "{{FLAGS}}" == *"--continue"* || "{{FLAGS}}" == *"-c"* ]] && CONTINUE_MODE=true
  if $CONTINUE_MODE && [[ ! -L "$WORKSPACE_LINK" ]]; then
    echo "Error: no latest workspace for {{CASE}}"
    exit 1
  fi
  EVAL_PID=""
  SERVER_PID=""
  cleanup() {
    [[ -n "$EVAL_PID" ]] && kill "$EVAL_PID" 2>/dev/null || true
    [[ -n "$SERVER_PID" ]] && kill "$SERVER_PID" 2>/dev/null || true
  }
  trap cleanup EXIT
  if $CONTINUE_MODE; then
    echo "Continuing eval for {{CASE}} in background..."
  else
    echo "Starting eval for {{CASE}} in background..."
  fi
  RUST_LOG=eval=info cargo run -p eval -- run {{CASE}} {{FLAGS}} &
  EVAL_PID=$!
  if ! $CONTINUE_MODE; then
    sleep 2
    if [[ ! -L "$WORKSPACE_LINK" ]]; then
      echo "Waiting for workspace symlink..."
      for i in {1..10}; do
        sleep 1
        [[ -L "$WORKSPACE_LINK" ]] && break
      done
    fi
  fi
  if [[ -L "$WORKSPACE_LINK" ]]; then
    echo ""
    echo "Workspace: $(readlink "$WORKSPACE_LINK")"
    echo "Starting backend server..."
    RUST_LOG=runner_ui=info cargo run -p runner-ui -- --project-dir "$WORKSPACE_LINK" &
    SERVER_PID=$!
    if [[ ! -d "ui/node_modules" ]]; then
      echo "Installing frontend dependencies..."
      (cd ui && bun install)
    fi
    echo "Starting frontend dev server..."
    echo "Open http://localhost:5173"
    echo ""
    cd ui && bun run dev
  else
    echo "Error: workspace symlink not created"
    exit 1
  fi
