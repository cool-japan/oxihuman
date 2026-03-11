//! Edge fillet (round) and chamfer (bevel) operations.

#[allow(dead_code)]
pub struct FilletResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub new_vert_count: usize,
}

#[allow(dead_code)]
pub struct ChamferResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub new_vert_count: usize,
}

#[allow(dead_code)]
pub struct FilletConfig {
    pub radius: f32,
    pub segments: u32,
    pub angle_threshold_deg: f32,
}

/// Returns a default fillet configuration.
#[allow(dead_code)]
pub fn default_fillet_config() -> FilletConfig {
    FilletConfig {
        radius: 0.05,
        segments: 4,
        angle_threshold_deg: 30.0,
    }
}

/// Split an edge at `amount` from each end, returning the two new points.
#[allow(dead_code)]
pub fn blend_edge_points(p0: [f32; 3], p1: [f32; 3], amount: f32) -> ([f32; 3], [f32; 3]) {
    let lerp = |a: [f32; 3], b: [f32; 3], t: f32| -> [f32; 3] {
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]
    };
    let dx = p1[0] - p0[0];
    let dy = p1[1] - p0[1];
    let dz = p1[2] - p0[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    let t = if len > 1e-8 {
        (amount / len).min(0.5)
    } else {
        0.0
    };
    (lerp(p0, p1, t), lerp(p1, p0, t))
}

/// Compute the chamfer amount given radius and angle.
#[allow(dead_code)]
pub fn chamfer_amount_from_radius(radius: f32, angle_deg: f32) -> f32 {
    let half = (angle_deg * std::f32::consts::PI / 180.0) / 2.0;
    if half.cos().abs() < 1e-8 {
        radius
    } else {
        radius / half.tan()
    }
}

/// Chamfer a single edge by inserting two new vertices.
#[allow(dead_code)]
pub fn chamfer_edge(
    positions: &[[f32; 3]],
    indices: &[u32],
    edge: [u32; 2],
    amount: f32,
) -> ChamferResult {
    let p0 = positions[edge[0] as usize];
    let p1 = positions[edge[1] as usize];
    let (c0, c1) = blend_edge_points(p0, p1, amount);

    let mut new_positions = positions.to_vec();
    let n0 = new_positions.len() as u32;
    new_positions.push(c0);
    let n1 = new_positions.len() as u32;
    new_positions.push(c1);

    let mut new_indices = indices.to_vec();
    // Add a chamfer triangle between the two new points and the midpoint
    let mid = [
        (p0[0] + p1[0]) / 2.0,
        (p0[1] + p1[1]) / 2.0,
        (p0[2] + p1[2]) / 2.0,
    ];
    let nm = new_positions.len() as u32;
    new_positions.push(mid);
    new_indices.push(n0);
    new_indices.push(nm);
    new_indices.push(n1);

    let new_vert_count = new_positions.len();
    ChamferResult {
        positions: new_positions,
        indices: new_indices,
        new_vert_count,
    }
}

/// Chamfer multiple edges.
#[allow(dead_code)]
pub fn chamfer_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    edges: &[[u32; 2]],
    amount: f32,
) -> ChamferResult {
    let mut cur_positions = positions.to_vec();
    let mut cur_indices = indices.to_vec();

    for edge in edges {
        let result = chamfer_edge(&cur_positions, &cur_indices, *edge, amount);
        cur_positions = result.positions;
        cur_indices = result.indices;
    }

    let new_vert_count = cur_positions.len();
    ChamferResult {
        positions: cur_positions,
        indices: cur_indices,
        new_vert_count,
    }
}

/// Generate arc points between start and end around center.
#[allow(dead_code)]
pub fn arc_points(
    center: [f32; 3],
    start: [f32; 3],
    end: [f32; 3],
    segments: u32,
) -> Vec<[f32; 3]> {
    if segments == 0 {
        return vec![start, end];
    }
    let r0 = [
        start[0] - center[0],
        start[1] - center[1],
        start[2] - center[2],
    ];
    let r1 = [end[0] - center[0], end[1] - center[1], end[2] - center[2]];
    let len0 = (r0[0] * r0[0] + r0[1] * r0[1] + r0[2] * r0[2]).sqrt();
    let len1 = (r1[0] * r1[0] + r1[1] * r1[1] + r1[2] * r1[2]).sqrt();
    if len0 < 1e-8 || len1 < 1e-8 {
        return vec![start, end];
    }
    let u = [r0[0] / len0, r0[1] / len0, r0[2] / len0];
    let v = [r1[0] / len1, r1[1] / len1, r1[2] / len1];
    let dot = (u[0] * v[0] + u[1] * v[1] + u[2] * v[2]).clamp(-1.0, 1.0);
    let angle = dot.acos();
    let radius = (len0 + len1) / 2.0;

    let count = (segments + 1) as usize;
    let mut pts = Vec::with_capacity(count + 1);
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let a = t * angle;
        // Slerp-like blend in plane of u and v
        let w0 = a.cos();
        let w1 = if angle.sin().abs() < 1e-8 {
            t
        } else {
            ((1.0 - t) * angle).sin() / angle.sin()
        };
        let w0b = if angle.sin().abs() < 1e-8 {
            1.0 - t
        } else {
            (t * angle).sin() / angle.sin()
        };
        let _ = w1; // suppress unused warning
        let dir = [
            w0b * u[0] + w0 * v[0],
            w0b * u[1] + w0 * v[1],
            w0b * u[2] + w0 * v[2],
        ];
        let dlen = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        let nd = if dlen > 1e-8 {
            [dir[0] / dlen, dir[1] / dlen, dir[2] / dlen]
        } else {
            u
        };
        pts.push([
            center[0] + nd[0] * radius,
            center[1] + nd[1] * radius,
            center[2] + nd[2] * radius,
        ]);
    }
    pts
}

