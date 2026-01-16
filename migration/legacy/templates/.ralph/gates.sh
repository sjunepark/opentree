#!/usr/bin/env bash
set -euo pipefail

log() {
  printf '[gates] %s\n' "$*" >&2
}

run() {
  log "$*"
  "$@"
}

has_file() {
  [[ -f "$1" ]]
}

require_architecture_doc() {
  local path=".ralph/architecture.md"
  local max_lines="${RALPH_ARCH_MAX_LINES:-250}"
  local min_content_lines="${RALPH_ARCH_MIN_CONTENT_LINES:-5}"

  if [[ ! -f "${path}" ]]; then
    log "Missing required architecture doc: ${path}"
    log "Create it and remove placeholders (e.g. RALPH_TODO)."
    exit 1
  fi

  if grep -q -E '(RALPH_TODO|REPLACE_ME)' "${path}"; then
    log "Architecture doc still contains placeholders (RALPH_TODO/REPLACE_ME): ${path}"
    exit 1
  fi

  local lines
  lines="$(wc -l <"${path}" | tr -d ' ')"
  if [[ "${lines}" -gt "${max_lines}" ]]; then
    log "Architecture doc too long (${lines} lines > ${max_lines}): ${path}"
    exit 1
  fi

  local content_lines
  content_lines="$(grep -c -v -E '^[[:space:]]*(#|$)' "${path}" || true)"
  if [[ "${content_lines}" -lt "${min_content_lines}" ]]; then
    log "Architecture doc too empty (${content_lines} content lines < ${min_content_lines}): ${path}"
    exit 1
  fi
}

require_prd_platform_constraints() {
  local path=".ralph/prd.json"

  if [[ ! -f "${path}" ]]; then
    log "Missing required PRD file: ${path}"
    exit 1
  fi

  if ! command -v jq >/dev/null 2>&1; then
    log "Missing required command: jq (needed to validate ${path})"
    exit 1
  fi

  if ! jq -e '.userStories | type=="array"' "${path}" >/dev/null 2>&1; then
    log "Invalid PRD JSON (missing/invalid userStories[]): ${path}"
    exit 1
  fi

  if ! jq -e '
      (.platformConstraints | type=="object")
      and (.platformConstraints.supportedOS | type=="array" and length > 0 and all(type=="string" and (gsub("[[:space:]]+"; "")) != ""))
      and (.platformConstraints.ciOS | type=="array" and length > 0 and all(type=="string" and (gsub("[[:space:]]+"; "")) != ""))
      and (.platformConstraints.runtimeDependencies | type=="array")
      and (.platformConstraints.nonSupportedOSBehavior | type=="string")
      and (.platformConstraints.ciMismatchStrategy | type=="string")
    ' "${path}" >/dev/null 2>&1; then
    log "Invalid PRD JSON (missing/invalid platformConstraints.*): ${path}"
    exit 1
  fi

  if ! jq -e '
      (.platformConstraints.supportedOS as $supported | .platformConstraints.ciOS as $ci
        | ([ $ci[] | select($supported | index(.) == null) ] | length) == 0)
      or
      (
        ((.platformConstraints.nonSupportedOSBehavior | gsub("[[:space:]]+"; "")) != "" and (.platformConstraints.nonSupportedOSBehavior | ascii_downcase) != "n/a" and (.platformConstraints.nonSupportedOSBehavior | ascii_downcase) != "null")
        and
        ((.platformConstraints.ciMismatchStrategy | gsub("[[:space:]]+"; "")) != "" and (.platformConstraints.ciMismatchStrategy | ascii_downcase) != "n/a" and (.platformConstraints.ciMismatchStrategy | ascii_downcase) != "null")
      )
    ' "${path}" >/dev/null 2>&1; then
    log "PRD platformConstraints indicate CI OS mismatch, but nonSupportedOSBehavior/ciMismatchStrategy are not specified: ${path}"
    log "Set platformConstraints.nonSupportedOSBehavior and platformConstraints.ciMismatchStrategy to concrete values when ciOS is not a subset of supportedOS."
    exit 1
  fi
}

require_prd_completion_for_passed_stories() {
  local path=".ralph/prd.json"

  if [[ ! -f "${path}" ]]; then
    log "Missing required PRD file: ${path}"
    exit 1
  fi

  if ! command -v jq >/dev/null 2>&1; then
    log "Missing required command: jq (needed to validate ${path})"
    exit 1
  fi

  if ! jq -e '.userStories | type=="array"' "${path}" >/dev/null 2>&1; then
    log "Invalid PRD JSON (missing/invalid userStories[]): ${path}"
    exit 1
  fi

  local missing_ids
  missing_ids="$(
    jq -r '
      .userStories[]
      | select(.passes == true)
      | select(
          (.completion | type != "object")
          or (.completion.summary | type != "string")
          or ((.completion.summary | gsub("[[:space:]]+"; "")) == "")
          or (.completion.retro | type != "object")
          or (
            [
              .completion.retro.difficulties,
              .completion.retro.prdWeaknesses,
              .completion.retro.ralphImprovements
            ]
            | any((type != "string") or ((gsub("[[:space:]]+"; "")) == ""))
          )
          or (.completion.scopeAppropriate | type != "string")
          or ((.completion.scopeAppropriate | gsub("[[:space:]]+"; "")) == "")
          or (.completion.autoCompactOccurred | type != "boolean")
        )
      | (.id // "<missing id>")
    ' "${path}"
  )"

  if [[ -n "${missing_ids}" ]]; then
    log "PRD completion fields are required when passes=true. Missing/empty completion for:"
    echo "${missing_ids}" | sed 's/^/ - /' >&2
    exit 1
  fi
}

