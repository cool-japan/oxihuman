#!/usr/bin/env bash
# Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
# SPDX-License-Identifier: Apache-2.0
#
# Build the static BodyLab demo payload:
#   1. Compile the oxihuman-wasm crate to a browser (`--target web`) package
#      into demo/pkg/  (JS glue + optimised .wasm).
#   2. Copy the real OHPK core pack into demo/pack/.
#   3. Print raw + gzip transfer sizes of the wasm and the pack.
#
# The result is a fully static site: `python3 -m http.server` from demo/ serves
# it with NO build step at serve time. Re-running this script is idempotent
# (wasm-pack cleans its --out-dir; the pack copy overwrites).
#
# Usage:  scripts/build_demo.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

CRATE_DIR="${REPO_ROOT}/crates/oxihuman-wasm"
OUT_PKG="${REPO_ROOT}/demo/pkg"
PACK_SRC="${REPO_ROOT}/assets/packs/oxihuman-core-v1.ohpk"
PACK_DST_DIR="${REPO_ROOT}/demo/pack"
PACK_DST="${PACK_DST_DIR}/oxihuman-core-v1.ohpk"

echo "==> OxiHuman BodyLab demo build"
echo "    repo root : ${REPO_ROOT}"
echo "    wasm out  : ${OUT_PKG}"

command -v wasm-pack >/dev/null 2>&1 || {
  echo "ERROR: wasm-pack not found. Install it: cargo install wasm-pack" >&2
  exit 1
}

# 1. Build the browser WASM package (bindgen feature = full JS/TS surface).
echo "==> wasm-pack build (--target web, --features bindgen) ..."
# wasm-pack options (--out-dir, --target, --release) precede the crate PATH;
# everything after the PATH is forwarded to `cargo build`.
wasm-pack build --release --target web --out-dir "${OUT_PKG}" "${CRATE_DIR}" \
  --no-default-features --features bindgen

WASM_FILE="${OUT_PKG}/oxihuman_wasm_bg.wasm"
JS_FILE="${OUT_PKG}/oxihuman_wasm.js"
[ -f "${WASM_FILE}" ] || { echo "ERROR: expected ${WASM_FILE} to exist" >&2; exit 1; }
[ -f "${JS_FILE}" ]   || { echo "ERROR: expected ${JS_FILE} to exist"   >&2; exit 1; }

# 2. Copy the real core pack next to the page.
echo "==> copying core pack ..."
[ -f "${PACK_SRC}" ] || {
  echo "ERROR: core pack not found at ${PACK_SRC}" >&2
  echo "       Build it first: cargo run -p oxihuman-cli -- pack-core ..." >&2
  exit 1
}
mkdir -p "${PACK_DST_DIR}"
cp -f "${PACK_SRC}" "${PACK_DST}"

# 3. Report raw + gzip transfer sizes (what a browser actually downloads).
gz_size() { gzip -9 -c "$1" | wc -c | tr -d ' '; }
raw_size() { wc -c < "$1" | tr -d ' '; }
human() { awk -v b="$1" 'BEGIN{
  split("B KB MB GB", u, " "); i=1;
  while (b >= 1024 && i < 4) { b /= 1024; i++ }
  printf (i==1 ? "%d %s" : "%.2f %s"), b, u[i]
}'; }

WASM_RAW=$(raw_size "${WASM_FILE}"); WASM_GZ=$(gz_size "${WASM_FILE}")
PACK_RAW=$(raw_size "${PACK_DST}"); PACK_GZ=$(gz_size "${PACK_DST}")

echo ""
echo "==> transfer sizes (raw / gzip -9)"
printf "    %-28s %12s  %12s\n" "artifact" "raw" "gzip"
printf "    %-28s %12s  %12s\n" "oxihuman_wasm_bg.wasm" "$(human "${WASM_RAW}")" "$(human "${WASM_GZ}")"
printf "    %-28s %12s  %12s\n" "oxihuman-core-v1.ohpk" "$(human "${PACK_RAW}")" "$(human "${PACK_GZ}")"
echo ""
echo "    raw bytes  : wasm=${WASM_RAW} pack=${PACK_RAW}"
echo "    gzip bytes : wasm=${WASM_GZ} pack=${PACK_GZ}"
echo ""
echo "==> done. Serve with:  ( cd demo && python3 -m http.server 8000 )"
