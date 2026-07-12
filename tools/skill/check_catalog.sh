#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
catalog_path="$repo_root/catalog/agent-skills-lab.json"
source_commit="${1:-$(git -C "$repo_root" rev-parse HEAD)}"
scan_root="${2:-$repo_root}"
generated_path="$(mktemp)"
tracked_normalized_path="$(mktemp)"
generated_normalized_path="$(mktemp)"
trap 'rm -f "$generated_path" "$tracked_normalized_path" "$generated_normalized_path"' EXIT

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
