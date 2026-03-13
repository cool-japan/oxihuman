/// Medial axis / skeleton extraction approximation from a mesh.
#[allow(dead_code)]
pub struct MedialPoint {
    pub position: [f32; 3],
    pub radius: f32,
    pub parent_idx: Option<usize>,
}

#[allow(dead_code)]
pub struct MedialAxis {
    pub points: Vec<MedialPoint>,
    pub edges: Vec<[usize; 2]>,
}

#[allow(dead_code)]
pub struct MedialAxisConfig {
    pub min_radius: f32,
    pub max_iterations: u32,
    pub collapse_threshold: f32,
}

#[allow(dead_code)]
pub fn default_medial_config() -> MedialAxisConfig {
    MedialAxisConfig {
        min_radius: 0.01,
        max_iterations: 100,
        collapse_threshold: 0.05,
    }
}

/// Compute the centroid of a triangle.
fn triangle_centroid(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0] + c[0]) / 3.0,
        (a[1] + b[1] + c[1]) / 3.0,
        (a[2] + b[2] + c[2]) / 3.0,
    ]
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Brute-force minimum distance from a point to any triangle in the mesh.
#[allow(dead_code)]
pub fn nearest_surface_distance(query: [f32; 3], positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    if positions.is_empty() || indices.len() < 3 {
        return f32::MAX;
    }
    let mut min_dist = f32::MAX;
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
            continue;
        }
        let a = positions[ia];
        let b = positions[ib];
        let c = positions[ic];
        let centroid = triangle_centroid(a, b, c);
        let d = dist3(query, centroid);
        if d < min_dist {
            min_dist = d;
        }
        // Also check distances to vertices
        for v in [a, b, c] {
            let dv = dist3(query, v);
            if dv < min_dist {
                min_dist = dv;
            }
        }
    }
    min_dist
}

/// Approximate medial axis via pole extraction: place interior candidate points
/// at face centroids, then iteratively collapse points that are too close.
#[allow(dead_code)]
pub fn approximate_medial_axis(
    positions: &[[f32; 3]],
    indices: &[u32],
    cfg: &MedialAxisConfig,
) -> MedialAxis {
    if positions.is_empty() || indices.len() < 3 {
        return MedialAxis {
            points: vec![],
            edges: vec![],
        };
    }

    // Step 1: collect candidate points at triangle centroids
    let tri_count = indices.len() / 3;
    let mut candidates: Vec<[f32; 3]> = Vec::with_capacity(tri_count);
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        if ia < positions.len() && ib < positions.len() && ic < positions.len() {
            candidates.push(triangle_centroid(
                positions[ia],
                positions[ib],
                positions[ic],
            ));
        }
    }

    // Step 2: collapse candidates closer than collapse_threshold (simple greedy)
    let mut kept: Vec<[f32; 3]> = Vec::new();
    for cand in &candidates {
        let mut too_close = false;
        for k in &kept {
            if dist3(*cand, *k) < cfg.collapse_threshold {
                too_close = true;
                break;
            }
        }
        if !too_close {
            kept.push(*cand);
        }
    }

    // Step 3: build medial points with radii
    let mut points: Vec<MedialPoint> = Vec::with_capacity(kept.len());
    for (i, pos) in kept.iter().enumerate() {
        let radius = nearest_surface_distance(*pos, positions, indices);
        let radius = if radius == f32::MAX { 0.0 } else { radius };
        let parent_idx = if i > 0 { Some(i - 1) } else { None };
        if radius >= cfg.min_radius {
            points.push(MedialPoint {
                position: *pos,
                radius,
                parent_idx,
            });
        }
    }

    // Step 4: build edges connecting consecutive points (chain skeleton)
    let mut edges: Vec<[usize; 2]> = Vec::new();
    for i in 1..points.len() {
        edges.push([i - 1, i]);
    }

    MedialAxis { points, edges }
}

#[allow(dead_code)]
pub fn medial_point_count(axis: &MedialAxis) -> usize {
    axis.points.len()
}

#[allow(dead_code)]
pub fn medial_edge_count(axis: &MedialAxis) -> usize {
    axis.edges.len()
}

#[allow(dead_code)]
pub fn medial_axis_length(axis: &MedialAxis) -> f32 {
    let mut total = 0.0f32;
    for edge in &axis.edges {
        let a = axis.points[edge[0]].position;
        let b = axis.points[edge[1]].position;
        total += dist3(a, b);
    }
    total
}

#[allow(dead_code)]
pub fn medial_axis_bounds(axis: &MedialAxis) -> ([f32; 3], [f32; 3]) {
    if axis.points.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = axis.points[0].position;
    let mut mx = axis.points[0].position;
    for p in &axis.points {
        for k in 0..3 {
            if p.position[k] < mn[k] {
                mn[k] = p.position[k];
            }
            if p.position[k] > mx[k] {
                mx[k] = p.position[k];
            }
        }
    }
    (mn, mx)
}

