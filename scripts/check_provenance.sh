#!/usr/bin/env bash
#
# check_provenance.sh
#
# Machine-verifies the claims made in PROVENANCE.md against the actual
# on-disk artifacts:
#
#   1. assets/packs/oxihuman-core-v1.ohpk SHA-256 matches BOTH the hash
#      stated in PROVENANCE.md and the hash recorded in the pack's
#      provenance sidecar (assets/packs/oxihuman-core-v1.provenance.json).
#   2. If assets/upstream/ is present (gitignored; populated by
#      scripts/fetch_upstream_assets.sh), every bundled target's SHA-256
#      (plus base.obj) is re-verified against the sidecar's per-target
#      list. If assets/upstream/ is absent, this step is skipped with a
#      notice, not a failure.
#   3. assets/alpha_pack/oxihuman_assets.toml's core_pack_sha256 matches
#      the actual pack file hash.
#
# Exits non-zero on any mismatch. All paths are repo-relative, derived from
# this script's own location -- no absolute paths are hardcoded.

set -euo pipefail

# --------------------------------------------------------------------------
# Paths (repo-relative, derived from script location)
# --------------------------------------------------------------------------

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." &>/dev/null && pwd)"

PROVENANCE_MD="${REPO_ROOT}/PROVENANCE.md"
PACK_FILE="${REPO_ROOT}/assets/packs/oxihuman-core-v1.ohpk"
PACK_SIDECAR="${REPO_ROOT}/assets/packs/oxihuman-core-v1.provenance.json"
UPSTREAM_MANIFEST="${REPO_ROOT}/assets/upstream/UPSTREAM_MANIFEST.json"
UPSTREAM_CHECKOUT_DIR="${REPO_ROOT}/assets/upstream/makehuman"
ALPHA_PACK_TOML="${REPO_ROOT}/assets/alpha_pack/oxihuman_assets.toml"

FAILURES=0

# --------------------------------------------------------------------------
# Helpers
# --------------------------------------------------------------------------

log_ok() {
  printf '  [OK]   %s\n' "$1"
}

log_fail() {
  printf '  [FAIL] %s\n' "$1" >&2
  FAILURES=$((FAILURES + 1))
}

log_notice() {
  printf '  [SKIP] %s\n' "$1"
}

sha256_of() {
  # Portable SHA-256: prefer shasum (macOS default), fall back to sha256sum.
  if command -v shasum &>/dev/null; then
    shasum -a 256 "$1" | awk '{print $1}'
  elif command -v sha256sum &>/dev/null; then
    sha256sum "$1" | awk '{print $1}'
  else
    echo "ERROR: neither shasum nor sha256sum found on PATH" >&2
    exit 2
  fi
}

require_cmd() {
  if ! command -v "$1" &>/dev/null; then
    echo "ERROR: required command '$1' not found on PATH" >&2
    exit 2
  fi
}

require_cmd jq
require_cmd awk
require_cmd grep

# --------------------------------------------------------------------------
# 1. Pack SHA-256: file itself vs. PROVENANCE.md vs. sidecar JSON
# --------------------------------------------------------------------------

echo "== 1. Core pack SHA-256 =="

if [[ ! -f "${PACK_FILE}" ]]; then
  log_fail "pack file not found: ${PACK_FILE}"
else
  ACTUAL_PACK_SHA="$(sha256_of "${PACK_FILE}")"
  echo "  actual file hash:      ${ACTUAL_PACK_SHA}"

  if [[ ! -f "${PROVENANCE_MD}" ]]; then
    log_fail "PROVENANCE.md not found: ${PROVENANCE_MD}"
  else
    # Grep the pack SHA-256 out of PROVENANCE.md. We look for the specific
    # "SHA-256 | \`<hex>\`" table row for the pack (§1 summary table), which
    # is the first 64-hex-char code span following the "SHA-256" label in
    # that table.
    MD_PACK_SHA="$(grep -m1 -oE '\| SHA-256 \| `[0-9a-f]{64}`' "${PROVENANCE_MD}" \
      | grep -oE '[0-9a-f]{64}' || true)"
    if [[ -z "${MD_PACK_SHA}" ]]; then
      log_fail "could not find pack SHA-256 in PROVENANCE.md"
    else
      echo "  PROVENANCE.md hash:     ${MD_PACK_SHA}"
      if [[ "${MD_PACK_SHA}" == "${ACTUAL_PACK_SHA}" ]]; then
        log_ok "pack hash matches PROVENANCE.md"
      else
        log_fail "pack hash MISMATCH vs PROVENANCE.md (${MD_PACK_SHA} != ${ACTUAL_PACK_SHA})"
      fi
    fi
  fi

  if [[ ! -f "${PACK_SIDECAR}" ]]; then
    log_fail "provenance sidecar not found: ${PACK_SIDECAR}"
  else
    SIDECAR_PACK_SHA="$(jq -r '.pack.sha256' "${PACK_SIDECAR}")"
    echo "  sidecar JSON hash:      ${SIDECAR_PACK_SHA}"
    if [[ "${SIDECAR_PACK_SHA}" == "${ACTUAL_PACK_SHA}" ]]; then
      log_ok "pack hash matches provenance sidecar"
    else
      log_fail "pack hash MISMATCH vs sidecar (${SIDECAR_PACK_SHA} != ${ACTUAL_PACK_SHA})"
    fi
  fi
fi

# --------------------------------------------------------------------------
# 2. Per-target source hashes vs. upstream manifest (only if fetched)
# --------------------------------------------------------------------------

echo
echo "== 2. Per-target source SHA-256 vs. upstream manifest =="

