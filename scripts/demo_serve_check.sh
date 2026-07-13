#!/usr/bin/env bash
# Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
# SPDX-License-Identifier: Apache-2.0
#
# Serve the static demo exactly as a user would (`python3 -m http.server` from
# demo/, no build step) and assert every referenced asset returns HTTP 200.
# Fails if the demo is unbuilt (pkg/ or pack/ missing) or any path 404s.
#
# Usage:  scripts/demo_serve_check.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DEMO_DIR="${REPO_ROOT}/demo"

# Every path the page references at runtime.
ASSETS=(
  "/"
  "/index.html"
  "/styles.css"
  "/app.js"
  "/sw.js"
  "/src/viewer.js"
  "/src/controls.js"
  "/src/badges.js"
  "/vendor/three.module.min.js"
  "/vendor/OrbitControls.js"
  "/pkg/oxihuman_wasm.js"
  "/pkg/oxihuman_wasm_bg.wasm"
  "/pack/oxihuman-core-v1.ohpk"
)

for f in pkg/oxihuman_wasm.js pkg/oxihuman_wasm_bg.wasm pack/oxihuman-core-v1.ohpk; do
  if [ ! -f "${DEMO_DIR}/${f}" ]; then
    echo "ERROR: ${DEMO_DIR}/${f} is missing. Run scripts/build_demo.sh first." >&2
    exit 2
  fi
done

# Pick a free ephemeral port.
PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()')"
BASE="http://127.0.0.1:${PORT}"

echo "==> serving ${DEMO_DIR} at ${BASE}"
python3 -m http.server "${PORT}" --bind 127.0.0.1 --directory "${DEMO_DIR}" >/dev/null 2>&1 &
SERVER_PID=$!
cleanup() { kill "${SERVER_PID}" >/dev/null 2>&1 || true; }
trap cleanup EXIT

# Wait for the server to accept connections.
for _ in $(seq 1 50); do
  if curl -fsS -o /dev/null "${BASE}/index.html" 2>/dev/null; then break; fi
  sleep 0.1
done

fail=0
for path in "${ASSETS[@]}"; do
  code="$(curl -s -o /dev/null -w '%{http_code}' "${BASE}${path}" || echo 000)"
  if [ "${code}" = "200" ]; then
    printf "  200  %s\n" "${path}"
  else
    printf "  %s  %s   <-- FAIL\n" "${code}" "${path}"
    fail=1
  fi
done

if [ "${fail}" -eq 0 ]; then
  echo "==> OK: all $((${#ASSETS[@]})) assets served 200"
else
  echo "==> FAIL: one or more assets did not return 200" >&2
fi
exit "${fail}"
