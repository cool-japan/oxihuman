//! Wireframe overlay rendering data.

#[allow(dead_code)]
pub struct WireframeConfig {
    pub color: [f32; 4],
    pub line_width: f32,
    pub depth_offset: f32,
    pub show_hidden: bool,
    pub crease_only: bool,
    pub crease_threshold_deg: f32,
}

#[allow(dead_code)]
pub struct WireEdge {
    pub v0: u32,
    pub v1: u32,
    pub is_crease: bool,
    pub is_boundary: bool,
}

#[allow(dead_code)]
pub struct WireframeOverlay {
    pub config: WireframeConfig,
    pub edges: Vec<WireEdge>,
    pub vertex_positions: Vec<[f32; 3]>,
    pub visible: bool,
}

#[allow(dead_code)]
pub fn default_wireframe_config() -> WireframeConfig {
    WireframeConfig {
        color: [1.0, 1.0, 1.0, 1.0],
        line_width: 1.0,
        depth_offset: -0.001,
        show_hidden: false,
        crease_only: false,
        crease_threshold_deg: 30.0,
    }
}

#[allow(dead_code)]
pub fn extract_edges(indices: &[u32]) -> Vec<[u32; 2]> {
    use std::collections::HashSet;
    let mut seen: HashSet<(u32, u32)> = HashSet::new();
    let mut edges = Vec::new();
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let tri_edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
        for (a, b) in tri_edges {
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                edges.push([key.0, key.1]);
            }
        }
    }
    edges
}

#[allow(dead_code)]
pub fn boundary_edges_wire(indices: &[u32]) -> Vec<[u32; 2]> {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let tri_edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
        for (a, b) in tri_edges {
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count
        .into_iter()
        .filter_map(|((a, b), count)| if count == 1 { Some([a, b]) } else { None })
        .collect()
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn triangle_normal(positions: &[[f32; 3]], a: u32, b: u32, c: u32) -> [f32; 3] {
    let pa = positions[a as usize];
    let pb = positions[b as usize];
    let pc = positions[c as usize];
    let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
    normalize3(cross3(ab, ac))
}

#[allow(dead_code)]
pub fn classify_crease_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    threshold_deg: f32,
) -> Vec<bool> {
    use std::collections::HashMap;
    let unique_edges = extract_edges(indices);
    let threshold_cos = threshold_deg.to_radians().cos();

    // Map each canonical edge to its adjacent triangle normals
    let mut edge_normals: HashMap<(u32, u32), Vec<[f32; 3]>> = HashMap::new();
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let normal = triangle_normal(positions, tri[0], tri[1], tri[2]);
        let tri_edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
        for (a, b) in tri_edges {
            let key = if a < b { (a, b) } else { (b, a) };
            edge_normals.entry(key).or_default().push(normal);
        }
    }

    unique_edges
        .iter()
        .map(|&[a, b]| {
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some(normals) = edge_normals.get(&key) {
                if normals.len() >= 2 {
                    let cos_angle = dot3(normals[0], normals[1]);
                    // Crease if angle between normals exceeds threshold
                    return cos_angle < threshold_cos;
                }
            }
            false
        })
        .collect()
}

#[allow(dead_code)]
pub fn build_wireframe(
    positions: &[[f32; 3]],
    indices: &[u32],
    cfg: &WireframeConfig,
) -> WireframeOverlay {
    let unique_edges = extract_edges(indices);
    let boundary = boundary_edges_wire(indices);
    let boundary_set: std::collections::HashSet<(u32, u32)> = boundary
        .iter()
        .map(|&[a, b]| if a < b { (a, b) } else { (b, a) })
        .collect();
    let crease_flags = classify_crease_edges(positions, indices, cfg.crease_threshold_deg);

    let edges: Vec<WireEdge> = unique_edges
        .iter()
        .zip(crease_flags.iter())
        .map(|(&[v0, v1], &is_crease)| {
            let key = if v0 < v1 { (v0, v1) } else { (v1, v0) };
            WireEdge {
                v0,
                v1,
                is_crease,
                is_boundary: boundary_set.contains(&key),
            }
        })
        .collect();

    WireframeOverlay {
        config: WireframeConfig {
            color: cfg.color,
            line_width: cfg.line_width,
            depth_offset: cfg.depth_offset,
            show_hidden: cfg.show_hidden,
            crease_only: cfg.crease_only,
            crease_threshold_deg: cfg.crease_threshold_deg,
        },
        edges,
        vertex_positions: positions.to_vec(),
        visible: true,
    }
}