/// Fillet a single edge with arc segments.
#[allow(dead_code)]
pub fn fillet_edge(
    positions: &[[f32; 3]],
    indices: &[u32],
    edge: [u32; 2],
    cfg: &FilletConfig,
) -> FilletResult {
    let p0 = positions[edge[0] as usize];
    let p1 = positions[edge[1] as usize];
    let (c0, c1) = blend_edge_points(p0, p1, cfg.radius);

    let mid = [
        (p0[0] + p1[0]) / 2.0,
        (p0[1] + p1[1]) / 2.0,
        (p0[2] + p1[2]) / 2.0,
    ];
    let arc = arc_points(mid, c0, c1, cfg.segments);

    let mut new_positions = positions.to_vec();
    let base = new_positions.len() as u32;
    for p in &arc {
        new_positions.push(*p);
    }

    let mut new_indices = indices.to_vec();
    for i in 0..(arc.len().saturating_sub(1)) {
        new_indices.push(base + i as u32);
        new_indices.push(base + i as u32 + 1);
        new_indices.push(edge[0]);
    }

    let new_vert_count = new_positions.len();
    FilletResult {
        positions: new_positions,
        indices: new_indices,
        new_vert_count,
    }
}

/// Bevel selected vertices.
#[allow(dead_code)]
pub fn bevel_vertices(
    positions: &[[f32; 3]],
    indices: &[u32],
    vert_mask: &[bool],
    amount: f32,
) -> ChamferResult {
    let mut new_positions = positions.to_vec();
    let mut new_indices = indices.to_vec();
    let orig_count = positions.len();

    // For each masked vertex find neighbors and create bevel verts
    // Build adjacency: collect edges from indices
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); orig_count];
    for chunk in indices.chunks(3) {
        if chunk.len() == 3 {
            let (a, b, c) = (chunk[0] as usize, chunk[1] as usize, chunk[2] as usize);
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[a].contains(&c) {
                adj[a].push(c);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
            if !adj[b].contains(&c) {
                adj[b].push(c);
            }
            if !adj[c].contains(&a) {
                adj[c].push(a);
            }
            if !adj[c].contains(&b) {
                adj[c].push(b);
            }
        }
    }

    for (vi, &masked) in vert_mask.iter().enumerate() {
        if !masked {
            continue;
        }
        let pv = positions[vi];
        for &neighbor in &adj[vi] {
            let pn = positions[neighbor];
            let (bv, _) = blend_edge_points(pv, pn, amount);
            let bi = new_positions.len() as u32;
            new_positions.push(bv);
            new_indices.push(vi as u32);
            new_indices.push(bi);
            new_indices.push(neighbor as u32);
        }
    }

    let new_vert_count = new_positions.len();
    ChamferResult {
        positions: new_positions,
        indices: new_indices,
        new_vert_count,
    }
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-8 {
        v
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn face_normal_from_tri(positions: &[[f32; 3]], i0: usize, i1: usize, i2: usize) -> [f32; 3] {
    let p0 = positions[i0];
    let p1 = positions[i1];
    let p2 = positions[i2];
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    normalize3(cross3(e1, e2))
}

/// Compute dihedral angle between two adjacent faces sharing an edge.
#[allow(dead_code)]
pub fn edge_dihedral_angle(positions: &[[f32; 3]], indices: &[u32], edge: [u32; 2]) -> f32 {
    let num_faces = indices.len() / 3;
    let mut face_normals: Vec<[f32; 3]> = Vec::new();

    for fi in 0..num_faces {
        let i0 = indices[fi * 3];
        let i1 = indices[fi * 3 + 1];
        let i2 = indices[fi * 3 + 2];
        let has_a = i0 == edge[0] || i1 == edge[0] || i2 == edge[0];
        let has_b = i0 == edge[1] || i1 == edge[1] || i2 == edge[1];
        if has_a && has_b {
            face_normals.push(face_normal_from_tri(
                positions,
                i0 as usize,
                i1 as usize,
                i2 as usize,
            ));
        }
    }

    if face_normals.len() < 2 {
        return 0.0;
    }
    let n0 = face_normals[0];
    let n1 = face_normals[1];
    let dot = (n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2]).clamp(-1.0, 1.0);
    dot.acos() * 180.0 / std::f32::consts::PI
}

/// Find edges whose dihedral angle exceeds the threshold.
#[allow(dead_code)]
pub fn find_sharp_edges(
    positions: &[[f32; 3]],
    indices: &[u32],
    angle_threshold_deg: f32,
) -> Vec<[u32; 2]> {
    let num_faces = indices.len() / 3;
    let mut edge_set: Vec<[u32; 2]> = Vec::new();

    for fi in 0..num_faces {
        let verts = [indices[fi * 3], indices[fi * 3 + 1], indices[fi * 3 + 2]];
        for k in 0..3 {
            let a = verts[k];
            let b = verts[(k + 1) % 3];
            let edge = if a < b { [a, b] } else { [b, a] };
            if !edge_set.contains(&edge) {
                edge_set.push(edge);
            }
        }
    }

    edge_set
        .into_iter()
        .filter(|&e| {
            let angle = edge_dihedral_angle(positions, indices, e);
            angle > angle_threshold_deg
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri_positions() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]]
    }

    fn tri_indices() -> Vec<u32> {
        vec![0, 1, 2]
    }

    #[test]
    fn test_default_fillet_config() {
        let cfg = default_fillet_config();
        assert!(cfg.radius > 0.0);
        assert!(cfg.segments > 0);
        assert!(cfg.angle_threshold_deg > 0.0);
    }

    #[test]
    fn test_chamfer_edge_more_verts() {
        let positions = tri_positions();
        let indices = tri_indices();
        let result = chamfer_edge(&positions, &indices, [0, 1], 0.1);
        assert!(result.positions.len() > positions.len());
        assert_eq!(result.new_vert_count, result.positions.len());
    }

    #[test]
    fn test_chamfer_edges_multiple() {
        let positions = tri_positions();
        let indices = tri_indices();
        let edges = vec![[0u32, 1], [1, 2]];
        let result = chamfer_edges(&positions, &indices, &edges, 0.1);
        assert!(result.positions.len() > positions.len());
    }

    #[test]
    fn test_blend_edge_points_distance() {
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [4.0f32, 0.0, 0.0];
        let (c0, c1) = blend_edge_points(p0, p1, 1.0);
        assert!((c0[0] - 1.0).abs() < 1e-5);
        assert!((c1[0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_edge_points_symmetric() {
        let p0 = [0.0f32, 0.0, 0.0];
        let p1 = [2.0f32, 0.0, 0.0];
        let (c0, c1) = blend_edge_points(p0, p1, 0.5);
        // c0 is amount from p0 toward p1; c1 is amount from p1 toward p0
        // They are symmetric around midpoint: c0[0] + c1[0] == len
        assert!((c0[0] + c1[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_fillet_edge_more_verts() {
        let positions = tri_positions();
        let indices = tri_indices();
        let cfg = default_fillet_config();
        let result = fillet_edge(&positions, &indices, [0, 1], &cfg);
        assert!(result.new_vert_count > positions.len());
    }

    #[test]
    fn test_arc_points_count() {
        let center = [0.5f32, 0.0, 0.0];
        let start = [0.0f32, 0.0, 0.0];
        let end = [1.0f32, 0.0, 0.0];
        let pts = arc_points(center, start, end, 4);
        assert_eq!(pts.len(), 5); // segments + 1
    }

    #[test]
    fn test_arc_points_zero_segments() {
        let center = [0.0f32, 0.0, 0.0];
        let start = [1.0f32, 0.0, 0.0];
        let end = [0.0f32, 1.0, 0.0];
        let pts = arc_points(center, start, end, 0);
        assert_eq!(pts.len(), 2);
    }

    #[test]
    fn test_chamfer_amount_from_radius() {
        let amt = chamfer_amount_from_radius(1.0, 90.0);
        assert!(amt > 0.0);
    }

    #[test]
    fn test_bevel_vertices_more_verts() {
        let positions = tri_positions();
        let indices = tri_indices();
        let vert_mask = vec![true, false, false];
        let result = bevel_vertices(&positions, &indices, &vert_mask, 0.1);
        assert!(result.positions.len() > positions.len());
    }

    #[test]
    fn test_edge_dihedral_no_adjacent() {
        // Only one face, so no dihedral
        let positions = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        let angle = edge_dihedral_angle(&positions, &indices, [0, 1]);
        assert_eq!(angle, 0.0);
    }

    #[test]
    fn test_find_sharp_edges_flat_mesh() {
        // Two coplanar triangles share an edge — dihedral ~ 0 → no sharp edges at 30 deg
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let sharp = find_sharp_edges(&positions, &indices, 30.0);
        assert!(sharp.is_empty());
    }

    #[test]
    fn test_fillet_config_segments_default() {
        let cfg = default_fillet_config();
        assert_eq!(cfg.segments, 4);
    }
}
