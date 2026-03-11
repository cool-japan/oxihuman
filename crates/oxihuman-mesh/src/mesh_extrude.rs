//! Face/edge/vertex extrusion operations.

#[allow(dead_code)]
pub struct ExtrudeResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub new_face_start: u32,
}

#[allow(dead_code)]
pub enum ExtrudeMode {
    Individual,
    Region,
    AlongNormal,
}

#[allow(dead_code)]
pub struct ExtrudeConfig {
    pub distance: f32,
    pub scale: f32,
    pub mode: ExtrudeMode,
    pub cap: bool,
}

/// Returns a default extrude configuration.
#[allow(dead_code)]
pub fn default_extrude_config() -> ExtrudeConfig {
    ExtrudeConfig {
        distance: 0.1,
        scale: 1.0,
        mode: ExtrudeMode::AlongNormal,
        cap: true,
    }
}

/// Compute the face normal for a polygon given by vertex indices.
#[allow(dead_code)]
pub fn compute_face_normal(positions: &[[f32; 3]], face: &[u32]) -> [f32; 3] {
    if face.len() < 3 {
        return [0.0, 1.0, 0.0];
    }
    let p0 = positions[face[0] as usize];
    let p1 = positions[face[1] as usize];
    let p2 = positions[face[2] as usize];
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-8 {
        [0.0, 1.0, 0.0]
    } else {
        [n[0] / len, n[1] / len, n[2] / len]
    }
}

/// Move all vertices along their normals by distance.
#[allow(dead_code)]
pub fn extrude_distance(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    _indices: &[u32],
    distance: f32,
) -> Vec<[f32; 3]> {
    positions
        .iter()
        .zip(normals.iter())
        .map(|(p, n)| {
            [
                p[0] + n[0] * distance,
                p[1] + n[1] * distance,
                p[2] + n[2] * distance,
            ]
        })
        .collect()
}

/// Extrude selected faces along their normals.
#[allow(dead_code)]
pub fn extrude_faces(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    indices: &[u32],
    face_mask: &[bool],
    cfg: &ExtrudeConfig,
) -> ExtrudeResult {
    let mut new_positions: Vec<[f32; 3]> = positions.to_vec();
    let mut new_indices: Vec<u32> = indices.to_vec();
    let original_count = positions.len() as u32;
    let new_face_start = original_count;

    let num_faces = indices.len() / 3;
    for fi in 0..num_faces {
        let masked = fi < face_mask.len() && face_mask[fi];
        if !masked {
            continue;
        }
        let i0 = indices[fi * 3] as usize;
        let i1 = indices[fi * 3 + 1] as usize;
        let i2 = indices[fi * 3 + 2] as usize;

        // Average normal for face
        let nx = (normals[i0][0] + normals[i1][0] + normals[i2][0]) / 3.0;
        let ny = (normals[i0][1] + normals[i1][1] + normals[i2][1]) / 3.0;
        let nz = (normals[i0][2] + normals[i1][2] + normals[i2][2]) / 3.0;

        let base = new_positions.len() as u32;
        for &vi in &[i0, i1, i2] {
            let p = positions[vi];
            new_positions.push([
                p[0] + nx * cfg.distance,
                p[1] + ny * cfg.distance,
                p[2] + nz * cfg.distance,
            ]);
        }
        // New extruded face
        new_indices.push(base);
        new_indices.push(base + 1);
        new_indices.push(base + 2);

        // Side quads (as triangles): connect original edge to extruded edge
        if cfg.cap {
            let orig = [i0 as u32, i1 as u32, i2 as u32];
            let ext = [base, base + 1, base + 2];
            for k in 0..3 {
                let a = orig[k];
                let b = orig[(k + 1) % 3];
                let c = ext[k];
                let d = ext[(k + 1) % 3];
                new_indices.push(a);
                new_indices.push(b);
                new_indices.push(c);
                new_indices.push(b);
                new_indices.push(d);
                new_indices.push(c);
            }
        }
    }

    ExtrudeResult {
        positions: new_positions,
        indices: new_indices,
        new_face_start,
    }
}

/// Extrude an edge loop into a surface along direction.
#[allow(dead_code)]
pub fn extrude_edges(
    positions: &[[f32; 3]],
    edges: &[[u32; 2]],
    direction: [f32; 3],
    distance: f32,
) -> ExtrudeResult {
    let mut new_positions: Vec<[f32; 3]> = positions.to_vec();
    let mut new_indices: Vec<u32> = Vec::new();
    let new_face_start = positions.len() as u32;

    for edge in edges {
        let a = edge[0] as usize;
        let b = edge[1] as usize;
        let pa = positions[a];
        let pb = positions[b];

        let c_idx = new_positions.len() as u32;
        new_positions.push([
            pa[0] + direction[0] * distance,
            pa[1] + direction[1] * distance,
            pa[2] + direction[2] * distance,
        ]);
        let d_idx = new_positions.len() as u32;
        new_positions.push([
            pb[0] + direction[0] * distance,
            pb[1] + direction[1] * distance,
            pb[2] + direction[2] * distance,
        ]);

        // Quad as two triangles
        new_indices.push(edge[0]);
        new_indices.push(edge[1]);
        new_indices.push(c_idx);

        new_indices.push(edge[1]);
        new_indices.push(d_idx);
        new_indices.push(c_idx);
    }

    ExtrudeResult {
        positions: new_positions,
        indices: new_indices,
        new_face_start,
    }
}

