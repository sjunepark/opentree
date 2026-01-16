set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
  @just --list

mdfmt:
  rumdl fmt .

mdcheck:
  rumdl check .

fmt: mdfmt
check: mdcheck
ci: mdcheck
