#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Smoke-test a single Dockerfile build/run locally.

Usage:
  smoke_test_container.sh --tag myapp:smoke [--dockerfile Dockerfile] [--context .]
                         [--build-check] [--pull]
                         [--host-port 8000 --container-port 8000 --health-path /healthz]
                         [--env KEY=VALUE ...]

If --health-path is set, performs an HTTP GET on http://localhost:${host-port}${health-path}.
EOF
}

tag=""
dockerfile="Dockerfile"
context="."
build_check="false"
pull="false"
host_port=""
container_port=""
health_path=""
envs=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag) tag="$2"; shift 2 ;;
    --dockerfile) dockerfile="$2"; shift 2 ;;
    --context) context="$2"; shift 2 ;;
    --build-check) build_check="true"; shift 1 ;;
    --pull) pull="true"; shift 1 ;;
    --host-port) host_port="$2"; shift 2 ;;
    --container-port) container_port="$2"; shift 2 ;;
    --health-path) health_path="$2"; shift 2 ;;
    --env) envs+=("$2"); shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

if [[ -z "$tag" ]]; then
  echo "Missing --tag" >&2
  usage
  exit 2
fi

if [[ "$build_check" == "true" ]]; then
  if docker build --help 2>/dev/null | grep -q -- '--check' ; then
    echo "Running Dockerfile build checks..."
    docker build --check -f "$dockerfile" "$context"
  else
    echo "docker build --check not supported by this Docker version; skipping" >&2
  fi
fi

pull_flag=()
if [[ "$pull" == "true" ]]; then
  pull_flag=(--pull)
fi

echo "Building $tag ..."
docker build "${pull_flag[@]}" -f "$dockerfile" -t "$tag" "$context"

run_args=(--rm -d)
if [[ -n "$host_port" && -n "$container_port" ]]; then
  run_args+=(-p "${host_port}:${container_port}")
fi
for kv in "${envs[@]}"; do
  run_args+=(-e "$kv")
done

echo "Running container ..."
cid="$(docker run "${run_args[@]}" "$tag")"
cleanup() {
  docker stop "$cid" >/dev/null 2>&1 || true
}
trap cleanup EXIT

sleep 2

if [[ -n "$health_path" && -n "$host_port" ]]; then
  url="http://localhost:${host_port}${health_path}"
  echo "Checking health: $url"
  python3 - <<PY
import sys
import time
import urllib.error
import urllib.request

url = ${url@Q}
deadline = time.time() + 30
last_err = None
while time.time() < deadline:
    try:
        with urllib.request.urlopen(url, timeout=2) as r:
            if 200 <= r.status < 400:
                sys.exit(0)
    except Exception as e:
        last_err = e
    time.sleep(1)
print(f"Health check failed: {last_err}", file=sys.stderr)
sys.exit(1)
PY
fi

echo "OK: container started"
