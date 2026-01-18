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
  cargo run -p eval -- list

eval-run CASE:
  cargo run -p eval -- run {{CASE}}

eval-report CASE:
  cargo run -p eval -- report {{CASE}}

eval-clean CASE:
  cargo run -p eval -- clean {{CASE}}
