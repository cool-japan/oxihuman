// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cluster-based visibility / frustum-tile culling state.

/// A 3-D cluster grid tile index.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClusterIdx {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

/// Visibility record for one cluster.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ClusterVisEntry {
    pub idx: ClusterIdx,
    pub visible: bool,
    pub light_count: u32,
}

/// Manager.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct ClusterVisibility {
    pub entries: Vec<ClusterVisEntry>,
    pub grid_dims: [u32; 3],
}

#[allow(dead_code)]
pub fn new_cluster_visibility(x: u32, y: u32, z: u32) -> ClusterVisibility {
    ClusterVisibility {
        entries: Vec::new(),
        grid_dims: [x, y, z],
    }
}

#[allow(dead_code)]
pub fn cv_register(vis: &mut ClusterVisibility, idx: ClusterIdx) {
    if !vis.entries.iter().any(|e| e.idx == idx) {
        vis.entries.push(ClusterVisEntry {
            idx,
            visible: false,
            light_count: 0,
        });
    }
}

#[allow(dead_code)]
pub fn cv_set_visible(vis: &mut ClusterVisibility, idx: ClusterIdx, v: bool) {
    if let Some(e) = vis.entries.iter_mut().find(|e| e.idx == idx) {
        e.visible = v;
    }
}

#[allow(dead_code)]
pub fn cv_set_light_count(vis: &mut ClusterVisibility, idx: ClusterIdx, n: u32) {
    if let Some(e) = vis.entries.iter_mut().find(|e| e.idx == idx) {
        e.light_count = n;
    }
}

#[allow(dead_code)]
pub fn cv_visible_count(vis: &ClusterVisibility) -> usize {
    vis.entries.iter().filter(|e| e.visible).count()
}

#[allow(dead_code)]
pub fn cv_total_lights(vis: &ClusterVisibility) -> u32 {
    vis.entries.iter().map(|e| e.light_count).sum()
}

#[allow(dead_code)]
pub fn cv_clear(vis: &mut ClusterVisibility) {
    vis.entries.clear();
}

#[allow(dead_code)]
pub fn cv_total_clusters(vis: &ClusterVisibility) -> u32 {
    vis.grid_dims[0] * vis.grid_dims[1] * vis.grid_dims[2]
}

#[allow(dead_code)]
pub fn cv_to_json(vis: &ClusterVisibility) -> String {
    format!(
        "{{\"dims\":[{},{},{}],\"visible\":{},\"total\":{}}}",
        vis.grid_dims[0],
        vis.grid_dims[1],
        vis.grid_dims[2],
        cv_visible_count(vis),
        vis.entries.len()
    )
}

#[allow(dead_code)]
pub fn cv_fill_all_visible(vis: &mut ClusterVisibility) {
    for e in &mut vis.entries {
        e.visible = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk() -> ClusterVisibility {
        new_cluster_visibility(4, 4, 4)
    }

    #[test]
    fn new_empty() {
        assert!(mk().entries.is_empty());
    }

    #[test]
    fn register_adds_entry() {
        let mut v = mk();
        cv_register(&mut v, ClusterIdx { x: 0, y: 0, z: 0 });
        assert_eq!(v.entries.len(), 1);
    }

    #[test]
    fn duplicate_register_ignored() {
        let mut v = mk();
        let idx = ClusterIdx { x: 1, y: 1, z: 1 };
        cv_register(&mut v, idx);
        cv_register(&mut v, idx);
        assert_eq!(v.entries.len(), 1);
    }

    #[test]
    fn set_visible() {
        let mut v = mk();
        let idx = ClusterIdx { x: 0, y: 0, z: 0 };
        cv_register(&mut v, idx);
        cv_set_visible(&mut v, idx, true);
        assert_eq!(cv_visible_count(&v), 1);
    }

    #[test]
    fn visible_count_default_zero() {
        let mut v = mk();
        cv_register(&mut v, ClusterIdx { x: 0, y: 0, z: 0 });
        assert_eq!(cv_visible_count(&v), 0);
    }

    #[test]
    fn total_lights() {
        let mut v = mk();
        let idx = ClusterIdx { x: 0, y: 0, z: 0 };
        cv_register(&mut v, idx);
        cv_set_light_count(&mut v, idx, 5);
        assert_eq!(cv_total_lights(&v), 5);
    }

    #[test]
    fn clear_empty() {
        let mut v = mk();
        cv_register(&mut v, ClusterIdx { x: 0, y: 0, z: 0 });
        cv_clear(&mut v);
        assert!(v.entries.is_empty());
    }

    #[test]
    fn total_clusters_product() {
        let v = new_cluster_visibility(2, 3, 4);
        assert_eq!(cv_total_clusters(&v), 24);
    }

    #[test]
    fn fill_all_visible() {
        let mut v = mk();
        cv_register(&mut v, ClusterIdx { x: 0, y: 0, z: 0 });
        cv_register(&mut v, ClusterIdx { x: 1, y: 0, z: 0 });
        cv_fill_all_visible(&mut v);
        assert_eq!(cv_visible_count(&v), 2);
    }

    #[test]
    fn json_has_dims() {
        assert!(cv_to_json(&mk()).contains("dims"));
    }
}
