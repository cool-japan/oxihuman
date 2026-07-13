#!/usr/bin/env bash
# Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
# SPDX-License-Identifier: Apache-2.0
#
# Reproduce the measurement / fit round-trip error table.
#
# Usage:
#   scripts/measure_roundtrip.sh > docs/bench/measurement-error.md
#
# Requires only the checked-in core pack
# (assets/packs/oxihuman-core-v1.ohpk) — no network, no external tools.
set -euo pipefail
cd "$(dirname "$0")/.."
exec cargo run --quiet --release --example measure_roundtrip -p oxihuman-wasm "$@"