/// Extrude selected vertices along a direction.
#[allow(dead_code)]
pub fn extrude_vertices(
    positions: &[[f32; 3]],
    vert_mask: &[bool],
    direction: [f32; 3],
    distance: f32,
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut new_positions: Vec<[f32; 3]> = positions.to_vec();
    let mut new_indices: Vec<u32> = Vec::new();

    for (i, &masked) in vert_mask.iter().enumerate() {
        if !masked {
            continue;
        }
        let p = positions[i];
        let new_idx = new_positions.len() as u32;
        new_positions.push([
            p[0] + direction[0] * distance,
            p[1] + direction[1] * distance,
            p[2] + direction[2] * distance,
        ]);
        new_indices.push(i as u32);
        new_indices.push(new_idx);
    }

    (new_positions, new_indices)
}

/// Inset (shrink) selected faces inward.
#[allow(dead_code)]
pub fn inset_faces(
    positions: &[[f32; 3]],
    indices: &[u32],
    face_mask: &[bool],
    amount: f32,
) -> ExtrudeResult {
    let mut new_positions: Vec<[f32; 3]> = positions.to_vec();
    let mut new_indices: Vec<u32> = indices.to_vec();
    let new_face_start = positions.len() as u32;

    let num_faces = indices.len() / 3;
    for fi in 0..num_faces {
        let masked = fi < face_mask.len() && face_mask[fi];
        if !masked {
            continue;
        }
        let i0 = indices[fi * 3] as usize;
        let i1 = indices[fi * 3 + 1] as usize;
        let i2 = indices[fi * 3 + 2] as usize;

        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];

        // Centroid
        let cx = (p0[0] + p1[0] + p2[0]) / 3.0;
        let cy = (p0[1] + p1[1] + p2[1]) / 3.0;
        let cz = (p0[2] + p1[2] + p2[2]) / 3.0;

        let inset_pt = |p: [f32; 3]| -> [f32; 3] {
            let dx = cx - p[0];
            let dy = cy - p[1];
            let dz = cz - p[2];
            [p[0] + dx * amount, p[1] + dy * amount, p[2] + dz * amount]
        };

        let base = new_positions.len() as u32;
        new_positions.push(inset_pt(p0));
        new_positions.push(inset_pt(p1));
        new_positions.push(inset_pt(p2));

        new_indices.push(base);
        new_indices.push(base + 1);
        new_indices.push(base + 2);
    }

    ExtrudeResult {
        positions: new_positions,
        indices: new_indices,
        new_face_start,
    }
}

/// Create a shell mesh with two sides (solidify).
#[allow(dead_code)]
pub fn solidify_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    indices: &[u32],
    thickness: f32,
) -> ExtrudeResult {
    let n = positions.len();
    let mut new_positions: Vec<[f32; 3]> = positions.to_vec();
    let new_face_start = n as u32;

    // Offset copy
    for (p, norm) in positions.iter().zip(normals.iter()) {
        new_positions.push([
            p[0] + norm[0] * thickness,
            p[1] + norm[1] * thickness,
            p[2] + norm[2] * thickness,
        ]);
    }

    let mut new_indices: Vec<u32> = indices.to_vec();
    // Reversed winding for back side
    for chunk in indices.chunks(3) {
        if chunk.len() == 3 {
            new_indices.push(chunk[0] + n as u32);
            new_indices.push(chunk[2] + n as u32);
            new_indices.push(chunk[1] + n as u32);
        }
    }

    ExtrudeResult {
        positions: new_positions,
        indices: new_indices,
        new_face_start,
    }
}

/// Extrude a profile mesh along a curve path.
#[allow(dead_code)]
pub fn extrude_along_curve(
    positions: &[[f32; 3]],
    indices: &[u32],
    curve_points: &[[f32; 3]],
) -> ExtrudeResult {
    if curve_points.is_empty() {
        return ExtrudeResult {
            positions: positions.to_vec(),
            indices: indices.to_vec(),
            new_face_start: 0,
        };
    }

    let n = positions.len();
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut new_indices: Vec<u32> = Vec::new();

    // Place a copy of the profile at each curve point (translate only)
    for cp in curve_points {
        let base_idx = new_positions.len() as u32;
        for p in positions {
            new_positions.push([p[0] + cp[0], p[1] + cp[1], p[2] + cp[2]]);
        }
        // Connect consecutive segments with quads
        if new_positions.len() > n {
            let prev_base = base_idx - n as u32;
            for chunk in indices.chunks(3) {
                if chunk.len() == 3 {
                    new_indices.push(prev_base + chunk[0]);
                    new_indices.push(prev_base + chunk[1]);
                    new_indices.push(base_idx + chunk[0]);

                    new_indices.push(prev_base + chunk[1]);
                    new_indices.push(base_idx + chunk[1]);
                    new_indices.push(base_idx + chunk[0]);
                }
            }
        }
    }

    ExtrudeResult {
        positions: new_positions,
        indices: new_indices,
        new_face_start: 0,
    }
}

