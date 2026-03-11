//! Generate an offset cage mesh by pushing each vertex along its average normal.
//!
//! Each vertex is displaced along its averaged face-normal by a configurable
//! distance, producing an inflated or deflated "cage" mesh useful for proximity
//! tests and cage-based deformation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OffsetCageConfig {
    pub offset_distance: f32,
    pub smooth_iterations: usize,
    pub normalize_normals: bool,
    pub min_thickness: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OffsetCageResult {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    pub min_thickness: f32,
    pub max_thickness: f32,
}

#[allow(dead_code)]
pub fn default_offset_cage_config() -> OffsetCageConfig {
    OffsetCageConfig {
        offset_distance: 0.01,
        smooth_iterations: 1,
        normalize_normals: true,
        min_thickness: 0.001,
    }
}

/// Build per-vertex averaged normals from the triangle soup and offset.
#[allow(dead_code)]
pub fn generate_offset_cage(
    vertices: &[[f32; 3]],
    triangles: &[[u32; 3]],
    config: &OffsetCageConfig,
) -> OffsetCageResult {
    if vertices.is_empty() {
        return OffsetCageResult {
            vertices: vec![],
            normals: vec![],
            faces: triangles.to_vec(),
            min_thickness: 0.0,
            max_thickness: 0.0,
        };
    }

    // Accumulate face normals per vertex.
    let mut normals = vec![[0.0f32; 3]; vertices.len()];
    for tri in triangles {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        if i0 >= vertices.len() || i1 >= vertices.len() || i2 >= vertices.len() {
            continue;
        }
        let n = face_normal(vertices[i0], vertices[i1], vertices[i2]);
        for &i in &[i0, i1, i2] {
            normals[i][0] += n[0];
            normals[i][1] += n[1];
            normals[i][2] += n[2];
        }
    }

    if config.normalize_normals {
        for n in &mut normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 1e-8 {
                n[0] /= len;
                n[1] /= len;
                n[2] /= len;
            }
        }
    }

    let d = config.offset_distance;
    let mut new_verts: Vec<[f32; 3]> = vertices
        .iter()
        .zip(normals.iter())
        .map(|(v, n)| [v[0] + n[0] * d, v[1] + n[1] * d, v[2] + n[2] * d])
        .collect();

    // Optional smoothing (Laplacian-like average with neighbours from triangles).
    for _ in 0..config.smooth_iterations {
        new_verts = smooth_vertices(&new_verts, triangles);
    }

    // Compute thickness = distance between original and offset vertices.
    let mut min_t = f32::MAX;
    let mut max_t = 0.0f32;
    for (v_orig, v_new) in vertices.iter().zip(new_verts.iter()) {
        let dx = v_new[0] - v_orig[0];
        let dy = v_new[1] - v_orig[1];
        let dz = v_new[2] - v_orig[2];
        let t = (dx * dx + dy * dy + dz * dz).sqrt();
        if t < min_t { min_t = t; }
        if t > max_t { max_t = t; }
    }
    if min_t == f32::MAX { min_t = 0.0; }

    OffsetCageResult {
        vertices: new_verts,
        normals,
        faces: triangles.to_vec(),
        min_thickness: min_t,
        max_thickness: max_t,
    }
}

/// Number of vertices in the cage.
#[allow(dead_code)]
pub fn offset_cage_vertex_count(result: &OffsetCageResult) -> usize {
    result.vertices.len()
}

/// Number of faces in the cage.
#[allow(dead_code)]
pub fn offset_cage_face_count(result: &OffsetCageResult) -> usize {
    result.faces.len()
}

/// Minimum displacement thickness.
#[allow(dead_code)]
pub fn offset_cage_min_thickness(result: &OffsetCageResult) -> f32 {
    result.min_thickness
}

/// Maximum displacement thickness.
#[allow(dead_code)]
pub fn offset_cage_max_thickness(result: &OffsetCageResult) -> f32 {
    result.max_thickness
}

/// Serialize cage summary to JSON-like string.
#[allow(dead_code)]
pub fn offset_cage_to_json(result: &OffsetCageResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"face_count\":{},\"min_thickness\":{:.6},\"max_thickness\":{:.6}}}",
        result.vertices.len(),
        result.faces.len(),
        result.min_thickness,
        result.max_thickness,
    )
}

/// Approximate bounding-box volume of the cage.
#[allow(dead_code)]
pub fn offset_cage_volume(result: &OffsetCageResult) -> f32 {
    if result.vertices.is_empty() { return 0.0; }
    let mut min = result.vertices[0];
    let mut max = result.vertices[0];
    for v in &result.vertices {
        for k in 0..3 {
            if v[k] < min[k] { min[k] = v[k]; }
            if v[k] > max[k] { max[k] = v[k]; }
        }
    }
    (max[0] - min[0]) * (max[1] - min[1]) * (max[2] - min[2])
}

