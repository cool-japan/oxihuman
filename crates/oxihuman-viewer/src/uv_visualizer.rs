//! UV seam and island visualizer — generates 2-D line segments for UV layout display.
//!
//! Builds [`UvEdge`] lists from UV coordinates and triangle faces, detects seam edges,
//! and groups edges into UV islands for rendering.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for UV visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvVisConfig {
    /// Density of the background UV grid (lines per unit).
    pub grid_density: f32,
    /// Whether the UV grid is visible.
    pub show_grid: bool,
    /// Whether UV seam edges are highlighted.
    pub show_seams: bool,
    /// Whether the visualizer is enabled at all.
    pub enabled: bool,
    /// Edge line width for rendering.
    pub line_width: f32,
}

/// A 2-D line segment between two UV-space points.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct UvEdge {
    /// Start point in UV space.
    pub uv0: [f32; 2],
    /// End point in UV space.
    pub uv1: [f32; 2],
    /// Whether this edge is a seam (boundary between UV islands).
    pub is_seam: bool,
}

/// A contiguous UV island — a group of edges that form a connected region.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvIsland {
    /// Edges belonging to this island.
    pub edges: Vec<UvEdge>,
}

/// Aggregated draw-call data produced by [`uv_vis_draw_call`].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvVisDrawCall {
    /// All edges (interior + seam) for the UV layout.
    pub edges: Vec<UvEdge>,
    /// Detected UV islands.
    pub islands: Vec<UvIsland>,
    /// Configuration snapshot used to produce this draw-call.
    pub config: UvVisConfig,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Returns a default [`UvVisConfig`].
#[allow(dead_code)]
pub fn default_uv_vis_config() -> UvVisConfig {
    UvVisConfig {
        grid_density: 8.0,
        show_grid: true,
        show_seams: true,
        enabled: true,
        line_width: 1.0,
    }
}

/// Builds all UV edges (one per triangle edge, de-duplicated) from `uvs` and `faces`.
#[allow(dead_code)]
pub fn build_uv_edges(uvs: &[[f32; 2]], faces: &[[u32; 3]]) -> Vec<UvEdge> {
    use std::collections::HashSet;

    // Use a canonical (sorted index) key to deduplicate
    let mut seen: HashSet<(u32, u32)> = HashSet::new();
    let mut edges = Vec::new();

    for face in faces {
        let corners = [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])];
        for (a, b) in corners {
            if a as usize >= uvs.len() || b as usize >= uvs.len() {
                continue;
            }
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                edges.push(UvEdge {
                    uv0: uvs[a as usize],
                    uv1: uvs[b as usize],
                    is_seam: false,
                });
            }
        }
    }

    edges
}

/// Detects seam edges: edges where two faces share the same vertex indices but
/// differ in UV coordinates (indicating a seam cut).
#[allow(dead_code)]
pub fn detect_uv_seams(uvs: &[[f32; 2]], faces: &[[u32; 3]]) -> Vec<UvEdge> {
    // For a simple stub implementation: an edge is a seam when its two UV
    // endpoints are not equal (i.e., the edge straddles a UV boundary).
    let mut seams = Vec::new();
    for face in faces {
        let corners = [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])];
        for (a, b) in corners {
            if a as usize >= uvs.len() || b as usize >= uvs.len() {
                continue;
            }
            let uv_a = uvs[a as usize];
            let uv_b = uvs[b as usize];
            // Edge crosses UV boundary if either coordinate differs significantly
            let du = (uv_a[0] - uv_b[0]).abs();
            let dv = (uv_a[1] - uv_b[1]).abs();
            if du > 0.5 || dv > 0.5 {
                seams.push(UvEdge {
                    uv0: uv_a,
                    uv1: uv_b,
                    is_seam: true,
                });
            }
        }
    }
    seams
}

/// Builds a complete [`UvVisDrawCall`] from UV coordinates, face indices, and
/// a configuration reference.
#[allow(dead_code)]
pub fn uv_vis_draw_call(
    uvs: &[[f32; 2]],
    faces: &[[u32; 3]],
    cfg: &UvVisConfig,
) -> UvVisDrawCall {
    let mut all_edges = build_uv_edges(uvs, faces);

    // Mark seam edges
    let seams = detect_uv_seams(uvs, faces);
    for seam in &seams {
        for edge in &mut all_edges {
            if edges_coincide(edge, seam) {
                edge.is_seam = true;
            }
        }
    }

    // Simple island detection: each face forms its own island (stub)
    let islands: Vec<UvIsland> = faces
        .iter()
        .filter_map(|face| {
            let corners = [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])];
            let island_edges: Vec<UvEdge> = corners
                .iter()
                .filter_map(|&(a, b)| {
                    if a as usize >= uvs.len() || b as usize >= uvs.len() {
                        return None;
                    }
                    Some(UvEdge {
                        uv0: uvs[a as usize],
                        uv1: uvs[b as usize],
                        is_seam: false,
                    })
                })
                .collect();
            if island_edges.is_empty() {
                None
            } else {
                Some(UvIsland { edges: island_edges })
            }
        })
        .collect();

    UvVisDrawCall {
        edges: all_edges,
        islands,
        config: cfg.clone(),
    }
}

