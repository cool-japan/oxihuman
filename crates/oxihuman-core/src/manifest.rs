// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::policy::PolicyProfile;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackInfo {
    pub name: String,
    pub author: String,
    pub license: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifest {
    pub version: String,
    pub base_mesh_path: String,
    pub allowed_targets: Vec<String>,
    pub policy_profile: PolicyProfile,
    #[serde(default)]
    pub pack_info: Option<PackInfo>,
}

impl AssetManifest {
    pub fn load(path: &Path) -> Result<Self> {
        let src = std::fs::read_to_string(path)?;
        let manifest: AssetManifest = toml::from_str(&src)?;
        Ok(manifest)
    }

    pub fn default_manifest() -> Self {
        AssetManifest {
            version: "0.1.0".to_string(),
            base_mesh_path: "data/3dobjs/base.obj".to_string(),
            allowed_targets: Vec::new(),
            policy_profile: PolicyProfile::Standard,
            pack_info: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_manifest_fields() {
        let m = AssetManifest::default_manifest();
        assert_eq!(m.version, "0.1.0");
        assert!(!m.base_mesh_path.is_empty());
    }

    #[test]
    fn load_alpha_pack_manifest() {
        let path = {
            std::env::var("OXIHUMAN_ASSETS_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::path::PathBuf::from("/tmp/oxihuman_nonexistent_assets"))
                .join("alpha_pack/oxihuman_assets.toml")
        };
        if path.exists() {
            let m = AssetManifest::load(&path).expect("should succeed");
            assert_eq!(m.version, "0.1.0");
            assert!(!m.allowed_targets.is_empty());
        }
    }
}
