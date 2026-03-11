// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cloth tearing and fracture mechanics.
//!
//! Provides a threshold-based tearing model: constraints that exceed a
//! configurable force limit are broken and the tear propagates edge by edge
//! through the mesh topology.

// ── Type aliases ─────────────────────────────────────────────────────────────

/// An edge represented as a pair of vertex indices `(a, b)` with `a < b`.
#[allow(dead_code)]
pub type Edge = (usize, usize);

/// A simple JSON blob (string).
#[allow(dead_code)]
pub type JsonBlob = String;

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration for the cloth-tearing system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothTearingConfig {
    /// Force threshold above which an edge constraint tears.
    pub tear_threshold: f32,
    /// Maximum number of edges a tear can propagate in one `propagate_tear` call.
    pub max_propagation_steps: usize,
    /// Proximity threshold for `merge_close_tears`.
    pub merge_distance: f32,
}

impl Default for ClothTearingConfig {
    fn default() -> Self {
        Self {
            tear_threshold: 50.0,
            max_propagation_steps: 5,
            merge_distance: 0.05,
        }
    }
}

// ── Data structures ───────────────────────────────────────────────────────────

/// A single cloth tear.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothTear {
    /// Unique identifier.
    pub id: u32,
    /// Edges that have been severed, in order.
    pub edges: Vec<Edge>,
    /// Whether this tear is still growing.
    pub active: bool,
    /// Maximum constraint force observed along this tear.
    pub peak_force: f32,
}

/// State controlling how a tear propagates through the mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TearPropagation {
    /// Tear identifier this propagation belongs to.
    pub tear_id: u32,
    /// Remaining propagation budget (decremented each step).
    pub remaining_steps: usize,
    /// Direction hint (next candidate edge to sever).
    pub frontier: Option<Edge>,
}

/// The overall cloth-tearing system state.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ClothTearingSystem {
    pub tears: Vec<ClothTear>,
    pub config: ClothTearingConfig,
    next_id: u32,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
fn make_edge(a: usize, b: usize) -> Edge {
    if a < b { (a, b) } else { (b, a) }
}

#[allow(dead_code)]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a `ClothTearingConfig` with sensible defaults.
#[allow(dead_code)]
pub fn default_tearing_config() -> ClothTearingConfig {
    ClothTearingConfig::default()
}

/// Create a new cloth tear starting at `initial_edge`.
/// Returns the tear ID.
#[allow(dead_code)]
pub fn new_cloth_tear(
    system: &mut ClothTearingSystem,
    initial_edge: Edge,
    peak_force: f32,
) -> u32 {
    let id = system.next_id;
    system.next_id += 1;
    system.tears.push(ClothTear {
        id,
        edges: vec![initial_edge],
        active: true,
        peak_force,
    });
    id
}

/// Test whether the force on a constraint exceeds the tear threshold.
#[allow(dead_code)]
pub fn check_tear_threshold(force: f32, config: &ClothTearingConfig) -> bool {
    force > config.tear_threshold
}

/// Extend tear `tear_id` by one edge chosen from `candidate_edges`.
///
/// Picks the candidate adjacent to the tear frontier with the highest `forces`
/// value (if provided), otherwise picks the first adjacent edge.
/// Returns the edge that was severed, or `None` if no candidate was found.
#[allow(dead_code)]
pub fn propagate_tear(
    system: &mut ClothTearingSystem,
    tear_id: u32,
    candidate_edges: &[Edge],
    forces: &[f32],
) -> Option<Edge> {
    let tear = system.tears.iter_mut().find(|t| t.id == tear_id)?;
    if !tear.active {
        return None;
    }

    let last_edge = *tear.edges.last()?;

    // Candidates adjacent to the last severed edge, not yet in this tear.
    let mut adjacent: Vec<Edge> = candidate_edges
        .iter()
        .filter(|&&e| {
            !tear.edges.contains(&e)
                && (e.0 == last_edge.0
                    || e.1 == last_edge.0
                    || e.0 == last_edge.1
                    || e.1 == last_edge.1)
        })
        .copied()
        .collect();

    if adjacent.is_empty() {
        tear.active = false;
        return None;
    }

    // If force data is available, pick the highest-force edge.
    let chosen = if !forces.is_empty() {
        adjacent.sort_by(|&a, &b| {
            let fa = forces.get(a.0).copied().unwrap_or(0.0)
                + forces.get(a.1).copied().unwrap_or(0.0);
            let fb = forces.get(b.0).copied().unwrap_or(0.0)
                + forces.get(b.1).copied().unwrap_or(0.0);
            fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
        });
        adjacent[0]
    } else {
        adjacent[0]
    };

    tear.peak_force = tear.peak_force.max(
        forces.get(chosen.0).copied().unwrap_or(0.0)
            + forces.get(chosen.1).copied().unwrap_or(0.0),
    );
    tear.edges.push(chosen);
    Some(chosen)
}