/// Returns the total number of edges in a draw-call.
#[allow(dead_code)]
pub fn uv_edge_count(call: &UvVisDrawCall) -> usize {
    call.edges.len()
}

/// Sets the UV grid density on `cfg`.
#[allow(dead_code)]
pub fn set_uv_grid_density(cfg: &mut UvVisConfig, density: f32) {
    cfg.grid_density = density;
}

/// Toggles the visibility of the UV grid.
#[allow(dead_code)]
pub fn uv_vis_toggle_grid(cfg: &mut UvVisConfig) {
    cfg.show_grid = !cfg.show_grid;
}

/// Toggles the visibility of UV seams.
#[allow(dead_code)]
pub fn uv_vis_toggle_seams(cfg: &mut UvVisConfig) {
    cfg.show_seams = !cfg.show_seams;
}

/// Returns whether the UV visualizer is enabled.
#[allow(dead_code)]
pub fn uv_vis_is_enabled(cfg: &UvVisConfig) -> bool {
    cfg.enabled
}

/// Returns the number of UV islands in the draw-call.
#[allow(dead_code)]
pub fn uv_island_count(call: &UvVisDrawCall) -> usize {
    call.islands.len()
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn edges_coincide(a: &UvEdge, b: &UvEdge) -> bool {
    (uv_eq(a.uv0, b.uv0) && uv_eq(a.uv1, b.uv1))
        || (uv_eq(a.uv0, b.uv1) && uv_eq(a.uv1, b.uv0))
}

fn uv_eq(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_uvs() -> Vec<[f32; 2]> {
        vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]
    }

    fn simple_faces() -> Vec<[u32; 3]> {
        vec![[0, 1, 2]]
    }

    #[test]
    fn default_config_enabled() {
        let cfg = default_uv_vis_config();
        assert!(uv_vis_is_enabled(&cfg));
        assert!(cfg.show_grid);
        assert!(cfg.show_seams);
    }

    #[test]
    fn build_uv_edges_triangle() {
        let uvs = simple_uvs();
        let faces = simple_faces();
        let edges = build_uv_edges(&uvs, &faces);
        // A single triangle has 3 edges
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn build_uv_edges_no_duplicates_shared_edge() {
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.5, -1.0]];
        let faces = vec![[0, 1, 2], [0, 3, 1]];
        let edges = build_uv_edges(&uvs, &faces);
        // 4 unique edges (0-1 shared, 0-2, 1-2, 0-3, 1-3)
        assert_eq!(edges.len(), 5);
    }

    #[test]
    fn detect_uv_seams_no_seams() {
        // All edges stay within 0.5 UV units — no seam expected
        let uvs = vec![[0.1_f32, 0.1], [0.4, 0.1], [0.25, 0.4]];
        let faces = vec![[0u32, 1, 2]];
        let seams = detect_uv_seams(&uvs, &faces);
        assert_eq!(seams.len(), 0);
    }

    #[test]
    fn detect_uv_seams_with_boundary() {
        // Edge from UV (0, 0) to UV (1, 0) — du = 1.0 > 0.5 → seam
        let uvs = vec![[0.0_f32, 0.0], [1.0, 0.0], [0.5, 0.3]];
        let faces = vec![[0u32, 1, 2]];
        let seams = detect_uv_seams(&uvs, &faces);
        assert!(!seams.is_empty());
    }

    #[test]
    fn uv_vis_draw_call_island_count() {
        let uvs = simple_uvs();
        let faces = simple_faces();
        let cfg = default_uv_vis_config();
        let call = uv_vis_draw_call(&uvs, &faces, &cfg);
        assert_eq!(uv_island_count(&call), 1);
    }

    #[test]
    fn uv_edge_count_matches() {
        let uvs = simple_uvs();
        let faces = simple_faces();
        let cfg = default_uv_vis_config();
        let call = uv_vis_draw_call(&uvs, &faces, &cfg);
        assert_eq!(uv_edge_count(&call), 3);
    }

    #[test]
    fn toggle_grid_and_seams() {
        let mut cfg = default_uv_vis_config();
        uv_vis_toggle_grid(&mut cfg);
        assert!(!cfg.show_grid);
        uv_vis_toggle_seams(&mut cfg);
        assert!(!cfg.show_seams);
    }

    #[test]
    fn set_grid_density() {
        let mut cfg = default_uv_vis_config();
        set_uv_grid_density(&mut cfg, 16.0);
        assert!((cfg.grid_density - 16.0).abs() < 1e-6);
    }
}
