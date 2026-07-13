#!/usr/bin/env bash
#
# fetch_upstream_assets.sh
#
# Fetches the CC0-licensed MakeHuman base mesh and morph targets from the
# upstream makehumancommunity/makehuman repository, using a git partial
# clone + sparse checkout so only the required blobs are downloaded.
#
# What it does:
#   1. Resolves the latest stable release tag from upstream (prefers a
#      v1.3.x tag; falls back to the highest stable tag overall) and its
#      commit SHA, via `git ls-remote --tags` (no full clone needed).
#   2. Clones just enough of the repository (--filter=blob:none,
#      --no-checkout, --depth 1, --branch <tag>) and sparse-checks-out only:
#        makehuman/data/3dobjs
#        makehuman/data/targets
#        LICENSE.md
#        LICENSE.ASSETS.md
#   3. Writes assets/upstream/UPSTREAM_MANIFEST.json recording the pinned
#      repo/tag/commit plus a SHA-256 + byte size for base.obj, every
#      *.target file, and both LICENSE files.
#
# Idempotent: re-running when the pinned commit is already checked out only
# regenerates the manifest. Pass --force to wipe and re-clone unconditionally.
#
# All paths are derived from this script's own location -- no absolute
# paths are hardcoded.

set -euo pipefail

# --------------------------------------------------------------------------
# Paths (repo-relative, derived from script location)
# --------------------------------------------------------------------------

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." &>/dev/null && pwd)"

UPSTREAM_ROOT_DIR="${REPO_ROOT}/assets/upstream"
UPSTREAM_CHECKOUT_DIR="${UPSTREAM_ROOT_DIR}/makehuman"
MANIFEST_PATH="${UPSTREAM_ROOT_DIR}/UPSTREAM_MANIFEST.json"

UPSTREAM_REPO_URL="https://github.com/makehumancommunity/makehuman.git"

# Sparse-checkout targets (cone mode always also includes top-level repo
# files such as README.md/.gitignore; that is expected and harmless).
SPARSE_PATHS=(
  "makehuman/data/3dobjs"
  "makehuman/data/targets"
  "LICENSE.md"
  "LICENSE.ASSETS.md"
)

FORCE_RECLONE=0
if [[ "${1:-}" == "--force" ]]; then
  FORCE_RECLONE=1
fi

log() {
  printf '[fetch-upstream-assets] %s\n' "$*" >&2
}

die() {
  printf '[fetch-upstream-assets] ERROR: %s\n' "$*" >&2
  exit 1
}

# --------------------------------------------------------------------------
# Tooling checks
# --------------------------------------------------------------------------

command -v git >/dev/null 2>&1 || die "git is required but was not found on PATH"
command -v python3 >/dev/null 2>&1 || die "python3 is required (used to emit UPSTREAM_MANIFEST.json)"

SHA256_TOOL=""
if command -v shasum >/dev/null 2>&1; then
  SHA256_TOOL="shasum"
elif command -v sha256sum >/dev/null 2>&1; then
  SHA256_TOOL="sha256sum"
elif command -v python3 >/dev/null 2>&1; then
  SHA256_TOOL="python3"
else
  die "need one of: shasum, sha256sum, python3 (for SHA-256 hashing)"
fi

sha256_of() {
  local file="$1"
  case "${SHA256_TOOL}" in
    shasum)
      shasum -a 256 -- "${file}" | awk '{print $1}'
      ;;
    sha256sum)
      sha256sum -- "${file}" | awk '{print $1}'
      ;;
    python3)
      python3 -c "import hashlib,sys; print(hashlib.sha256(open(sys.argv[1],'rb').read()).hexdigest())" "${file}"
      ;;
  esac
}

# --------------------------------------------------------------------------
# Resolve the latest stable release tag + commit SHA from upstream
# --------------------------------------------------------------------------