/// Apply a tear to the mesh by duplicating vertices along the torn edges.
///
/// Returns the indices of all newly created duplicate vertices.
#[allow(dead_code)]
pub fn apply_tear_to_mesh(
    positions: &mut Vec<[f32; 3]>,
    tear: &ClothTear,
) -> Vec<usize> {
    let mut new_indices = Vec::new();
    for &(a, b) in &tear.edges {
        let na = positions[a];
        let new_idx_a = positions.len();
        positions.push(na);
        new_indices.push(new_idx_a);

        let nb = positions[b];
        let new_idx_b = positions.len();
        positions.push(nb);
        new_indices.push(new_idx_b);
    }
    new_indices
}

/// Number of tears in the system.
#[allow(dead_code)]
pub fn tear_count(system: &ClothTearingSystem) -> usize {
    system.tears.len()
}

/// Total number of severed edges in a tear.
#[allow(dead_code)]
pub fn tear_length(tear: &ClothTear) -> usize {
    tear.edges.len()
}

/// Return references to all currently active (still-growing) tears.
#[allow(dead_code)]
pub fn active_tears(system: &ClothTearingSystem) -> Vec<&ClothTear> {
    system.tears.iter().filter(|t| t.active).collect()
}

/// Update the tear threshold in the configuration.
#[allow(dead_code)]
pub fn set_tear_threshold(config: &mut ClothTearingConfig, threshold: f32) {
    config.tear_threshold = threshold;
}

/// Merge tears whose edges share endpoints within `config.merge_distance`.
///
/// When two tears are merged the second is deactivated and its edges appended
/// to the first.  Returns the number of merges performed.
#[allow(dead_code)]
pub fn merge_close_tears(
    system: &mut ClothTearingSystem,
    positions: &[[f32; 3]],
) -> usize {
    let merge_dist = system.config.merge_distance;
    let mut merges = 0;
    let n = system.tears.len();
    for i in 0..n {
        for j in (i + 1)..n {
            if !system.tears[i].active || !system.tears[j].active {
                continue;
            }
            let close = system.tears[j].edges.iter().any(|&ej| {
                system.tears[i].edges.iter().any(|&ei| {
                    let pi0 = positions.get(ei.0).copied().unwrap_or([0.0; 3]);
                    let pj0 = positions.get(ej.0).copied().unwrap_or([0.0; 3]);
                    dist3(pi0, pj0) < merge_dist
                })
            });
            if close {
                let extra: Vec<Edge> = system.tears[j].edges.clone();
                system.tears[i].edges.extend(extra);
                system.tears[j].active = false;
                merges += 1;
            }
        }
    }
    merges
}

/// Serialise a `ClothTear` to a simple JSON string.
#[allow(dead_code)]
pub fn tear_to_json(tear: &ClothTear) -> JsonBlob {
    let edges_json: String = tear
        .edges
        .iter()
        .map(|(a, b)| format!("[{a},{b}]"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"id":{},"active":{},"peak_force":{:.4},"edges":[{}]}}"#,
        tear.id, tear.active, tear.peak_force, edges_json
    )
}

