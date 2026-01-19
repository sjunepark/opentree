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

doc:
  cargo clean --doc && cargo doc --workspace --no-deps --open

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
#   just investigate-llm decomposer  - run only decomposer tests
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
  cargo run -p eval -- list

eval-run CASE:
  cargo run -p eval -- run {{CASE}}

eval-continue CASE:
  cargo run -p eval -- run {{CASE}} --continue

eval-clean CASE:
  cargo run -p eval -- clean {{CASE}}

# Runner UI commands
ui-install:
  cd ui && bun install

ui-dev:
  cd ui && bun run dev

ui-build:
  cd ui && bun run build

# Run UI backend server
# Usage: just ui-eval calculator-go         (expands to eval/workspaces/calculator-go_latest)
#        just ui-eval ./my-project          (uses path as-is)
ui-eval PROJECT=".":
  #!/usr/bin/env bash
  set -euo pipefail
  PROJECT_PATH="{{PROJECT}}"
  [[ ! "$PROJECT_PATH" == *"/"* ]] && PROJECT_PATH="eval/workspaces/${PROJECT_PATH}_latest"
  cargo run -p runner-ui -- --project-dir "$PROJECT_PATH"

# Prompt Lab commands
lab-list AGENT="decomposer":
  cargo run -p prompt_lab -- list {{AGENT}}

lab-run AGENT="decomposer" *FLAGS:
  RUST_LOG=prompt_lab=info cargo run -p prompt_lab -- run {{AGENT}} {{FLAGS}}

lab-dashboard:
  cd runner/prompt_lab/dashboard && bun run dev

lab-build:
  cd runner/prompt_lab/dashboard && bun run build

lab-install:
  cd runner/prompt_lab/dashboard && bun install

# Run eval with UI monitoring (runs eval, backend, and frontend together)
# Usage: just eval-ui calculator-go
#        just eval-ui calculator-go --continue
#        Then open http://localhost:5173
eval-ui CASE *FLAGS:
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
  cargo run -p eval -- run {{CASE}} {{FLAGS}} &
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
    cargo run -p runner-ui -- --project-dir "$WORKSPACE_LINK" &
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
