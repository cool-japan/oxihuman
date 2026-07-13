#!/usr/bin/env bash
# Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
# SPDX-License-Identifier: Apache-2.0
#
# Print raw + gzip transfer sizes of the demo's shipped WASM and core pack.
# These are the honest "download" numbers the demo badges read at runtime.
#
# Usage:  web/bench/sizes.sh
#         (run scripts/build_demo.sh first so demo/pkg + demo/pack exist)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

gz() { gzip -9 -c "$1" | wc -c | tr -d ' '; }
raw() { wc -c < "$1" | tr -d ' '; }
human() { awk -v b="$1" 'BEGIN{
  split("B KB MB GB", u, " "); i=1;
  while (b >= 1024 && i < 4) { b /= 1024; i++ }
  printf (i==1 ? "%d %s" : "%.2f %s"), b, u[i]
}'; }

emit() {
  local label="$1" file="$2"
  if [ ! -f "${file}" ]; then
    printf "  %-26s %s\n" "${label}" "(missing — run scripts/build_demo.sh)"
    return 1
  fi
  local r g
  r="$(raw "${file}")"; g="$(gz "${file}")"
  printf "  %-26s raw %10s (%d B)   gzip %10s (%d B)\n" \
    "${label}" "$(human "${r}")" "${r}" "$(human "${g}")" "${g}"
}

echo "OxiHuman BodyLab — shipped artefact sizes"
rc=0
for w in "${REPO_ROOT}"/demo/pkg/*.wasm; do
  [ -e "${w}" ] || { echo "  (no demo/pkg/*.wasm — run scripts/build_demo.sh)"; rc=1; break; }
  emit "$(basename "${w}")" "${w}" || rc=1
done
for p in "${REPO_ROOT}"/demo/pack/*.ohpk; do
  [ -e "${p}" ] || { echo "  (no demo/pack/*.ohpk — run scripts/build_demo.sh)"; rc=1; break; }
  emit "$(basename "${p}")" "${p}" || rc=1
done
exit "${rc}"
