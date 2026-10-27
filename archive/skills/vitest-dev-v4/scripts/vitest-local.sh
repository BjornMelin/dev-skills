#!/usr/bin/env bash
set -euo pipefail

# Local runner for Vitest.
# - Default: watch mode (TTY) using `vitest`
# - Optional UI: set VITEST_UI=1

if [[ "${VITEST_UI:-}" == "1" ]]; then
  npx vitest --ui
else
  npx vitest
fi