#[allow(dead_code)]
pub fn wireframe_line_segments(overlay: &WireframeOverlay) -> Vec<([f32; 3], [f32; 3])> {
    overlay
        .edges
        .iter()
        .filter_map(|e| {
            let p0 = overlay.vertex_positions.get(e.v0 as usize)?;
            let p1 = overlay.vertex_positions.get(e.v1 as usize)?;
            Some((*p0, *p1))
        })
        .collect()
}

#[allow(dead_code)]
pub fn filter_crease_only(overlay: &mut WireframeOverlay) {
    overlay.edges.retain(|e| e.is_crease);
}

#[allow(dead_code)]
pub fn set_wireframe_color(overlay: &mut WireframeOverlay, color: [f32; 4]) {
    overlay.config.color = color;
}

#[allow(dead_code)]
pub fn edge_count_wire(overlay: &WireframeOverlay) -> usize {
    overlay.edges.len()
}

#[allow(dead_code)]
pub fn visible_edge_count(overlay: &WireframeOverlay) -> usize {
    if !overlay.visible {
        return 0;
    }
    overlay
        .edges
        .iter()
        .filter(|e| {
            if overlay.config.crease_only && !e.is_crease {
                return false;
            }
            true
        })
        .count()
}

#[allow(dead_code)]
pub fn wireframe_aabb(overlay: &WireframeOverlay) -> ([f32; 3], [f32; 3]) {
    if overlay.vertex_positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut min = overlay.vertex_positions[0];
    let mut max = overlay.vertex_positions[0];
    for pos in &overlay.vertex_positions {
        for i in 0..3 {
            if pos[i] < min[i] {
                min[i] = pos[i];
            }
            if pos[i] > max[i] {
                max[i] = pos[i];
            }
        }
    }
    (min, max)
}