require_smoke_test_checklists_for_passed_stories() {
  local path=".ralph/prd.json"

  if [[ ! -f "${path}" ]]; then
    log "Missing required PRD file: ${path}"
    exit 1
  fi

  if ! command -v jq >/dev/null 2>&1; then
    log "Missing required command: jq (needed to validate ${path})"
    exit 1
  fi

  local missing_ids
  missing_ids="$(
    jq -r '
      .userStories[]
      | select(.passes == true)
      | select(
          (
            any((.acceptanceCriteria // [])[]?; (type=="string") and test("manual validation|manual smoke|smoke test"; "i"))
          )
          and
          (
            (.manualSmokeTestChecklist | type != "string")
            or
            ((.manualSmokeTestChecklist | gsub("[[:space:]]+"; "")) == "")
          )
        )
      | (.id // "<missing id>")
    ' "${path}"
  )"

  if [[ -n "${missing_ids}" ]]; then
    log "Passed stories with manual validation must set manualSmokeTestChecklist (e.g., .ralph/smoke-tests/US-012.md). Missing for:"
    echo "${missing_ids}" | sed 's/^/ - /' >&2
    exit 1
  fi

  local checklist_paths
  checklist_paths="$(
    jq -r '
      .userStories[]
      | select(.passes == true)
      | (.manualSmokeTestChecklist // empty)
      | select(type=="string")
      | gsub("[[:space:]]+"; "")
      | select(. != "")
    ' "${path}"
  )"

  local checklist_path
  while IFS= read -r checklist_path; do
    [[ -n "${checklist_path}" ]] || continue
    if [[ ! -f "${checklist_path}" ]]; then
      log "Missing manual smoke-test checklist file referenced by PRD: ${checklist_path}"
      exit 1
    fi
    if grep -q -E '(RALPH_TODO|REPLACE_ME)' "${checklist_path}"; then
      log "Smoke-test checklist contains placeholders (RALPH_TODO/REPLACE_ME): ${checklist_path}"
      exit 1
    fi
    content_lines="$(grep -c -v -E '^[[:space:]]*(#|$)' "${checklist_path}" || true)"
    if [[ "${content_lines}" -lt 5 ]]; then
      log "Smoke-test checklist too empty (${content_lines} content lines < 5): ${checklist_path}"
      exit 1
    fi
  done <<<"${checklist_paths}"
}

require_architecture_doc
require_prd_platform_constraints
require_prd_completion_for_passed_stories
require_smoke_test_checklists_for_passed_stories

if command -v just >/dev/null 2>&1 && { has_file justfile || has_file Justfile; }; then
  recipes="$(just --summary 2>/dev/null || true)"
  if echo "${recipes}" | tr ' ' '\n' | grep -qx "ci"; then
    run just ci
    exit 0
  fi
  if echo "${recipes}" | tr ' ' '\n' | grep -qx "check"; then
    run just check
    exit 0
  fi
  if echo "${recipes}" | tr ' ' '\n' | grep -qx "test"; then
    run just test
    exit 0
  fi
fi

if has_file package.json; then
  pm="npm"
  install_cmd=(npm install)
  if has_file package-lock.json; then
    install_cmd=(npm ci)
  fi

  if has_file pnpm-lock.yaml; then
    pm="pnpm"
    install_cmd=(pnpm install --frozen-lockfile)
  elif has_file yarn.lock; then
    pm="yarn"
    install_cmd=(yarn install --immutable)
  fi
  if [[ "${pm}" == "yarn" ]]; then
    log "yarn install (trying --immutable, then --frozen-lockfile)"
    if ! yarn install --immutable; then
      run yarn install --frozen-lockfile
    fi
  else
    run "${install_cmd[@]}"
  fi

  scripts="$(node -e 'const s=require("./package.json").scripts||{};console.log(Object.keys(s).join("\n"))')"

  if echo "${scripts}" | grep -qx "typecheck"; then
    run "${pm}" run typecheck
  fi
  if echo "${scripts}" | grep -qx "lint"; then
    run "${pm}" run lint
  fi
  if echo "${scripts}" | grep -qx "test"; then
    run "${pm}" run test
  fi

  log "OK"
  exit 0
fi

if has_file Cargo.toml; then
  run cargo fmt --check
  run cargo clippy --all-targets --all-features -- -D warnings
  run cargo test
  log "OK"
  exit 0
fi

if has_file go.mod; then
  run go test ./...
  log "OK"
  exit 0
fi

log "No recognized project type. Customize .ralph/gates.sh for this repo."
exit 1
