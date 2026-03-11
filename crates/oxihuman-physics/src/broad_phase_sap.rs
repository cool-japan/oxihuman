//! Sweep-and-prune (SAP) broadphase collision detection.
//!
//! Maintains a set of axis-aligned bounding boxes (AABBs) sorted along one
//! axis (X by default). Overlapping pairs are found by sweeping the sorted
//! interval list and checking for overlaps on all three axes.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Config / structs
// ---------------------------------------------------------------------------

/// Configuration for the SAP broadphase.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SapConfig {
    /// Primary sort axis: 0=X, 1=Y, 2=Z.
    pub sort_axis: usize,
    /// If true, re-sort the interval list on every overlap query.
    pub always_sort: bool,
}

/// Axis-aligned bounding box used by the SAP broadphase.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct SapAabb {
    /// Minimum corner.
    pub min: [f32; 3],
    /// Maximum corner.
    pub max: [f32; 3],
}

/// A pair of overlapping body IDs returned by [`sap_query_overlaps`].
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SapOverlapPair {
    /// Body with the smaller ID.
    pub id_a: u32,
    /// Body with the larger ID.
    pub id_b: u32,
}

/// The SAP broadphase state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SapBroadphase {
    /// Map from body ID to AABB.
    aabbs: HashMap<u32, SapAabb>,
    /// Configuration.
    cfg: SapConfig,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns a [`SapConfig`] with sensible defaults (sort on X axis).
#[allow(dead_code)]
pub fn default_sap_config() -> SapConfig {
    SapConfig { sort_axis: 0, always_sort: true }
}

/// Create a new [`SapBroadphase`] with the given config.
#[allow(dead_code)]
pub fn new_sap_broadphase(cfg: &SapConfig) -> SapBroadphase {
    SapBroadphase { aabbs: HashMap::new(), cfg: cfg.clone() }
}

/// Insert or replace the AABB for body `id`.
#[allow(dead_code)]
pub fn sap_insert(bp: &mut SapBroadphase, id: u32, aabb: SapAabb) {
    bp.aabbs.insert(id, aabb);
}

/// Remove the AABB for body `id`. Returns `true` if it existed.
#[allow(dead_code)]
pub fn sap_remove(bp: &mut SapBroadphase, id: u32) -> bool {
    bp.aabbs.remove(&id).is_some()
}

/// Update (replace) the AABB for body `id`. Same as insert.
#[allow(dead_code)]
pub fn sap_update(bp: &mut SapBroadphase, id: u32, aabb: SapAabb) {
    bp.aabbs.insert(id, aabb);
}

/// Return all overlapping pairs found by sweeping the sorted interval list.
#[allow(dead_code)]
pub fn sap_query_overlaps(bp: &SapBroadphase) -> Vec<SapOverlapPair> {
    let axis = bp.cfg.sort_axis.clamp(0, 2);

    // Collect and sort by min on the chosen axis
    let mut entries: Vec<(u32, SapAabb)> = bp.aabbs.iter().map(|(&id, &ab)| (id, ab)).collect();
    entries.sort_by(|a, b| a.1.min[axis].partial_cmp(&b.1.min[axis]).unwrap_or(std::cmp::Ordering::Equal));

    let mut pairs: Vec<SapOverlapPair> = Vec::new();

    for i in 0..entries.len() {
        let (id_a, aabb_a) = entries[i];
        for (id_b, aabb_b) in entries[i + 1..].iter().copied() {
            // Early exit: b starts after a ends on the sort axis
            if aabb_b.min[axis] > aabb_a.max[axis] {
                break;
            }
            if aabb_overlaps(&aabb_a, &aabb_b) {
                let pair = if id_a < id_b {
                    SapOverlapPair { id_a, id_b }
                } else {
                    SapOverlapPair { id_a: id_b, id_b: id_a }
                };
                pairs.push(pair);
            }
        }
    }

    pairs
}

/// Returns the number of AABBs currently in the broadphase.
#[allow(dead_code)]
pub fn sap_aabb_count(bp: &SapBroadphase) -> usize {
    bp.aabbs.len()
}

/// Returns true if two AABBs overlap on all three axes.
#[allow(dead_code)]
pub fn aabb_overlaps(a: &SapAabb, b: &SapAabb) -> bool {
    a.min[0] <= b.max[0]
        && a.max[0] >= b.min[0]
        && a.min[1] <= b.max[1]
        && a.max[1] >= b.min[1]
        && a.min[2] <= b.max[2]
        && a.max[2] >= b.min[2]
}