#[allow(dead_code)]
pub fn merge_wireframes(mut a: WireframeOverlay, b: WireframeOverlay) -> WireframeOverlay {
    let offset = a.vertex_positions.len() as u32;
    a.vertex_positions.extend_from_slice(&b.vertex_positions);
    for e in &b.edges {
        a.edges.push(WireEdge {
            v0: e.v0 + offset,
            v1: e.v1 + offset,
            is_crease: e.is_crease,
            is_boundary: e.is_boundary,
        });
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_positions() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }

    fn triangle_indices() -> Vec<u32> {
        vec![0, 1, 2]
    }

    fn quad_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn quad_indices() -> Vec<u32> {
        vec![0, 1, 2, 0, 2, 3]
    }

    #[test]
    fn test_extract_edges_triangle() {
        let edges = extract_edges(&triangle_indices());
        // One triangle has 3 edges
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_extract_edges_quad() {
        let edges = extract_edges(&quad_indices());
        // Two triangles sharing one edge: 5 unique edges
        assert_eq!(edges.len(), 5);
    }

    #[test]
    fn test_extract_edges_unique() {
        let indices = vec![0, 1, 2, 0, 2, 1]; // degenerate, two tris same verts
        let edges = extract_edges(&indices);
        assert_eq!(edges.len(), 3, "Should deduplicate edges");
    }

    #[test]
    fn test_build_wireframe() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let cfg = default_wireframe_config();
        let overlay = build_wireframe(&pos, &idx, &cfg);
        assert_eq!(overlay.edges.len(), 3);
        assert_eq!(overlay.vertex_positions.len(), 3);
        assert!(overlay.visible);
    }

    #[test]
    fn test_boundary_edges_triangle() {
        let boundary = boundary_edges_wire(&triangle_indices());
        // Single triangle: all 3 edges are boundary
        assert_eq!(boundary.len(), 3);
    }

    #[test]
    fn test_boundary_edges_quad() {
        let boundary = boundary_edges_wire(&quad_indices());
        // Two triangles sharing one diagonal: 4 outer edges are boundary
        assert_eq!(boundary.len(), 4);
    }

    #[test]
    fn test_classify_crease_flat() {
        let pos = quad_positions();
        let idx = quad_indices();
        // Flat quad: no creases for any threshold
        let flags = classify_crease_edges(&pos, &idx, 30.0);
        assert_eq!(flags.len(), 5); // 5 unique edges
                                    // All same plane so no crease
        for flag in &flags {
            assert!(!flag, "Flat quad should have no creases");
        }
    }

    #[test]
    fn test_wireframe_line_segments() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let cfg = default_wireframe_config();
        let overlay = build_wireframe(&pos, &idx, &cfg);
        let segments = wireframe_line_segments(&overlay);
        assert_eq!(segments.len(), 3);
    }

    #[test]
    fn test_filter_crease_only() {
        let pos = quad_positions();
        let idx = quad_indices();
        let cfg = default_wireframe_config();
        let mut overlay = build_wireframe(&pos, &idx, &cfg);
        // Flat mesh: all edges are non-crease
        filter_crease_only(&mut overlay);
        assert_eq!(overlay.edges.len(), 0, "Flat mesh should have no creases");
    }

    #[test]
    fn test_edge_count_wire() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let cfg = default_wireframe_config();
        let overlay = build_wireframe(&pos, &idx, &cfg);
        assert_eq!(edge_count_wire(&overlay), 3);
    }

    #[test]
    fn test_visible_edge_count() {
        let pos = quad_positions();
        let idx = quad_indices();
        let cfg = default_wireframe_config();
        let overlay = build_wireframe(&pos, &idx, &cfg);
        assert_eq!(visible_edge_count(&overlay), 5);
    }

    #[test]
    fn test_wireframe_aabb() {
        let pos = vec![[0.0, 0.0, 0.0], [2.0, 3.0, 4.0], [-1.0, 0.0, 0.0]];
        let idx = vec![0, 1, 2];
        let cfg = default_wireframe_config();
        let overlay = build_wireframe(&pos, &idx, &cfg);
        let (min, max) = wireframe_aabb(&overlay);
        assert!((min[0] - (-1.0)).abs() < 1e-6);
        assert!((max[0] - 2.0).abs() < 1e-6);
        assert!((max[1] - 3.0).abs() < 1e-6);
        assert!((max[2] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_merge_wireframes() {
        let cfg = default_wireframe_config();
        let pos_a = triangle_positions();
        let idx_a = triangle_indices();
        let a = build_wireframe(&pos_a, &idx_a, &cfg);

        let pos_b = vec![[5.0, 0.0, 0.0], [6.0, 0.0, 0.0], [5.0, 1.0, 0.0]];
        let idx_b = vec![0, 1, 2];
        let b = build_wireframe(&pos_b, &idx_b, &cfg);

        let merged = merge_wireframes(a, b);
        assert_eq!(merged.vertex_positions.len(), 6);
        assert_eq!(merged.edges.len(), 6);
    }

    #[test]
    fn test_set_wireframe_color() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let cfg = default_wireframe_config();
        let mut overlay = build_wireframe(&pos, &idx, &cfg);
        let new_color = [1.0, 0.0, 0.0, 1.0];
        set_wireframe_color(&mut overlay, new_color);
        assert_eq!(overlay.config.color, new_color);
    }
}

// ── WireframeOverlayConfig / WireframeOverlayState (overlay toggle API) ───────

/// Simple toggle configuration for wireframe overlay on a solid mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WireframeOverlayConfig {
    pub color: [f32; 4],
    pub line_width: f32,
    pub only_edges: bool,
    pub show_all_edges: bool,
}

