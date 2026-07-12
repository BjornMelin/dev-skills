#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
checker="$repo_root/tools/skill/check_catalog.sh"
fixture_dir="$repo_root/skills/bun-dev/references"
fixture=""
failure_output="$(mktemp)"

cleanup() {
  if [[ -n "$fixture" ]]; then
    rm -f -- "$fixture"
  fi
  rm -f -- "$failure_output"
}
trap cleanup EXIT

worktrees_before="$(git -C "$repo_root" worktree list --porcelain)"

assert_worktrees_cleaned() {
  local worktrees_after
  worktrees_after="$(git -C "$repo_root" worktree list --porcelain)"
  if [[ "$worktrees_after" != "$worktrees_before" ]]; then
    echo "catalog checker leaked a temporary worktree" >&2
    diff -u <(printf '%s\n' "$worktrees_before") <(printf '%s\n' "$worktrees_after") >&2 || true
    exit 1
  fi
}

previous_commit="$(git -C "$repo_root" rev-parse --verify 'HEAD^')"
if bash "$checker" "$previous_commit" >"$failure_output" 2>&1; then
  echo "catalog checker accepted a non-HEAD implicit source commit" >&2
  exit 1
fi
if ! grep -q "source_commit to match HEAD" "$failure_output"; then
  echo "catalog checker returned the wrong non-HEAD diagnostic" >&2
  cat "$failure_output" >&2
  exit 1
fi
assert_worktrees_cleaned

bash "$checker" >/dev/null
assert_worktrees_cleaned

fixture="$(mktemp "$fixture_dir/catalog-check-untracked-fixture.md.XXXXXX")"
printf '# Catalog checker untracked-file fixture\n' >"$fixture"
if bash "$checker" >"$failure_output" 2>&1; then
  echo "catalog checker ignored a non-ignored untracked resource" >&2
  exit 1
fi
assert_worktrees_cleaned

rm -f "$fixture"
fixture=""
bash "$checker" >/dev/null
assert_worktrees_cleaned

echo "agent-skills-catalog-checker-tests-ok"
