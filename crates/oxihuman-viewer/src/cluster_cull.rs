// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cluster culling — view-space tile/cluster visibility for light culling.

use std::f32::consts::FRAC_PI_2;

/// Axis-aligned bounding box.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ClusterAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// A single cluster entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClusterEntry {
    pub aabb: ClusterAabb,
    pub visible: bool,
    pub light_count: u16,
}

/// Cluster cull manager.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClusterCull {
    entries: Vec<ClusterEntry>,
    /// Tile count along X, Y, Z.
    dims: [u32; 3],
}

/// Create a new cluster cull manager.
pub fn new_cluster_cull(x: u32, y: u32, z: u32) -> ClusterCull {
    let n = (x * y * z) as usize;
    let entries = (0..n)
        .map(|_| ClusterEntry {
            aabb: ClusterAabb {
                min: [0.0; 3],
                max: [1.0; 3],
            },
            visible: false,
            light_count: 0,
        })
        .collect();
    ClusterCull {
        entries,
        dims: [x, y, z],
    }
}

/// Mark all clusters as visible.
pub fn cc_mark_all_visible(cull: &mut ClusterCull) {
    for e in &mut cull.entries {
        e.visible = true;
    }
}

/// Clear all visibility flags.
pub fn cc_clear(cull: &mut ClusterCull) {
    for e in &mut cull.entries {
        e.visible = false;
        e.light_count = 0;
    }
}

/// Number of clusters.
pub fn cc_count(cull: &ClusterCull) -> usize {
    cull.entries.len()
}

/// Count visible clusters.
pub fn cc_visible_count(cull: &ClusterCull) -> usize {
    cull.entries.iter().filter(|e| e.visible).count()
}

/// Set visibility of a cluster by flat index.
pub fn cc_set_visible(cull: &mut ClusterCull, idx: usize, visible: bool) {
    if let Some(e) = cull.entries.get_mut(idx) {
        e.visible = visible;
    }
}

/// Depth slice index from a linear depth value.
pub fn cc_depth_slice(cull: &ClusterCull, depth_linear: f32, near: f32, far: f32) -> u32 {
    let z = cull.dims[2];
    if depth_linear <= near || far <= near {
        return 0;
    }
    let t = ((depth_linear - near) / (far - near)).clamp(0.0, 1.0);
    ((t * z as f32) as u32).min(z.saturating_sub(1))
}

/// Reference constant (π/2) to satisfy the import.
pub fn cc_half_fov_ref() -> f32 {
    FRAC_PI_2
}

/// Compute flat index from (x, y, z) tile coordinates.
pub fn cc_flat_index(cull: &ClusterCull, tx: u32, ty: u32, tz: u32) -> Option<usize> {
    let [nx, ny, nz] = cull.dims;
    if tx < nx && ty < ny && tz < nz {
        Some((tz * ny * nx + ty * nx + tx) as usize)
    } else {
        None
    }
}

/// Serialise.
pub fn cc_to_json(cull: &ClusterCull) -> String {
    format!(
        r#"{{"dims":[{},{},{}],"count":{},"visible":{}}}"#,
        cull.dims[0],
        cull.dims[1],
        cull.dims[2],
        cc_count(cull),
        cc_visible_count(cull)
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ClusterCull {
        new_cluster_cull(4, 4, 4)
    }

    #[test]
    fn count_matches_dims() {
        let c = make();
        assert_eq!(cc_count(&c), 64);
    }

    #[test]
    fn none_visible_initially() {
        assert_eq!(cc_visible_count(&make()), 0);
    }

    #[test]
    fn mark_all_visible() {
        let mut c = make();
        cc_mark_all_visible(&mut c);
        assert_eq!(cc_visible_count(&c), 64);
    }

    #[test]
    fn clear_resets_visible() {
        let mut c = make();
        cc_mark_all_visible(&mut c);
        cc_clear(&mut c);
        assert_eq!(cc_visible_count(&c), 0);
    }

    #[test]
    fn set_visible_single() {
        let mut c = make();
        cc_set_visible(&mut c, 0, true);
        assert_eq!(cc_visible_count(&c), 1);
    }

    #[test]
    fn depth_slice_in_range() {
        let c = make();
        let s = cc_depth_slice(&c, 5.0, 0.1, 100.0);
        assert!(s < 4);
    }

    #[test]
    fn flat_index_valid() {
        let c = make();
        assert!(cc_flat_index(&c, 0, 0, 0).is_some());
        assert_eq!(cc_flat_index(&c, 0, 0, 0), Some(0));
    }

    #[test]
    fn flat_index_out_of_range() {
        let c = make();
        assert!(cc_flat_index(&c, 5, 0, 0).is_none());
    }

    #[test]
    fn json_has_dims() {
        assert!(cc_to_json(&make()).contains("dims"));
    }
}