/// Remove all AABBs from the broadphase.
#[allow(dead_code)]
pub fn sap_clear(bp: &mut SapBroadphase) {
    bp.aabbs.clear();
}

/// Returns the number of overlapping pairs (re-queries each call).
#[allow(dead_code)]
pub fn sap_overlap_count(bp: &SapBroadphase) -> usize {
    sap_query_overlaps(bp).len()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_aabb(cx: f32, cy: f32, cz: f32, half: f32) -> SapAabb {
        SapAabb {
            min: [cx - half, cy - half, cz - half],
            max: [cx + half, cy + half, cz + half],
        }
    }

    #[test]
    fn test_default_config() {
        let cfg = default_sap_config();
        assert_eq!(cfg.sort_axis, 0);
    }

    #[test]
    fn test_insert_and_count() {
        let cfg = default_sap_config();
        let mut bp = new_sap_broadphase(&cfg);
        sap_insert(&mut bp, 1, unit_aabb(0.0, 0.0, 0.0, 0.5));
        sap_insert(&mut bp, 2, unit_aabb(5.0, 0.0, 0.0, 0.5));
        assert_eq!(sap_aabb_count(&bp), 2);
    }

    #[test]
    fn test_no_overlap_when_separated() {
        let cfg = default_sap_config();
        let mut bp = new_sap_broadphase(&cfg);
        sap_insert(&mut bp, 0, unit_aabb(0.0, 0.0, 0.0, 0.4));
        sap_insert(&mut bp, 1, unit_aabb(2.0, 0.0, 0.0, 0.4));
        assert_eq!(sap_query_overlaps(&bp).len(), 0);
    }

    #[test]
    fn test_overlap_detected() {
        let cfg = default_sap_config();
        let mut bp = new_sap_broadphase(&cfg);
        sap_insert(&mut bp, 0, unit_aabb(0.0, 0.0, 0.0, 1.0));
        sap_insert(&mut bp, 1, unit_aabb(0.5, 0.0, 0.0, 1.0));
        let pairs = sap_query_overlaps(&bp);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].id_a, 0);
        assert_eq!(pairs[0].id_b, 1);
    }

    #[test]
    fn test_remove() {
        let cfg = default_sap_config();
        let mut bp = new_sap_broadphase(&cfg);
        sap_insert(&mut bp, 42, unit_aabb(0.0, 0.0, 0.0, 1.0));
        assert!(sap_remove(&mut bp, 42));
        assert!(!sap_remove(&mut bp, 42));
        assert_eq!(sap_aabb_count(&bp), 0);
    }

    #[test]
    fn test_clear() {
        let cfg = default_sap_config();
        let mut bp = new_sap_broadphase(&cfg);
        for i in 0..5u32 {
            sap_insert(&mut bp, i, unit_aabb(i as f32 * 10.0, 0.0, 0.0, 0.5));
        }
        sap_clear(&mut bp);
        assert_eq!(sap_aabb_count(&bp), 0);
    }

    #[test]
    fn test_aabb_overlaps_touching() {
        // Touching exactly on the boundary => overlaps
        let a = SapAabb { min: [0.0, 0.0, 0.0], max: [1.0, 1.0, 1.0] };
        let b = SapAabb { min: [1.0, 0.0, 0.0], max: [2.0, 1.0, 1.0] };
        assert!(aabb_overlaps(&a, &b));
    }

    #[test]
    fn test_update_replaces_aabb() {
        let cfg = default_sap_config();
        let mut bp = new_sap_broadphase(&cfg);
        sap_insert(&mut bp, 1, unit_aabb(0.0, 0.0, 0.0, 0.5));
        sap_insert(&mut bp, 2, unit_aabb(5.0, 0.0, 0.0, 0.5));
        // No overlap initially
        assert_eq!(sap_overlap_count(&bp), 0);
        // Move body 2 close to body 1
        sap_update(&mut bp, 2, unit_aabb(0.3, 0.0, 0.0, 0.5));
        assert_eq!(sap_overlap_count(&bp), 1);
    }

    #[test]
    fn test_three_way_overlaps() {
        let cfg = default_sap_config();
        let mut bp = new_sap_broadphase(&cfg);
        sap_insert(&mut bp, 0, unit_aabb(0.0, 0.0, 0.0, 2.0));
        sap_insert(&mut bp, 1, unit_aabb(0.5, 0.0, 0.0, 2.0));
        sap_insert(&mut bp, 2, unit_aabb(1.0, 0.0, 0.0, 2.0));
        // All three mutually overlap
        assert_eq!(sap_overlap_count(&bp), 3);
    }
}