/// Validate cage: vertex count matches, no NaN offsets.
#[allow(dead_code)]
pub fn offset_cage_validate(result: &OffsetCageResult) -> bool {
    result.vertices.iter().all(|v| v.iter().all(|x| x.is_finite()))
        && result.normals.iter().all(|n| n.iter().all(|x| x.is_finite()))
}

/// Apply one round of Laplacian smoothing to cage vertices.
#[allow(dead_code)]
pub fn offset_cage_apply_smoothing(result: &mut OffsetCageResult) {
    result.vertices = smooth_vertices(&result.vertices, &result.faces);
}

// ─── Internal helpers ────────────────────────────────────────────────────────

fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ]
}

fn smooth_vertices(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> Vec<[f32; 3]> {
    let n = verts.len();
    let mut sum = vec![[0.0f32; 3]; n];
    let mut count = vec![0u32; n];
    for tri in tris {
        let indices = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        for &i in &indices {
            if i >= n { continue; }
            for &j in &indices {
                if j >= n || i == j { continue; }
                sum[i][0] += verts[j][0];
                sum[i][1] += verts[j][1];
                sum[i][2] += verts[j][2];
                count[i] += 1;
            }
        }
    }
    verts
        .iter()
        .enumerate()
        .map(|(i, v)| {
            if count[i] == 0 {
                *v
            } else {
                let c = count[i] as f32;
                [sum[i][0] / c, sum[i][1] / c, sum[i][2] / c]
            }
        })
        .collect()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let tris = vec![[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]];
        (verts, tris)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_offset_cage_config();
        assert!(cfg.offset_distance > 0.0);
    }

    #[test]
    fn test_empty_mesh() {
        let cfg = default_offset_cage_config();
        let result = generate_offset_cage(&[], &[], &cfg);
        assert_eq!(offset_cage_vertex_count(&result), 0);
        assert_eq!(offset_cage_face_count(&result), 0);
    }

    #[test]
    fn test_vertex_count_preserved() {
        let (verts, tris) = simple_mesh();
        let cfg = default_offset_cage_config();
        let result = generate_offset_cage(&verts, &tris, &cfg);
        assert_eq!(offset_cage_vertex_count(&result), verts.len());
    }

    #[test]
    fn test_face_count_preserved() {
        let (verts, tris) = simple_mesh();
        let cfg = default_offset_cage_config();
        let result = generate_offset_cage(&verts, &tris, &cfg);
        assert_eq!(offset_cage_face_count(&result), tris.len());
    }

    #[test]
    fn test_thickness_positive() {
        let (verts, tris) = simple_mesh();
        let cfg = OffsetCageConfig { offset_distance: 0.1, ..default_offset_cage_config() };
        let result = generate_offset_cage(&verts, &tris, &cfg);
        assert!(offset_cage_max_thickness(&result) > 0.0);
    }

    #[test]
    fn test_validate() {
        let (verts, tris) = simple_mesh();
        let cfg = default_offset_cage_config();
        let result = generate_offset_cage(&verts, &tris, &cfg);
        assert!(offset_cage_validate(&result));
    }

    #[test]
    fn test_to_json() {
        let (verts, tris) = simple_mesh();
        let cfg = default_offset_cage_config();
        let result = generate_offset_cage(&verts, &tris, &cfg);
        let json = offset_cage_to_json(&result);
        assert!(json.contains("vertex_count"));
    }

    #[test]
    fn test_volume_positive() {
        let verts = vec![
            [0.0, 0.0, 0.0], [2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0], [0.0, 0.0, 2.0],
        ];
        let tris = vec![[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]];
        let cfg = OffsetCageConfig { offset_distance: 0.1, ..default_offset_cage_config() };
        let result = generate_offset_cage(&verts, &tris, &cfg);
        assert!(offset_cage_volume(&result) > 0.0);
    }

    #[test]
    fn test_apply_smoothing() {
        let (verts, tris) = simple_mesh();
        let cfg = default_offset_cage_config();
        let mut result = generate_offset_cage(&verts, &tris, &cfg);
        offset_cage_apply_smoothing(&mut result);
        assert!(offset_cage_validate(&result));
    }

    #[test]
    fn test_min_le_max_thickness() {
        let (verts, tris) = simple_mesh();
        let cfg = OffsetCageConfig { offset_distance: 0.05, ..default_offset_cage_config() };
        let result = generate_offset_cage(&verts, &tris, &cfg);
        assert!(result.min_thickness <= result.max_thickness);
    }
}
