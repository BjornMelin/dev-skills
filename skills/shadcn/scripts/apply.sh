#!/usr/bin/env bash
#
# Refresh the `shadcn` skill from upstream, then re-apply the local overlay.
#
# Upstream: https://github.com/shadcn-ui/ui/tree/main/skills/shadcn
#
# Why this exists: upstream ships `npx shadcn@latest` throughout. Bun-first
# repos need `bunx --bun shadcn@latest`, and a plain reinstall silently reverts
# every call site back to npx. Run this script instead of updating by hand so
# the overlay survives each refresh.
#
# The overlay is exactly three edits:
#   1. Bun-first runner   - `npx shadcn@latest` -> `bunx --bun shadcn@latest`
#   2. Runner guidance    - the >**IMPORTANT:** line, rewritten Bun-first
#   3. Frontmatter        - broadened allowed-tools, trigger-style description,
#                           and `user-invocable` dropped so /shadcn works
#
# Everything else is upstream verbatim. If a diff shows up outside those three,
# upstream changed and the overlay may need revisiting.
#
# Usage:  skills/shadcn/scripts/apply.sh

set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT_DIR="$SKILL_DIR/scripts"
RAW="https://raw.githubusercontent.com/shadcn-ui/ui/main/skills/shadcn"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

FILES=(
  SKILL.md
  cli.md
  customization.md
  mcp.md
  registry.md
  rules/base-vs-radix.md
  rules/chat.md
  rules/composition.md
  rules/forms.md
  rules/icons.md
  rules/styling.md
  evals/evals.json
  assets/shadcn.png
  assets/shadcn-small.png
)

echo "==> Fetching upstream (${#FILES[@]} files)"
for f in "${FILES[@]}"; do
  mkdir -p "$TMP/$(dirname "$f")"
  curl -sSfL -o "$TMP/$f" "$RAW/$f"
done

echo "==> Installing into $SKILL_DIR"
for f in "${FILES[@]}"; do
  mkdir -p "$SKILL_DIR/$(dirname "$f")"
  cp "$TMP/$f" "$SKILL_DIR/$f"
done

# NOTE: agents/openai.yaml is deliberately NOT refreshed from upstream. Upstream
# ships agents/openai.yml without a `default_prompt`, which this repo's
# openai-agent-metadata gate requires. The local file is the maintained one.

# --- Overlay 2: runner guidance, and Overlay 3: frontmatter -------------------
# Done before the blind npx->bunx pass, because the upstream IMPORTANT line
# lists all three runners and a blind substitution would corrupt it into
# "`bunx --bun ...`, `pnpm dlx ...`, or `bunx --bun ...`".
echo "==> Applying overlay: frontmatter + runner guidance"
awk -v fmfile="$SCRIPT_DIR/frontmatter.txt" -v impfile="$SCRIPT_DIR/important.txt" '
  BEGIN {
    while ((getline line < fmfile) > 0) fm[++nfm] = line
    while ((getline line < impfile) > 0) imp[++nimp] = line
  }
  NR == 1 && $0 == "---" { for (i = 1; i <= nfm; i++) print fm[i]; infm = 1; next }
  infm && $0 == "---"    { infm = 0; next }
  infm                   { next }
  /^> \*\*IMPORTANT:\*\*/ { for (i = 1; i <= nimp; i++) print imp[i]; next }
  { print }
' "$SKILL_DIR/SKILL.md" > "$TMP/SKILL.md.overlaid"
mv "$TMP/SKILL.md.overlaid" "$SKILL_DIR/SKILL.md"

# --- Overlay 1: Bun-first runner ---------------------------------------------
# The frontmatter allowed-tools line deliberately keeps an `npx shadcn@latest`
# entry, so it is restored afterwards from frontmatter.txt.
echo "==> Applying overlay: Bun-first runner"
find "$SKILL_DIR" -name '*.md' -type f -not -path "$SCRIPT_DIR/*" \
  -exec sed -i 's|npx shadcn@latest|bunx --bun shadcn@latest|g' {} +

# Restore the allowed-tools line clobbered by the pass above.
allowed="$(grep '^allowed-tools:' "$SCRIPT_DIR/frontmatter.txt")"
awk -v allowed="$allowed" '
  /^allowed-tools:/ && !done { print allowed; done = 1; next }
  { print }
' "$SKILL_DIR/SKILL.md" > "$TMP/SKILL.md.fixed"
mv "$TMP/SKILL.md.fixed" "$SKILL_DIR/SKILL.md"

# --- Verify -------------------------------------------------------------------
echo "==> Verifying"
fail=0
# `grep` exits 1 on no-match, which `set -o pipefail` would turn into a script
# abort, so both counts are guarded with `|| true`.
bunx_count=$( { grep -ro 'bunx --bun shadcn@latest' "$SKILL_DIR" --include='*.md' || true; } | wc -l)
stray_npx=$( { grep -rn 'npx shadcn@latest' "$SKILL_DIR" --include='*.md' || true; } | { grep -v 'allowed-tools:' || true; } | wc -l)

[ "$bunx_count" -gt 0 ] || { echo "FAIL: no bunx runner found"; fail=1; }
[ "$stray_npx" -eq 0 ] || { echo "FAIL: $stray_npx stray npx outside allowed-tools"; fail=1; }
if grep -q '^user-invocable:' "$SKILL_DIR/SKILL.md"; then echo "FAIL: user-invocable survived"; fail=1; fi
grep -q 'Default shadcn runner' "$SKILL_DIR/SKILL.md" || { echo "FAIL: IMPORTANT line missing"; fail=1; }
grep -q '^name: shadcn$' "$SKILL_DIR/SKILL.md" || { echo "FAIL: frontmatter name missing"; fail=1; }

if [ "$fail" -eq 0 ]; then
  echo "OK: upstream refreshed, overlay applied ($bunx_count bunx call sites)"
else
  echo "Overlay verification FAILED - inspect $SKILL_DIR before use" >&2
  exit 1
fi