resolve_upstream_tag() {
  local raw_refs stable_pairs v13_pairs chosen_pair

  raw_refs="$(git ls-remote --tags --refs "${UPSTREAM_REPO_URL}")" \
    || die "could not list tags from ${UPSTREAM_REPO_URL}"

  # "<normalized-version> <original-tag>", stable (X.Y.Z / vX.Y.Z, no
  # -rc/-alpha/-beta/-dev suffixes) tags only.
  stable_pairs="$(printf '%s\n' "${raw_refs}" \
    | awk '{sub("refs/tags/", "", $2); print $2}' \
    | grep -E '^v?[0-9]+\.[0-9]+\.[0-9]+$' \
    | awk '{orig=$0; norm=$0; sub(/^v/, "", norm); print norm, orig}')"

  [[ -n "${stable_pairs}" ]] || die "no stable release tags found on ${UPSTREAM_REPO_URL}"

  # Prefer the highest 1.3.x tag if one exists.
  v13_pairs="$(printf '%s\n' "${stable_pairs}" | awk '$1 ~ /^1\.3\.[0-9]+$/')"

  if [[ -n "${v13_pairs}" ]]; then
    chosen_pair="$(printf '%s\n' "${v13_pairs}" | sort -t. -k1,1n -k2,2n -k3,3n | tail -1)"
  else
    chosen_pair="$(printf '%s\n' "${stable_pairs}" | sort -t. -k1,1n -k2,2n -k3,3n | tail -1)"
  fi

  # Emits "<tag> <commit>" on stdout.
  local tag commit
  tag="$(printf '%s' "${chosen_pair}" | awk '{print $2}')"
  commit="$(printf '%s\n' "${raw_refs}" | awk -v t="refs/tags/${tag}" '$2 == t {print $1}')"

  [[ -n "${tag}" && -n "${commit}" ]] || die "failed to resolve tag/commit from upstream refs"

  printf '%s %s\n' "${tag}" "${commit}"
}

log "Resolving latest stable release tag from ${UPSTREAM_REPO_URL} ..."
read -r UPSTREAM_TAG UPSTREAM_COMMIT < <(resolve_upstream_tag)
log "Pinned upstream tag=${UPSTREAM_TAG} commit=${UPSTREAM_COMMIT}"

# --------------------------------------------------------------------------
# Partial clone + sparse checkout (idempotent)
# --------------------------------------------------------------------------

mkdir -p "${UPSTREAM_ROOT_DIR}"

need_clone=1
if [[ "${FORCE_RECLONE}" -eq 0 && -d "${UPSTREAM_CHECKOUT_DIR}/.git" ]]; then
  current_commit="$(git -C "${UPSTREAM_CHECKOUT_DIR}" rev-parse HEAD 2>/dev/null || true)"
  if [[ "${current_commit}" == "${UPSTREAM_COMMIT}" \
        && -f "${UPSTREAM_CHECKOUT_DIR}/makehuman/data/3dobjs/base.obj" \
        && -f "${UPSTREAM_CHECKOUT_DIR}/LICENSE.ASSETS.md" ]]; then
    need_clone=0
    log "Existing checkout already at pinned commit ${UPSTREAM_COMMIT}; skipping clone."
  fi
fi

if [[ "${need_clone}" -eq 1 ]]; then
  log "(Re)cloning ${UPSTREAM_REPO_URL} @ ${UPSTREAM_TAG} (partial clone, sparse checkout) ..."
  rm -rf "${UPSTREAM_CHECKOUT_DIR}"
  git clone \
    --filter=blob:none \
    --no-checkout \
    --depth 1 \
    --branch "${UPSTREAM_TAG}" \
    "${UPSTREAM_REPO_URL}" \
    "${UPSTREAM_CHECKOUT_DIR}"

  git -C "${UPSTREAM_CHECKOUT_DIR}" sparse-checkout init --cone
  git -C "${UPSTREAM_CHECKOUT_DIR}" sparse-checkout set "${SPARSE_PATHS[@]}"
  git -C "${UPSTREAM_CHECKOUT_DIR}" checkout "${UPSTREAM_TAG}"

  checked_out_commit="$(git -C "${UPSTREAM_CHECKOUT_DIR}" rev-parse HEAD)"
  [[ "${checked_out_commit}" == "${UPSTREAM_COMMIT}" ]] \
    || die "checked-out commit ${checked_out_commit} != resolved commit ${UPSTREAM_COMMIT}"
fi

