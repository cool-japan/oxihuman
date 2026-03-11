//! Compute morphed normals from delta vertices.
#![allow(dead_code)]

/// Result of a morph normal computation.
#[allow(dead_code)]
pub struct MorphNormalResult {
    pub normals: Vec<[f32; 3]>,
    pub vertex_count: usize,
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
    if len < 1e-10 { [0.0, 0.0, 1.0] } else { [v[0]/len, v[1]/len, v[2]/len] }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1]*b[2] - a[2]*b[1],
        a[2]*b[0] - a[0]*b[2],
        a[0]*b[1] - a[1]*b[0],
    ]
}

/// Compute normals for morphed positions given base positions + delta and triangle indices.
#[allow(dead_code)]
pub fn compute_morph_normals(
    positions: &[[f32; 3]],
    deltas: &[[f32; 3]],
    indices: &[u32],
) -> MorphNormalResult {
    let n = positions.len();
    let morphed: Vec<[f32; 3]> = positions.iter().zip(deltas.iter())
        .map(|(p, d)| [p[0]+d[0], p[1]+d[1], p[2]+d[2]])
        .collect();
    let normals = recompute_normals_from_positions(&morphed, indices);
    MorphNormalResult { vertex_count: n, normals }
}

/// Compute per-vertex normal deltas between two normal arrays.
#[allow(dead_code)]
pub fn morph_normal_delta(base: &[[f32; 3]], morphed: &[[f32; 3]]) -> Vec<[f32; 3]> {
    base.iter().zip(morphed.iter())
        .map(|(b, m)| [m[0]-b[0], m[1]-b[1], m[2]-b[2]])
        .collect()
}

/// Apply normal morph: add delta normals to base normals and renormalize.
#[allow(dead_code)]
pub fn apply_normal_morph(base: &[[f32; 3]], deltas: &[[f32; 3]], weight: f32) -> Vec<[f32; 3]> {
    base.iter().zip(deltas.iter())
        .map(|(b, d)| normalize3([b[0]+d[0]*weight, b[1]+d[1]*weight, b[2]+d[2]*weight]))
        .collect()
}

/// Normalize a list of morph normals in place.
#[allow(dead_code)]
pub fn normalize_morph_normals(normals: &[[f32; 3]]) -> Vec<[f32; 3]> {
    normals.iter().map(|n| normalize3(*n)).collect()
}

/// Smooth morph normals by averaging with neighbors (simple Laplacian over index pairs).
#[allow(dead_code)]
pub fn morph_normal_smooth(normals: &[[f32; 3]], iterations: usize) -> Vec<[f32; 3]> {
    let mut current = normals.to_vec();
    for _ in 0..iterations {
        let prev = current.clone();
        let n = prev.len();
        if n < 2 { break; }
        for i in 0..n {
            let left = if i == 0 { n - 1 } else { i - 1 };
            let right = (i + 1) % n;
            let nx = (prev[left][0] + prev[i][0] + prev[right][0]) / 3.0;
            let ny = (prev[left][1] + prev[i][1] + prev[right][1]) / 3.0;
            let nz = (prev[left][2] + prev[i][2] + prev[right][2]) / 3.0;
            current[i] = normalize3([nx, ny, nz]);
        }
    }
    current
}

/// Recompute normals from positions and triangle indices.
#[allow(dead_code)]
pub fn recompute_normals_from_positions(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut acc = vec![[0.0f32; 3]; n];
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let i0 = indices[t*3] as usize;
        let i1 = indices[t*3+1] as usize;
        let i2 = indices[t*3+2] as usize;
        if i0 >= n || i1 >= n || i2 >= n { continue; }
        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        let e1 = [p1[0]-p0[0], p1[1]-p0[1], p1[2]-p0[2]];
        let e2 = [p2[0]-p0[0], p2[1]-p0[1], p2[2]-p0[2]];
        let fn_ = cross3(e1, e2);
        for &idx in &[i0, i1, i2] {
            acc[idx][0] += fn_[0];
            acc[idx][1] += fn_[1];
            acc[idx][2] += fn_[2];
        }
    }
    acc.iter().map(|n| normalize3(*n)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recompute_normals_plane() {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[1.0,1.0,0.0]];
        let idx = vec![0u32,1,2, 1,3,2];
        let normals = recompute_normals_from_positions(&pos, &idx);
        assert_eq!(normals.len(), 4);
        for n in &normals {
            assert!((n[2].abs() - 1.0).abs() < 1e-4, "normal z should be 1: {:?}", n);
        }
    }

    #[test]
    fn test_morph_normal_delta_zero() {
        let a = vec![[0.0f32,0.0,1.0],[0.0,1.0,0.0]];
        let d = morph_normal_delta(&a, &a);
        for dd in &d { assert!(dd[0].abs() < 1e-6); }
    }

    #[test]
    fn test_apply_normal_morph_zero_weight() {
        let base = vec![[0.0f32,0.0,1.0]];
        let delta = vec![[0.0f32,1.0,0.0]];
        let r = apply_normal_morph(&base, &delta, 0.0);
        assert!((r[0][2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_morph_normals_unit() {
        let ns = vec![[3.0f32,0.0,0.0],[0.0,2.0,0.0]];
        let r = normalize_morph_normals(&ns);
        for n in &r {
            let l = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt();
            assert!((l-1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_morph_normal_smooth_count() {
        let ns = vec![[1.0f32,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]];
        let r = morph_normal_smooth(&ns, 2);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn test_compute_morph_normals_count() {
        let pos = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let deltas = vec![[0.0f32;3];3];
        let idx = vec![0u32,1,2];
        let r = compute_morph_normals(&pos, &deltas, &idx);
        assert_eq!(r.vertex_count, 3);
    }

    #[test]
    fn test_morph_normal_result_vertex_count() {
        let r = MorphNormalResult { normals: vec![[0.0,0.0,1.0]], vertex_count: 1 };
        assert_eq!(r.vertex_count, 1);
    }

    #[test]
    fn test_morph_normal_smooth_normalized() {
        let ns = vec![[1.0f32,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]];
        let r = morph_normal_smooth(&ns, 1);
        for n in &r {
            let l = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt();
            assert!((l-1.0).abs() < 1e-4);
        }
    }

    #[test]
    fn test_apply_morph_unit_result() {
        let base = vec![[1.0f32,0.0,0.0]];
        let delta = vec![[0.0f32,0.0,0.0]];
        let r = apply_normal_morph(&base, &delta, 1.0);
        let l = (r[0][0]*r[0][0]+r[0][1]*r[0][1]+r[0][2]*r[0][2]).sqrt();
        assert!((l-1.0).abs() < 1e-5);
    }
}