/// Runtime state for wireframe overlay toggle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WireframeOverlayState {
    pub enabled: bool,
    pub config: WireframeOverlayConfig,
}

#[allow(dead_code)]
pub fn default_wireframe_overlay_config() -> WireframeOverlayConfig {
    WireframeOverlayConfig {
        color: [0.0, 0.0, 0.0, 1.0],
        line_width: 1.0,
        only_edges: false,
        show_all_edges: true,
    }
}

#[allow(dead_code)]
pub fn new_wireframe_overlay_state() -> WireframeOverlayState {
    WireframeOverlayState {
        enabled: false,
        config: default_wireframe_overlay_config(),
    }
}

#[allow(dead_code)]
pub fn wo_set_enabled(state: &mut WireframeOverlayState, v: bool) {
    state.enabled = v;
}

#[allow(dead_code)]
pub fn wo_is_enabled(state: &WireframeOverlayState) -> bool {
    state.enabled
}

#[allow(dead_code)]
pub fn wo_set_color(state: &mut WireframeOverlayState, color: [f32; 4]) {
    state.config.color = color;
}

#[allow(dead_code)]
pub fn wo_set_line_width(state: &mut WireframeOverlayState, w: f32) {
    state.config.line_width = w.max(0.1);
}

#[allow(dead_code)]
pub fn wo_to_json(state: &WireframeOverlayState) -> String {
    let c = &state.config.color;
    format!(
        r#"{{"enabled":{},"color":[{:.4},{:.4},{:.4},{:.4}],"line_width":{:.4},"only_edges":{},"show_all_edges":{}}}"#,
        state.enabled,
        c[0],
        c[1],
        c[2],
        c[3],
        state.config.line_width,
        state.config.only_edges,
        state.config.show_all_edges
    )
}

#[allow(dead_code)]
pub fn wo_reset(state: &mut WireframeOverlayState) {
    *state = new_wireframe_overlay_state();
}

#[cfg(test)]
mod overlay_tests {
    use super::*;

    #[test]
    fn test_wo_default_config() {
        let cfg = default_wireframe_overlay_config();
        assert!((cfg.line_width - 1.0).abs() < 1e-6);
        assert!(!cfg.only_edges);
        assert!(cfg.show_all_edges);
    }

    #[test]
    fn test_wo_new_state_disabled() {
        let s = new_wireframe_overlay_state();
        assert!(!s.enabled);
    }

    #[test]
    fn test_wo_set_enabled() {
        let mut s = new_wireframe_overlay_state();
        wo_set_enabled(&mut s, true);
        assert!(wo_is_enabled(&s));
    }

    #[test]
    fn test_wo_set_color() {
        let mut s = new_wireframe_overlay_state();
        wo_set_color(&mut s, [1.0, 0.5, 0.0, 0.8]);
        assert!((s.config.color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_wo_set_line_width_clamps() {
        let mut s = new_wireframe_overlay_state();
        wo_set_line_width(&mut s, -1.0);
        assert!(s.config.line_width >= 0.1);
    }

    #[test]
    fn test_wo_set_line_width_valid() {
        let mut s = new_wireframe_overlay_state();
        wo_set_line_width(&mut s, 2.5);
        assert!((s.config.line_width - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_wo_to_json_contains_fields() {
        let s = new_wireframe_overlay_state();
        let j = wo_to_json(&s);
        assert!(j.contains("enabled"));
        assert!(j.contains("color"));
        assert!(j.contains("line_width"));
    }

    #[test]
    fn test_wo_reset() {
        let mut s = new_wireframe_overlay_state();
        wo_set_enabled(&mut s, true);
        wo_set_line_width(&mut s, 3.0);
        wo_reset(&mut s);
        assert!(!s.enabled);
        assert!((s.config.line_width - 1.0).abs() < 1e-6);
    }
}
