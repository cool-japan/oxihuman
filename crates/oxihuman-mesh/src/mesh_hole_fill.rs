//! Mesh hole detection and filling by fan triangulation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HoleFillConfig {
    pub max_hole_size: usize,
    pub smooth_iterations: u32,
    pub flat_fill: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HoleFill {
    pub boundary_vertices: Vec<u32>,
    pub area_estimate: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HoleFillResult {
    pub holes_found: usize,
    pub holes_filled: usize,
    pub new_triangle_count: usize,
    pub new_triangles: Vec<[u32; 3]>,
}

#[allow(dead_code)]
pub fn default_hole_fill_config() -> HoleFillConfig {
    HoleFillConfig {
        max_hole_size: 64,
        smooth_iterations: 2,
        flat_fill: true,
    }
}

/// Detect holes in the mesh by finding boundary edge loops.
/// An edge is a boundary edge if it appears only once across all edge pairs.
#[allow(dead_code)]
pub fn detect_holes(edges: &[[u32; 2]], vertex_count: usize) -> Vec<HoleFill> {
    use std::collections::HashMap;

    // Count how many times each directed edge appears
    let mut edge_count: HashMap<(u32, u32), usize> = HashMap::new();
    for e in edges {
        let key = (e[0].min(e[1]), e[0].max(e[1]));
        *edge_count.entry(key).or_insert(0) += 1;
    }

    // Boundary edges appear exactly once
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for ((a, b), cnt) in &edge_count {
        if *cnt == 1 {
            adj.entry(*a).or_default().push(*b);
            adj.entry(*b).or_default().push(*a);
        }
    }

    // Walk boundary loops
    let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut holes = Vec::new();

    for start in 0..vertex_count as u32 {
        if !adj.contains_key(&start) || visited.contains(&start) {
            continue;
        }
        // Walk the loop
        let mut loop_verts = Vec::new();
        let mut current = start;
        let mut prev = u32::MAX;
        loop {
            if visited.contains(&current) {
                break;
            }
            visited.insert(current);
            loop_verts.push(current);
            let neighbors = match adj.get(&current) {
                Some(n) => n,
                None => break,
            };
            let next = neighbors.iter().find(|&&v| v != prev && !visited.contains(&v));
            match next {
                Some(&v) => { prev = current; current = v; }
                None => break,
            }
        }
        if loop_verts.len() >= 3 {
            holes.push(HoleFill { boundary_vertices: loop_verts, area_estimate: 0.0 });
        }
    }

    holes
}

/// Fan triangulate a hole from its first vertex.
#[allow(dead_code)]
pub fn fill_hole_fan(hole: &HoleFill) -> Vec<[u32; 3]> {
    let verts = &hole.boundary_vertices;
    if verts.len() < 3 {
        return Vec::new();
    }
    let pivot = verts[0];
    (1..verts.len() - 1)
        .map(|i| [pivot, verts[i], verts[i + 1]])
        .collect()
}

#[allow(dead_code)]
pub fn fill_all_holes(holes: &[HoleFill], cfg: &HoleFillConfig) -> HoleFillResult {
    let holes_found = holes.len();
    let mut holes_filled = 0usize;
    let mut new_triangles: Vec<[u32; 3]> = Vec::new();

    for hole in holes {
        if hole.boundary_vertices.len() > cfg.max_hole_size {
            continue;
        }
        let tris = fill_hole_fan(hole);
        if !tris.is_empty() {
            holes_filled += 1;
            new_triangles.extend_from_slice(&tris);
        }
    }

    let new_triangle_count = new_triangles.len();
    HoleFillResult { holes_found, holes_filled, new_triangle_count, new_triangles }
}

#[allow(dead_code)]
pub fn hole_boundary_length(hole: &HoleFill, positions: &[[f32; 3]]) -> f32 {
    let verts = &hole.boundary_vertices;
    if verts.len() < 2 {
        return 0.0;
    }
    let mut len = 0.0f32;
    for i in 0..verts.len() {
        let a = verts[i] as usize;
        let b = verts[(i + 1) % verts.len()] as usize;
        if a < positions.len() && b < positions.len() {
            let dx = positions[b][0] - positions[a][0];
            let dy = positions[b][1] - positions[a][1];
            let dz = positions[b][2] - positions[a][2];
            len += (dx * dx + dy * dy + dz * dz).sqrt();
        }
    }
    len
}

#[allow(dead_code)]
pub fn hole_vertex_count(hole: &HoleFill) -> usize {
    hole.boundary_vertices.len()
}

#[allow(dead_code)]
pub fn hole_to_json(hole: &HoleFill) -> String {
    let verts: Vec<String> = hole.boundary_vertices.iter().map(|v| v.to_string()).collect();
    format!(
        "{{\"vertex_count\":{},\"area_estimate\":{:.4},\"vertices\":[{}]}}",
        hole.boundary_vertices.len(),
        hole.area_estimate,
        verts.join(",")
    )
}

#[allow(dead_code)]
pub fn hole_fill_result_to_json(r: &HoleFillResult) -> String {
    format!(
        "{{\"holes_found\":{},\"holes_filled\":{},\"new_triangle_count\":{}}}",
        r.holes_found, r.holes_filled, r.new_triangle_count
    )
}

/// Sort holes largest first (by boundary vertex count).
#[allow(dead_code)]
pub fn holes_by_size(holes: &mut [HoleFill]) {
    holes.sort_by(|a, b| b.boundary_vertices.len().cmp(&a.boundary_vertices.len()));
}

/// Estimate hole area using the shoelace formula projected onto XZ plane.
#[allow(dead_code)]
pub fn estimate_hole_area(hole: &HoleFill, positions: &[[f32; 3]]) -> f32 {
    let verts = &hole.boundary_vertices;
    if verts.len() < 3 {
        return 0.0;
    }
    let mut area = 0.0f32;
    let n = verts.len();
    for i in 0..n {
        let a = verts[i] as usize;
        let b = verts[(i + 1) % n] as usize;
        if a < positions.len() && b < positions.len() {
            area += positions[a][0] * positions[b][2];
            area -= positions[b][0] * positions[a][2];
        }
    }
    (area * 0.5).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_hole() -> HoleFill {
        HoleFill {
            boundary_vertices: vec![0, 1, 2, 3],
            area_estimate: 1.0,
        }
    }

    #[test]
    fn default_config_sane() {
        let cfg = default_hole_fill_config();
        assert!(cfg.max_hole_size > 0);
        assert!(cfg.flat_fill);
    }

    #[test]
    fn fill_hole_fan_square_produces_two_tris() {
        let hole = square_hole();
        let tris = fill_hole_fan(&hole);
        assert_eq!(tris.len(), 2);
        // All triangles share pivot vertex 0
        for tri in &tris {
            assert_eq!(tri[0], 0);
        }
    }

    #[test]
    fn fill_hole_fan_triangle_produces_one_tri() {
        let hole = HoleFill { boundary_vertices: vec![0, 1, 2], area_estimate: 0.5 };
        let tris = fill_hole_fan(&hole);
        assert_eq!(tris.len(), 1);
        assert_eq!(tris[0], [0, 1, 2]);
    }

    #[test]
    fn fill_hole_fan_too_small_returns_empty() {
        let hole = HoleFill { boundary_vertices: vec![0, 1], area_estimate: 0.0 };
        let tris = fill_hole_fan(&hole);
        assert!(tris.is_empty());
    }

    #[test]
    fn fill_all_holes_respects_max_hole_size() {
        let cfg = HoleFillConfig { max_hole_size: 3, smooth_iterations: 0, flat_fill: true };
        let holes = vec![
            HoleFill { boundary_vertices: vec![0, 1, 2], area_estimate: 0.5 },
            HoleFill { boundary_vertices: vec![0, 1, 2, 3, 4], area_estimate: 2.0 },
        ];
        let result = fill_all_holes(&holes, &cfg);
        assert_eq!(result.holes_found, 2);
        assert_eq!(result.holes_filled, 1);
    }

    #[test]
    fn hole_vertex_count_returns_correct() {
        let hole = square_hole();
        assert_eq!(hole_vertex_count(&hole), 4);
    }

    #[test]
    fn hole_to_json_contains_vertex_count() {
        let hole = square_hole();
        let json = hole_to_json(&hole);
        assert!(json.contains("\"vertex_count\":4"));
    }

    #[test]
    fn hole_fill_result_to_json_contains_counts() {
        let r = HoleFillResult {
            holes_found: 2,
            holes_filled: 1,
            new_triangle_count: 4,
            new_triangles: vec![],
        };
        let json = hole_fill_result_to_json(&r);
        assert!(json.contains("\"holes_found\":2"));
        assert!(json.contains("\"holes_filled\":1"));
        assert!(json.contains("\"new_triangle_count\":4"));
    }

    #[test]
    fn holes_by_size_sorts_largest_first() {
        let mut holes = vec![
            HoleFill { boundary_vertices: vec![0, 1, 2], area_estimate: 0.5 },
            HoleFill { boundary_vertices: vec![0, 1, 2, 3, 4, 5], area_estimate: 3.0 },
            HoleFill { boundary_vertices: vec![0, 1, 2, 3], area_estimate: 1.0 },
        ];
        holes_by_size(&mut holes);
        assert_eq!(holes[0].boundary_vertices.len(), 6);
        assert_eq!(holes[1].boundary_vertices.len(), 4);
        assert_eq!(holes[2].boundary_vertices.len(), 3);
    }

    #[test]
    fn hole_boundary_length_square() {
        let hole = HoleFill { boundary_vertices: vec![0, 1, 2, 3], area_estimate: 1.0 };
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let len = hole_boundary_length(&hole, &positions);
        assert!((len - 4.0).abs() < 1e-5, "perimeter of unit square = 4, got {len}");
    }

    #[test]
    fn estimate_hole_area_unit_square() {
        let hole = HoleFill { boundary_vertices: vec![0, 1, 2, 3], area_estimate: 0.0 };
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let area = estimate_hole_area(&hole, &positions);
        assert!((area - 1.0).abs() < 1e-5, "unit square XZ area = 1, got {area}");
    }
}