/// Returns total vertex count after extrusion.
#[allow(dead_code)]
pub fn extrude_vertex_count(original: usize, extruded_count: usize) -> usize {
    original + extruded_count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn square_normals() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 1.0]; 4]
    }

    fn square_indices() -> Vec<u32> {
        vec![0, 1, 2, 0, 2, 3]
    }

    #[test]
    fn test_default_extrude_config() {
        let cfg = default_extrude_config();
        assert!(cfg.distance > 0.0);
        assert!(cfg.cap);
    }

    #[test]
    fn test_compute_face_normal_z() {
        let positions = square_positions();
        let face = [0u32, 1, 2];
        let n = compute_face_normal(&positions, &face);
        assert!((n[2].abs() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_face_normal_empty() {
        let positions = square_positions();
        let n = compute_face_normal(&positions, &[]);
        assert_eq!(n, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_extrude_distance_moves_verts() {
        let positions = square_positions();
        let normals = square_normals();
        let indices = square_indices();
        let moved = extrude_distance(&positions, &normals, &indices, 2.0);
        for p in &moved {
            assert!((p[2] - 2.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_extrude_distance_zero() {
        let positions = square_positions();
        let normals = square_normals();
        let indices = square_indices();
        let moved = extrude_distance(&positions, &normals, &indices, 0.0);
        for (orig, moved) in positions.iter().zip(moved.iter()) {
            assert!((orig[0] - moved[0]).abs() < 1e-6);
            assert!((orig[1] - moved[1]).abs() < 1e-6);
            assert!((orig[2] - moved[2]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_extrude_faces_more_vertices() {
        let positions = square_positions();
        let normals = square_normals();
        let indices = square_indices();
        let face_mask = vec![true, false];
        let cfg = default_extrude_config();
        let result = extrude_faces(&positions, &normals, &indices, &face_mask, &cfg);
        assert!(result.positions.len() > positions.len());
    }

    #[test]
    fn test_extrude_faces_new_face_start() {
        let positions = square_positions();
        let normals = square_normals();
        let indices = square_indices();
        let face_mask = vec![true, true];
        let cfg = default_extrude_config();
        let result = extrude_faces(&positions, &normals, &indices, &face_mask, &cfg);
        assert_eq!(result.new_face_start, positions.len() as u32);
    }

    #[test]
    fn test_extrude_edges_produces_quads() {
        let positions = square_positions();
        let edges: Vec<[u32; 2]> = vec![[0, 1], [1, 2]];
        let result = extrude_edges(&positions, &edges, [0.0, 0.0, 1.0], 1.0);
        assert!(result.positions.len() > positions.len());
        assert!(!result.indices.is_empty());
    }

    #[test]
    fn test_extrude_vertices_along_direction() {
        let positions = square_positions();
        let vert_mask = vec![true, false, true, false];
        let (new_pos, new_idx) = extrude_vertices(&positions, &vert_mask, [0.0, 1.0, 0.0], 1.0);
        assert_eq!(new_pos.len(), positions.len() + 2);
        assert!(!new_idx.is_empty());
    }

    #[test]
    fn test_inset_shrinks_face() {
        let positions = square_positions();
        let indices = square_indices();
        let face_mask = vec![true, false];
        let result = inset_faces(&positions, &indices, &face_mask, 0.5);
        assert!(result.positions.len() > positions.len());
        // New positions should be closer to centroid
        let cx = (0.0 + 1.0 + 1.0) / 3.0_f32;
        let cy = (0.0 + 0.0 + 1.0) / 3.0_f32;
        // The 3 new verts at the end
        let base = positions.len();
        let p = result.positions[base];
        let dist = ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt();
        assert!(dist < 1.0);
    }

    #[test]
    fn test_solidify_doubles_vertex_count() {
        let positions = square_positions();
        let normals = square_normals();
        let indices = square_indices();
        let result = solidify_mesh(&positions, &normals, &indices, 0.1);
        assert_eq!(result.positions.len(), positions.len() * 2);
    }

    #[test]
    fn test_solidify_has_both_sides() {
        let positions = square_positions();
        let normals = square_normals();
        let indices = square_indices();
        let result = solidify_mesh(&positions, &normals, &indices, 0.5);
        assert!(result.indices.len() > indices.len());
    }

    #[test]
    fn test_extrude_along_curve() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        let curve = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, 2.0]];
        let result = extrude_along_curve(&positions, &indices, &curve);
        assert_eq!(result.positions.len(), positions.len() * curve.len());
    }

    #[test]
    fn test_extrude_vertex_count() {
        assert_eq!(extrude_vertex_count(10, 5), 15);
    }

    #[test]
    fn test_extrude_along_curve_empty_curve() {
        let positions = vec![[0.0, 0.0, 0.0]];
        let indices = vec![0u32];
        let result = extrude_along_curve(&positions, &indices, &[]);
        assert_eq!(result.positions.len(), 1);
    }
}