if [[ ! -d "${REPO_ROOT}/assets/upstream" ]] || [[ ! -f "${UPSTREAM_MANIFEST}" ]]; then
  log_notice "assets/upstream/ not present (not fetched in this checkout) -- skipping re-verification against live upstream files. Run scripts/fetch_upstream_assets.sh to enable this check."
elif [[ ! -f "${PACK_SIDECAR}" ]]; then
  log_fail "provenance sidecar not found, cannot cross-check targets: ${PACK_SIDECAR}"
else
  TARGET_COUNT="$(jq -r '.targets | length' "${PACK_SIDECAR}")"
  echo "  checking ${TARGET_COUNT} bundled targets + base mesh ..."

  MISMATCHES=0
  MISSING=0

  while IFS=$'\t' read -r rel_source sidecar_sha; do
    manifest_sha="$(jq -r --arg p "${rel_source}" \
      '.files[] | select(.path == $p) | .sha256' "${UPSTREAM_MANIFEST}")"
    if [[ -z "${manifest_sha}" ]]; then
      log_fail "target not found in upstream manifest: ${rel_source}"
      MISSING=$((MISSING + 1))
      continue
    fi
    if [[ "${manifest_sha}" != "${sidecar_sha}" ]]; then
      log_fail "manifest/sidecar hash MISMATCH for ${rel_source} (${manifest_sha} != ${sidecar_sha})"
      MISMATCHES=$((MISMATCHES + 1))
      continue
    fi
    # Also verify against the actual checked-out file on disk, if present.
    checkout_path="${REPO_ROOT}/assets/upstream/${rel_source}"
    if [[ -f "${checkout_path}" ]]; then
      disk_sha="$(sha256_of "${checkout_path}")"
      if [[ "${disk_sha}" != "${sidecar_sha}" ]]; then
        log_fail "on-disk hash MISMATCH for ${rel_source} (${disk_sha} != ${sidecar_sha})"
        MISMATCHES=$((MISMATCHES + 1))
        continue
      fi
    fi
  done < <(jq -r '.targets[] | [.source, .sha256] | @tsv' "${PACK_SIDECAR}")

  # Base mesh
  base_source="$(jq -r '.upstream.base_mesh.path' "${PACK_SIDECAR}")"
  base_sidecar_sha="$(jq -r '.upstream.base_mesh.sha256' "${PACK_SIDECAR}")"
  base_manifest_sha="$(jq -r --arg p "${base_source}" \
    '.files[] | select(.path == $p) | .sha256' "${UPSTREAM_MANIFEST}")"
  if [[ -z "${base_manifest_sha}" ]]; then
    log_fail "base mesh not found in upstream manifest: ${base_source}"
    MISSING=$((MISSING + 1))
  elif [[ "${base_manifest_sha}" != "${base_sidecar_sha}" ]]; then
    log_fail "base mesh hash MISMATCH (${base_manifest_sha} != ${base_sidecar_sha})"
    MISMATCHES=$((MISMATCHES + 1))
  fi
  base_checkout_path="${REPO_ROOT}/assets/upstream/${base_source}"
  if [[ -f "${base_checkout_path}" ]]; then
    base_disk_sha="$(sha256_of "${base_checkout_path}")"
    if [[ "${base_disk_sha}" != "${base_sidecar_sha}" ]]; then
      log_fail "base mesh on-disk hash MISMATCH (${base_disk_sha} != ${base_sidecar_sha})"
      MISMATCHES=$((MISMATCHES + 1))
    fi
  fi

  if [[ "${MISMATCHES}" -eq 0 && "${MISSING}" -eq 0 ]]; then
    log_ok "all ${TARGET_COUNT} targets + base mesh verified against upstream manifest"
  else
    log_fail "${MISMATCHES} mismatch(es), ${MISSING} missing entrie(s) found"
  fi
fi

# --------------------------------------------------------------------------
# 3. alpha_pack manifest core_pack_sha256 vs. actual pack file
# --------------------------------------------------------------------------

echo
echo "== 3. Alpha-pack manifest core_pack_sha256 =="

if [[ ! -f "${ALPHA_PACK_TOML}" ]]; then
  log_fail "alpha pack manifest not found: ${ALPHA_PACK_TOML}"
elif [[ ! -f "${PACK_FILE}" ]]; then
  log_fail "pack file not found, cannot cross-check: ${PACK_FILE}"
else
  TOML_PACK_SHA="$(grep -m1 -oE '^core_pack_sha256 = "[0-9a-f]{64}"' "${ALPHA_PACK_TOML}" \
    | grep -oE '[0-9a-f]{64}' || true)"
  ACTUAL_PACK_SHA="$(sha256_of "${PACK_FILE}")"
  if [[ -z "${TOML_PACK_SHA}" ]]; then
    log_fail "could not find core_pack_sha256 in ${ALPHA_PACK_TOML}"
  else
    echo "  toml hash:   ${TOML_PACK_SHA}"
    echo "  actual hash: ${ACTUAL_PACK_SHA}"
    if [[ "${TOML_PACK_SHA}" == "${ACTUAL_PACK_SHA}" ]]; then
      log_ok "alpha_pack manifest core_pack_sha256 matches pack file"
    else
      log_fail "alpha_pack manifest core_pack_sha256 MISMATCH (${TOML_PACK_SHA} != ${ACTUAL_PACK_SHA})"
    fi
  fi
fi

# --------------------------------------------------------------------------
# Summary
# --------------------------------------------------------------------------

echo
if [[ "${FAILURES}" -eq 0 ]]; then
  echo "check_provenance.sh: ALL CHECKS PASSED"
  exit 0
else
  echo "check_provenance.sh: ${FAILURES} CHECK(S) FAILED" >&2
  exit 1
fi