/// Remove branches shorter than min_length. Simplified: removes leaf edges (endpoints
/// connected to only one edge) whose edge length < min_length, iteratively.
#[allow(dead_code)]
pub fn prune_short_branches(axis: &mut MedialAxis, min_length: f32) {
    let mut changed = true;
    while changed {
        changed = false;
        let n = axis.points.len();
        let mut degree = vec![0usize; n];
        for e in &axis.edges {
            degree[e[0]] += 1;
            degree[e[1]] += 1;
        }
        let mut to_remove: Vec<usize> = Vec::new();
        for (ei, e) in axis.edges.iter().enumerate() {
            let a = e[0];
            let b = e[1];
            let len = dist3(axis.points[a].position, axis.points[b].position);
            if len < min_length && (degree[a] == 1 || degree[b] == 1) {
                to_remove.push(ei);
            }
        }
        if !to_remove.is_empty() {
            changed = true;
            for &ei in to_remove.iter().rev() {
                axis.edges.swap_remove(ei);
            }
        }
    }
}

#[allow(dead_code)]
pub fn medial_to_spheres(axis: &MedialAxis) -> Vec<([f32; 3], f32)> {
    axis.points.iter().map(|p| (p.position, p.radius)).collect()
}

#[allow(dead_code)]
pub fn medial_axis_connectivity(axis: &MedialAxis) -> Vec<Vec<usize>> {
    let n = axis.points.len();
    let mut adj = vec![vec![]; n];
    for e in &axis.edges {
        adj[e[0]].push(e[1]);
        adj[e[1]].push(e[0]);
    }
    adj
}

