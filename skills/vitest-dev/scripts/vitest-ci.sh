#!/usr/bin/env bash
set -euo pipefail

# CI runner for Vitest.
# - Forces single-run mode.
# - Emits JUnit + JSON to ./reports using Vitest's --outputFile.* flags.
#   See: https://vitest.dev/config/outputfile

mkdir -p reports

npx vitest run   --reporter=default   --reporter=junit   --reporter=json   --outputFile.junit=reports/junit.xml   --outputFile.json=reports/results.json
