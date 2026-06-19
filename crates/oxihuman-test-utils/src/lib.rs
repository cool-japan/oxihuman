// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Shared test helpers for the OxiHuman workspace.
//!
//! All helpers resolve their paths from environment variables and fall back
//! to a sentinel path under `/tmp` that does not exist, causing tests that
//! require the data to skip gracefully via an early `if !path.exists()` check.
//!
//! Environment variables:
//! - `MAKEHUMAN_DATA_DIR` — MakeHuman `data/` directory (contains `3dobjs/` and `targets/`).
//! - `OXIHUMAN_ASSETS_DIR` — directory containing `alpha_pack/oxihuman_assets.toml`.

use std::path::PathBuf;

/// Returns the MakeHuman data root from `$MAKEHUMAN_DATA_DIR`, or a
/// nonexistent sentinel path so tests skip gracefully when the var is unset.
pub fn makehuman_data_dir() -> PathBuf {
    std::env::var("MAKEHUMAN_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/oxihuman_nonexistent_data"))
}

/// Returns `$MAKEHUMAN_DATA_DIR/3dobjs/base.obj`.
pub fn base_obj() -> PathBuf {
    makehuman_data_dir().join("3dobjs/base.obj")
}

/// Returns `$MAKEHUMAN_DATA_DIR/targets`.
///
/// When a test needs a specific sub-directory (e.g. `bodyshapes`), call
/// `.join("bodyshapes")` on the returned path.
pub fn targets_dir() -> PathBuf {
    makehuman_data_dir().join("targets")
}

/// Returns the OxiHuman assets root from `$OXIHUMAN_ASSETS_DIR`, or a
/// nonexistent sentinel path so tests skip gracefully when the var is unset.
pub fn assets_dir() -> PathBuf {
    std::env::var("OXIHUMAN_ASSETS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/oxihuman_nonexistent_assets"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentinel_paths_are_absolute() {
        let d = makehuman_data_dir();
        assert!(d.is_absolute(), "makehuman_data_dir must be absolute");
        let b = base_obj();
        assert!(b.is_absolute(), "base_obj must be absolute");
        let t = targets_dir();
        assert!(t.is_absolute(), "targets_dir must be absolute");
        let a = assets_dir();
        assert!(a.is_absolute(), "assets_dir must be absolute");
    }

    #[test]
    fn base_obj_is_child_of_data_dir() {
        let d = makehuman_data_dir();
        let b = base_obj();
        assert!(b.starts_with(&d));
    }

    #[test]
    fn targets_dir_is_child_of_data_dir() {
        let d = makehuman_data_dir();
        let t = targets_dir();
        assert!(t.starts_with(&d));
    }
}
