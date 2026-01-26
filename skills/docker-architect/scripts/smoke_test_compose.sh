#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Smoke-test a docker compose environment locally.

Usage:
  smoke_test_compose.sh -f docker-compose.yml [-f docker-compose.dev.yml] [--project myproj]
                       [--service app] [--wait-seconds 30]

Performs:
  - docker compose config (validation)
  - docker compose up -d --build
  - optional health wait for a service if it defines a healthcheck
EOF
}

files=()
project="docker-architect-smoke"
service=""
wait_seconds="30"

while [[ $# -gt 0 ]]; do
  case "$1" in
    -f|--file) files+=("$2"); shift 2 ;;
    --project) project="$2"; shift 2 ;;
    --service) service="$2"; shift 2 ;;
    --wait-seconds) wait_seconds="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

if [[ ${#files[@]} -eq 0 ]]; then
  echo "At least one -f/--file is required" >&2
  usage
  exit 2
fi

args=(--project-name "$project")
for f in "${files[@]}"; do
  args+=(-f "$f")
done

echo "Validating compose config..."
docker compose "${args[@]}" config >/dev/null

echo "Bringing up services..."
docker compose "${args[@]}" up -d --build

cleanup() {
  docker compose "${args[@]}" down -v --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT

if [[ -n "$service" ]]; then
  echo "Waiting for service health: $service (up to ${wait_seconds}s)"
  deadline=$(( $(date +%s) + wait_seconds ))
  while true; do
    cid="$(docker compose "${args[@]}" ps -q "$service" | head -n1 || true)"
    if [[ -n "$cid" ]]; then
      health="$(docker inspect -f '{{.State.Health.Status}}' "$cid" 2>/dev/null || true)"
      state="$(docker inspect -f '{{.State.Status}}' "$cid" 2>/dev/null || true)"
      if [[ "$health" == "healthy" ]]; then
        break
      fi
      if [[ -z "$health" && "$state" == "running" ]]; then
        # No healthcheck defined.
        break
      fi
    fi
    if [[ $(date +%s) -ge $deadline ]]; then
      echo "Timed out waiting for $service (state=$state health=$health)" >&2
      exit 1
    fi
    sleep 1
  done
fi

echo "OK: compose is up"