/// Remove all tears and reset the system ID counter.
#[allow(dead_code)]
pub fn reset_tears(system: &mut ClothTearingSystem) {
    system.tears.clear();
    system.next_id = 0;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_system() -> ClothTearingSystem {
        ClothTearingSystem::default()
    }

    #[test]
    fn test_default_tearing_config() {
        let cfg = default_tearing_config();
        assert!(cfg.tear_threshold > 0.0);
        assert!(cfg.max_propagation_steps > 0);
        assert!(cfg.merge_distance >= 0.0);
    }

    #[test]
    fn test_check_tear_threshold_above() {
        let cfg = default_tearing_config();
        assert!(check_tear_threshold(cfg.tear_threshold + 1.0, &cfg));
    }

    #[test]
    fn test_check_tear_threshold_below() {
        let cfg = default_tearing_config();
        assert!(!check_tear_threshold(cfg.tear_threshold - 1.0, &cfg));
    }

    #[test]
    fn test_check_tear_threshold_equal() {
        let cfg = default_tearing_config();
        // Equal to threshold → should NOT tear (strict >).
        assert!(!check_tear_threshold(cfg.tear_threshold, &cfg));
    }

    #[test]
    fn test_new_cloth_tear_creates_entry() {
        let mut sys = make_system();
        new_cloth_tear(&mut sys, (0, 1), 60.0);
        assert_eq!(tear_count(&sys), 1);
    }

    #[test]
    fn test_new_cloth_tear_ids_unique() {
        let mut sys = make_system();
        let id0 = new_cloth_tear(&mut sys, (0, 1), 60.0);
        let id1 = new_cloth_tear(&mut sys, (2, 3), 70.0);
        assert_ne!(id0, id1);
    }

    #[test]
    fn test_tear_length_initial() {
        let mut sys = make_system();
        let id = new_cloth_tear(&mut sys, (0, 1), 60.0);
        let tear = sys.tears.iter().find(|t| t.id == id).unwrap();
        assert_eq!(tear_length(tear), 1);
    }

    #[test]
    fn test_propagate_tear_extends() {
        let mut sys = make_system();
        let id = new_cloth_tear(&mut sys, (0, 1), 60.0);
        let candidates = vec![(1, 2), (1, 3)];
        let severed = propagate_tear(&mut sys, id, &candidates, &[]);
        assert!(severed.is_some());
        let tear = sys.tears.iter().find(|t| t.id == id).unwrap();
        assert_eq!(tear_length(tear), 2);
    }

    #[test]
    fn test_propagate_tear_no_candidates_deactivates() {
        let mut sys = make_system();
        let id = new_cloth_tear(&mut sys, (0, 1), 60.0);
        let severed = propagate_tear(&mut sys, id, &[], &[]);
        assert!(severed.is_none());
        let tear = sys.tears.iter().find(|t| t.id == id).unwrap();
        assert!(!tear.active);
    }

    #[test]
    fn test_active_tears_filters_inactive() {
        let mut sys = make_system();
        let id0 = new_cloth_tear(&mut sys, (0, 1), 60.0);
        let id1 = new_cloth_tear(&mut sys, (2, 3), 55.0);
        sys.tears.iter_mut().find(|t| t.id == id0).unwrap().active = false;
        let active = active_tears(&sys);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, id1);
    }

    #[test]
    fn test_apply_tear_to_mesh_adds_verts() {
        let mut sys = make_system();
        let id = new_cloth_tear(&mut sys, (0, 1), 60.0);
        let tear = sys.tears.iter().find(|t| t.id == id).unwrap().clone();
        let mut positions: Vec<[f32; 3]> =
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let before = positions.len();
        let new_idxs = apply_tear_to_mesh(&mut positions, &tear);
        assert!(positions.len() > before);
        assert!(!new_idxs.is_empty());
    }

    #[test]
    fn test_set_tear_threshold() {
        let mut cfg = default_tearing_config();
        set_tear_threshold(&mut cfg, 100.0);
        assert!((cfg.tear_threshold - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_reset_tears_clears() {
        let mut sys = make_system();
        new_cloth_tear(&mut sys, (0, 1), 60.0);
        new_cloth_tear(&mut sys, (2, 3), 55.0);
        reset_tears(&mut sys);
        assert_eq!(tear_count(&sys), 0);
    }

    #[test]
    fn test_tear_to_json_has_id() {
        let mut sys = make_system();
        let id = new_cloth_tear(&mut sys, (0, 1), 60.0);
        let tear = sys.tears.iter().find(|t| t.id == id).unwrap();
        let json = tear_to_json(tear);
        assert!(json.contains(&format!("\"id\":{id}")));
    }

    #[test]
    fn test_tear_to_json_has_edges_key() {
        let mut sys = make_system();
        let id = new_cloth_tear(&mut sys, (3, 7), 60.0);
        let tear = sys.tears.iter().find(|t| t.id == id).unwrap();
        let json = tear_to_json(tear);
        assert!(json.contains("edges"));
    }

    #[test]
    fn test_merge_close_tears_same_vertex() {
        let mut sys = make_system();
        sys.config.merge_distance = 0.5;
        new_cloth_tear(&mut sys, (0, 1), 60.0);
        new_cloth_tear(&mut sys, (0, 2), 55.0); // shares vertex 0
        let positions: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let merged = merge_close_tears(&mut sys, &positions);
        // Vertex 0 is at distance 0 from itself → should merge.
        assert_eq!(merged, 1);
    }

    #[test]
    fn test_propagate_with_forces() {
        let mut sys = make_system();
        let id = new_cloth_tear(&mut sys, (0, 1), 60.0);
        let candidates = vec![(1, 2), (1, 3)];
        // High force on vertex 3
        let forces = vec![0.0_f32, 10.0, 0.0, 100.0];
        let severed = propagate_tear(&mut sys, id, &candidates, &forces);
        assert_eq!(severed, Some((1, 3)));
    }

    #[test]
    fn test_tear_count_after_multiple_adds() {
        let mut sys = make_system();
        for i in 0..5_usize {
            new_cloth_tear(&mut sys, (i, i + 1), 60.0);
        }
        assert_eq!(tear_count(&sys), 5);
    }
}