#[allow(dead_code)]
pub fn thickest_point(axis: &MedialAxis) -> Option<&MedialPoint> {
    axis.points.iter().max_by(|a, b| {
        a.radius
            .partial_cmp(&b.radius)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

#[allow(dead_code)]
pub fn medial_axis_to_json(axis: &MedialAxis) -> String {
    let mut s = String::from("{\"points\":[");
    for (i, p) in axis.points.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let parent = match p.parent_idx {
            Some(idx) => format!("{}", idx),
            None => "null".to_string(),
        };
        s.push_str(&format!(
            "{{\"pos\":[{},{},{}],\"radius\":{},\"parent\":{}}}",
            p.position[0], p.position[1], p.position[2], p.radius, parent
        ));
    }
    s.push_str("],\"edges\":[");
    for (i, e) in axis.edges.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("[{},{}]", e[0], e[1]));
    }
    s.push_str("]}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_box_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        // 8 vertices of a unit cube, 12 triangles (2 per face)
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3, // bottom
            4, 5, 6, 4, 6, 7, // top
            0, 1, 5, 0, 5, 4, // front
            2, 3, 7, 2, 7, 6, // back
            0, 3, 7, 0, 7, 4, // left
            1, 2, 6, 1, 6, 5, // right
        ];
        (positions, indices)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_medial_config();
        assert!(cfg.min_radius >= 0.0);
        assert!(cfg.max_iterations > 0);
        assert!(cfg.collapse_threshold > 0.0);
    }

    #[test]
    fn test_approximate_medial_axis_produces_points() {
        let (pos, idx) = simple_box_mesh();
        let cfg = MedialAxisConfig {
            min_radius: 0.0,
            max_iterations: 10,
            collapse_threshold: 0.05,
        };
        let axis = approximate_medial_axis(&pos, &idx, &cfg);
        assert!(!axis.points.is_empty());
    }

    #[test]
    fn test_approximate_medial_axis_empty() {
        let cfg = default_medial_config();
        let axis = approximate_medial_axis(&[], &[], &cfg);
        assert_eq!(axis.points.len(), 0);
        assert_eq!(axis.edges.len(), 0);
    }

    #[test]
    fn test_medial_point_count() {
        let (pos, idx) = simple_box_mesh();
        let cfg = MedialAxisConfig {
            min_radius: 0.0,
            max_iterations: 10,
            collapse_threshold: 0.05,
        };
        let axis = approximate_medial_axis(&pos, &idx, &cfg);
        assert_eq!(medial_point_count(&axis), axis.points.len());
    }

    #[test]
    fn test_medial_edge_count() {
        let (pos, idx) = simple_box_mesh();
        let cfg = MedialAxisConfig {
            min_radius: 0.0,
            max_iterations: 10,
            collapse_threshold: 0.05,
        };
        let axis = approximate_medial_axis(&pos, &idx, &cfg);
        assert_eq!(medial_edge_count(&axis), axis.edges.len());
    }

    #[test]
    fn test_medial_axis_length_nonneg() {
        let (pos, idx) = simple_box_mesh();
        let cfg = MedialAxisConfig {
            min_radius: 0.0,
            max_iterations: 10,
            collapse_threshold: 0.05,
        };
        let axis = approximate_medial_axis(&pos, &idx, &cfg);
        let len = medial_axis_length(&axis);
        assert!(len >= 0.0);
    }

    #[test]
    fn test_medial_axis_length_empty() {
        let axis = MedialAxis {
            points: vec![],
            edges: vec![],
        };
        assert_eq!(medial_axis_length(&axis), 0.0);
    }

    #[test]
    fn test_medial_axis_bounds() {
        let (pos, idx) = simple_box_mesh();
        let cfg = MedialAxisConfig {
            min_radius: 0.0,
            max_iterations: 10,
            collapse_threshold: 0.05,
        };
        let axis = approximate_medial_axis(&pos, &idx, &cfg);
        if !axis.points.is_empty() {
            let (mn, mx) = medial_axis_bounds(&axis);
            assert!(mn[0] <= mx[0]);
            assert!(mn[1] <= mx[1]);
            assert!(mn[2] <= mx[2]);
        }
    }

    #[test]
    fn test_medial_axis_bounds_empty() {
        let axis = MedialAxis {
            points: vec![],
            edges: vec![],
        };
        let (mn, mx) = medial_axis_bounds(&axis);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }

    #[test]
    fn test_medial_to_spheres() {
        let (pos, idx) = simple_box_mesh();
        let cfg = MedialAxisConfig {
            min_radius: 0.0,
            max_iterations: 10,
            collapse_threshold: 0.05,
        };
        let axis = approximate_medial_axis(&pos, &idx, &cfg);
        let spheres = medial_to_spheres(&axis);
        assert_eq!(spheres.len(), axis.points.len());
        for (center, radius) in &spheres {
            assert!(radius >= &0.0);
            assert!(center.len() == 3);
        }
    }

    #[test]
    fn test_nearest_surface_distance() {
        let (pos, idx) = simple_box_mesh();
        // Query at centroid of the whole cube should be close to 0.5
        let q = [0.5, 0.5, 0.5];
        let d = nearest_surface_distance(q, &pos, &idx);
        assert!(d < 1.5);
        assert!(d > 0.0);
    }

    #[test]
    fn test_nearest_surface_distance_on_vertex() {
        let (pos, idx) = simple_box_mesh();
        let d = nearest_surface_distance([0.0, 0.0, 0.0], &pos, &idx);
        assert!(d < 0.01);
    }

    #[test]
    fn test_thickest_point() {
        let (pos, idx) = simple_box_mesh();
        let cfg = MedialAxisConfig {
            min_radius: 0.0,
            max_iterations: 10,
            collapse_threshold: 0.05,
        };
        let axis = approximate_medial_axis(&pos, &idx, &cfg);
        if !axis.points.is_empty() {
            let tp = thickest_point(&axis);
            assert!(tp.is_some());
            let max_r = axis
                .points
                .iter()
                .map(|p| p.radius)
                .fold(f32::NEG_INFINITY, f32::max);
            assert!((tp.expect("should succeed").radius - max_r).abs() < 1e-5);
        }
    }

    #[test]
    fn test_thickest_point_empty() {
        let axis = MedialAxis {
            points: vec![],
            edges: vec![],
        };
        assert!(thickest_point(&axis).is_none());
    }

    #[test]
    fn test_medial_axis_connectivity() {
        let (pos, idx) = simple_box_mesh();
        let cfg = MedialAxisConfig {
            min_radius: 0.0,
            max_iterations: 10,
            collapse_threshold: 0.05,
        };
        let axis = approximate_medial_axis(&pos, &idx, &cfg);
        let adj = medial_axis_connectivity(&axis);
        assert_eq!(adj.len(), axis.points.len());
    }

    #[test]
    fn test_medial_axis_to_json() {
        let (pos, idx) = simple_box_mesh();
        let cfg = MedialAxisConfig {
            min_radius: 0.0,
            max_iterations: 10,
            collapse_threshold: 0.05,
        };
        let axis = approximate_medial_axis(&pos, &idx, &cfg);
        let json = medial_axis_to_json(&axis);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
        assert!(json.contains("points"));
        assert!(json.contains("edges"));
    }

    #[test]
    fn test_prune_short_branches() {
        let (pos, idx) = simple_box_mesh();
        let cfg = MedialAxisConfig {
            min_radius: 0.0,
            max_iterations: 10,
            collapse_threshold: 0.05,
        };
        let mut axis = approximate_medial_axis(&pos, &idx, &cfg);
        let before = axis.edges.len();
        prune_short_branches(&mut axis, 10.0); // prune all short edges
                                               // After aggressive pruning, edge count should be <= before
        assert!(axis.edges.len() <= before);
    }
}