BASE_OBJ_PATH="${UPSTREAM_CHECKOUT_DIR}/makehuman/data/3dobjs/base.obj"
TARGETS_DIR="${UPSTREAM_CHECKOUT_DIR}/makehuman/data/targets"
LICENSE_MD_PATH="${UPSTREAM_CHECKOUT_DIR}/LICENSE.md"
LICENSE_ASSETS_PATH="${UPSTREAM_CHECKOUT_DIR}/LICENSE.ASSETS.md"

[[ -f "${BASE_OBJ_PATH}" ]] || die "expected file missing after checkout: ${BASE_OBJ_PATH}"
[[ -d "${TARGETS_DIR}" ]] || die "expected directory missing after checkout: ${TARGETS_DIR}"
[[ -f "${LICENSE_MD_PATH}" ]] || die "expected file missing after checkout: ${LICENSE_MD_PATH}"
[[ -f "${LICENSE_ASSETS_PATH}" ]] || die "expected file missing after checkout: ${LICENSE_ASSETS_PATH}"

# --------------------------------------------------------------------------
# Build UPSTREAM_MANIFEST.json (base.obj + every *.target + both LICENSEs)
# --------------------------------------------------------------------------

log "Hashing manifest files (base.obj, *.target, LICENSE*.md) ..."

MANIFEST_FILE_LIST="$(mktemp)"
MANIFEST_ROWS_FILE="$(mktemp)"
trap 'rm -f "${MANIFEST_FILE_LIST}" "${MANIFEST_ROWS_FILE}"' EXIT

{
  printf '%s\n' "${BASE_OBJ_PATH}"
  printf '%s\n' "${LICENSE_MD_PATH}"
  printf '%s\n' "${LICENSE_ASSETS_PATH}"
  find "${TARGETS_DIR}" -type f -name '*.target' | sort
} > "${MANIFEST_FILE_LIST}"

FILE_COUNT="$(wc -l < "${MANIFEST_FILE_LIST}" | tr -d ' ')"
log "Manifest will record ${FILE_COUNT} files."

: > "${MANIFEST_ROWS_FILE}"
while IFS= read -r abs_path; do
  rel_path="${abs_path#"${UPSTREAM_CHECKOUT_DIR}"/}"
  digest="$(sha256_of "${abs_path}")"
  bytes="$(wc -c < "${abs_path}" | tr -d ' ')"
  printf '%s\t%s\t%s\n' "${rel_path}" "${digest}" "${bytes}" >> "${MANIFEST_ROWS_FILE}"
done < "${MANIFEST_FILE_LIST}"

FETCHED_AT="$(TZ=UTC date -u +%Y-%m-%dT%H:%M:%SZ)"
COMMIT_DATE="$(git -C "${UPSTREAM_CHECKOUT_DIR}" show -s --format=%cI "${UPSTREAM_COMMIT}")"

python3 - "${MANIFEST_PATH}" "${UPSTREAM_REPO_URL}" "${UPSTREAM_TAG}" "${UPSTREAM_COMMIT}" \
  "${FETCHED_AT}" "${COMMIT_DATE}" "${MANIFEST_ROWS_FILE}" <<'PYEOF'
import json
import sys

(manifest_path, repo, tag, commit, fetched_at, commit_date, rows_path) = sys.argv[1:8]

files = []
with open(rows_path, "r", encoding="utf-8") as fh:
    for line in fh:
        line = line.rstrip("\n")
        if not line:
            continue
        path, sha256, byte_count = line.split("\t")
        files.append({"path": path, "sha256": sha256, "bytes": int(byte_count)})

manifest = {
    "repo": repo,
    "tag": tag,
    "commit": commit,
    "commit_date": commit_date,
    "fetched_at": fetched_at,
    "file_count": len(files),
    "total_bytes": sum(f["bytes"] for f in files),
    "files": files,
}

with open(manifest_path, "w", encoding="utf-8") as fh:
    json.dump(manifest, fh, indent=2, sort_keys=False)
    fh.write("\n")

print(f"[fetch-upstream-assets] wrote {manifest_path} ({len(files)} files)", file=sys.stderr)
PYEOF

log "Done. Manifest: ${MANIFEST_PATH}"
