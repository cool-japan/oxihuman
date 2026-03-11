// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cluster shadow — tile-based clustered shadow assignment for many lights.

/// Configuration for cluster shadow.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClusterShadowConfig {
    pub tile_width: u32,
    pub tile_height: u32,
    pub depth_slices: u32,
    pub max_shadows_per_cluster: u32,
}

/// A single shadow entry in the cluster.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClusterShadowEntry {
    pub light_id: u32,
    pub shadow_map_index: u32,
    pub enabled: bool,
}

/// The cluster shadow manager.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClusterShadowManager {
    pub config: ClusterShadowConfig,
    pub entries: Vec<ClusterShadowEntry>,
}

#[allow(dead_code)]
pub fn default_cluster_shadow_config() -> ClusterShadowConfig {
    ClusterShadowConfig {
        tile_width: 16,
        tile_height: 16,
        depth_slices: 24,
        max_shadows_per_cluster: 8,
    }
}

#[allow(dead_code)]
pub fn new_cluster_shadow_manager(config: ClusterShadowConfig) -> ClusterShadowManager {
    ClusterShadowManager {
        config,
        entries: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn csm_add_entry(mgr: &mut ClusterShadowManager, light_id: u32, shadow_map_index: u32) {
    mgr.entries.push(ClusterShadowEntry {
        light_id,
        shadow_map_index,
        enabled: true,
    });
}

#[allow(dead_code)]
pub fn csm_remove_entry(mgr: &mut ClusterShadowManager, light_id: u32) {
    mgr.entries.retain(|e| e.light_id != light_id);
}

#[allow(dead_code)]
pub fn csm_set_enabled(mgr: &mut ClusterShadowManager, light_id: u32, enabled: bool) {
    for e in &mut mgr.entries {
        if e.light_id == light_id {
            e.enabled = enabled;
            break;
        }
    }
}

#[allow(dead_code)]
pub fn csm_entry_count(mgr: &ClusterShadowManager) -> usize {
    mgr.entries.len()
}

#[allow(dead_code)]
pub fn csm_enabled_count(mgr: &ClusterShadowManager) -> usize {
    mgr.entries.iter().filter(|e| e.enabled).count()
}

#[allow(dead_code)]
pub fn csm_total_clusters(mgr: &ClusterShadowManager) -> u32 {
    mgr.config.tile_width * mgr.config.tile_height * mgr.config.depth_slices
}

#[allow(dead_code)]
pub fn csm_clear(mgr: &mut ClusterShadowManager) {
    mgr.entries.clear();
}

#[allow(dead_code)]
pub fn csm_to_json(mgr: &ClusterShadowManager) -> String {
    format!(
        r#"{{"entry_count":{},"enabled_count":{},"total_clusters":{}}}"#,
        csm_entry_count(mgr),
        csm_enabled_count(mgr),
        csm_total_clusters(mgr)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_cluster_shadow_config();
        assert_eq!(cfg.tile_width, 16);
    }

    #[test]
    fn new_manager_empty() {
        let mgr = new_cluster_shadow_manager(default_cluster_shadow_config());
        assert_eq!(csm_entry_count(&mgr), 0);
    }

    #[test]
    fn add_entry() {
        let mut mgr = new_cluster_shadow_manager(default_cluster_shadow_config());
        csm_add_entry(&mut mgr, 0, 0);
        assert_eq!(csm_entry_count(&mgr), 1);
    }

    #[test]
    fn remove_entry() {
        let mut mgr = new_cluster_shadow_manager(default_cluster_shadow_config());
        csm_add_entry(&mut mgr, 1, 0);
        csm_remove_entry(&mut mgr, 1);
        assert_eq!(csm_entry_count(&mgr), 0);
    }

    #[test]
    fn set_disabled() {
        let mut mgr = new_cluster_shadow_manager(default_cluster_shadow_config());
        csm_add_entry(&mut mgr, 2, 1);
        csm_set_enabled(&mut mgr, 2, false);
        assert_eq!(csm_enabled_count(&mgr), 0);
    }

    #[test]
    fn total_clusters() {
        let mgr = new_cluster_shadow_manager(default_cluster_shadow_config());
        assert_eq!(csm_total_clusters(&mgr), 16 * 16 * 24);
    }

    #[test]
    fn clear_entries() {
        let mut mgr = new_cluster_shadow_manager(default_cluster_shadow_config());
        csm_add_entry(&mut mgr, 0, 0);
        csm_add_entry(&mut mgr, 1, 1);
        csm_clear(&mut mgr);
        assert_eq!(csm_entry_count(&mgr), 0);
    }

    #[test]
    fn enabled_count_all_enabled() {
        let mut mgr = new_cluster_shadow_manager(default_cluster_shadow_config());
        csm_add_entry(&mut mgr, 0, 0);
        csm_add_entry(&mut mgr, 1, 1);
        assert_eq!(csm_enabled_count(&mgr), 2);
    }

    #[test]
    fn to_json_fields() {
        let mgr = new_cluster_shadow_manager(default_cluster_shadow_config());
        let j = csm_to_json(&mgr);
        assert!(j.contains("entry_count"));
    }
}
