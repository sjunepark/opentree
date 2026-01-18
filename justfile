set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

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

# Run investigation tests (ignored by default, require Codex CLI)
# Usage: just investigate [filter]
#   just investigate        - run all
#   just investigate codex  - run codex-related tests
investigate filter="":
  cargo test -p runner --test investigation {{filter}} -- --ignored

fmt: mdfmt rustfmt
check: mdcheck rustfmt-check clippy test
ci: check

eval-list:
  RUST_LOG=eval=info cargo run -p eval -- list

eval-run CASE:
  RUST_LOG=eval=info cargo run -p eval -- run {{CASE}}

eval-run-debug CASE:
  RUST_LOG=eval=debug cargo run -p eval -- run {{CASE}}

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

# Run both UI server and Vite dev server (for development)
ui-dev-full PROJECT_DIR=".":
  @echo "Start backend: just ui-server {{PROJECT_DIR}}"
  @echo "Start frontend: just ui-dev"
  @echo "Then open http://localhost:5173"
