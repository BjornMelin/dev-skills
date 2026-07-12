#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
catalog_path="$repo_root/catalog/agent-skills-lab.json"
source_commit="$(git -C "$repo_root" rev-parse --verify "${1:-HEAD}^{commit}")"
scan_root="${2:-}"
generated_path="$(mktemp)"
tracked_normalized_path="$(mktemp)"
generated_normalized_path="$(mktemp)"
temporary_worktree_parent=""

cleanup() {
  local status=$?
  rm -f "$generated_path" "$tracked_normalized_path" "$generated_normalized_path"
  if [[ -n "$temporary_worktree_parent" ]]; then
    git -C "$repo_root" worktree remove --force "$scan_root" >/dev/null 2>&1 || true
    rmdir "$temporary_worktree_parent" >/dev/null 2>&1 || true
  fi
  exit "$status"
}
trap cleanup EXIT

if [[ -z "$scan_root" ]]; then
  head_commit="$(git -C "$repo_root" rev-parse --verify 'HEAD^{commit}')"
  if [[ "$source_commit" != "$head_commit" ]]; then
    echo "implicit catalog scan requires source_commit to match HEAD; pass an explicit scan root for another commit" >&2
    exit 2
  fi
  temporary_worktree_parent="$(mktemp -d)"
  scan_root="$temporary_worktree_parent/repo"
  git -C "$repo_root" worktree add --detach "$scan_root" "$source_commit" >/dev/null
  if ! git -C "$repo_root" diff --quiet "$source_commit" --; then
    git -C "$repo_root" diff --binary "$source_commit" -- |
      git -C "$scan_root" apply --whitespace=nowarn
  fi
  while IFS= read -r -d '' relative_path; do
    source_path="$repo_root/$relative_path"
    destination_path="$scan_root/$relative_path"
    mkdir -p "$(dirname "$destination_path")"
    cp -pP "$source_path" "$destination_path"
  done < <(git -C "$repo_root" ls-files --others --exclude-standard -z)
fi

jq -e '
  .schemaVersion == "agent_skills_lab_catalog.v1"
  and (.sourceCommit | test("^[0-9a-f]{7,40}$"; "i"))
  and .sourceRef == "main"
' "$catalog_path" >/dev/null

catalog_generated_at="$(jq -er '.generatedAt' "$catalog_path")"
catalog_source_commit="$(jq -er '.sourceCommit' "$catalog_path")"

cargo run -q -p codex-dev -- --json skills catalog \
  --repo-root "$scan_root" \
  --generated-at "$catalog_generated_at" \
  --source-commit "$source_commit" \
  --source-ref main \
  --out "$generated_path" \
  >/dev/null

normalize_catalog() {
  local input_path="$1"
  local output_path="$2"
  jq -S \
    --arg tracked_sha "$catalog_source_commit" \
    --arg current_sha "$source_commit" \
    'walk(if type == "string" then gsub($tracked_sha; "<sourceCommit>") | gsub($current_sha; "<sourceCommit>") else . end)' \
    "$input_path" >"$output_path"
}

normalize_catalog "$catalog_path" "$tracked_normalized_path"
normalize_catalog "$generated_path" "$generated_normalized_path"
diff -u "$tracked_normalized_path" "$generated_normalized_path"
echo "agent-skills-catalog-ok"
