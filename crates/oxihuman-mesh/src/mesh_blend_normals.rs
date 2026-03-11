//! Blend normals between two meshes by weight.
#![allow(dead_code)]

/// Configuration for normal blending.
#[allow(dead_code)]
pub struct NormalBlend {
    pub weight: f32,
}

/// Blend two normals by weight (0 = fully a, 1 = fully b), normalized.
#[allow(dead_code)]
pub fn blend_normals(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let nx = a[0] + (b[0] - a[0]) * t;
    let ny = a[1] + (b[1] - a[1]) * t;
    let nz = a[2] + (b[2] - a[2]) * t;
    normalize_normal([nx, ny, nz])
}

/// Perform a weighted blend of two normal lists.
#[allow(dead_code)]
pub fn weighted_normal_blend(
    normals_a: &[[f32; 3]],
    normals_b: &[[f32; 3]],
    weight: f32,
) -> Vec<[f32; 3]> {
    normals_a
        .iter()
        .zip(normals_b.iter())
        .map(|(a, b)| blend_normals(*a, *b, weight))
        .collect()
}

/// Normalize a normal vector to unit length.
#[allow(dead_code)]
pub fn normalize_normal(n: [f32; 3]) -> [f32; 3] {
    let len = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
    if len < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0]/len, n[1]/len, n[2]/len]
    }
}

/// Apply a smooth blend across a list of normal pairs using vertex index.
#[allow(dead_code)]
pub fn smooth_blend_normals(
    normals_a: &[[f32; 3]],
    normals_b: &[[f32; 3]],
    weights: &[f32],
) -> Vec<[f32; 3]> {
    normals_a
        .iter()
        .zip(normals_b.iter())
        .zip(weights.iter())
        .map(|((a, b), w)| blend_normals(*a, *b, *w))
        .collect()
}

/// Blend a normal map (list of normals) with per-texel weight.
#[allow(dead_code)]
pub fn blend_normal_map(map_a: &[[f32; 3]], map_b: &[[f32; 3]], weight: f32) -> Vec<[f32; 3]> {
    weighted_normal_blend(map_a, map_b, weight)
}

/// Linearly interpolate two normals and renormalize.
#[allow(dead_code)]
pub fn normal_blend_lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    blend_normals(a, b, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_unit() {
        let n = normalize_normal([3.0, 0.0, 0.0]);
        assert!((n[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_zero() {
        let n = normalize_normal([0.0, 0.0, 0.0]);
        assert!((n[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_normals_t0() {
        let a = [1.0, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let r = blend_normals(a, b, 0.0);
        assert!((r[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_normals_t1() {
        let a = [1.0, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let r = blend_normals(a, b, 1.0);
        assert!((r[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_weighted_normal_blend_length() {
        let a = vec![[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let b = vec![[0.0f32, 0.0, 1.0], [1.0, 0.0, 0.0]];
        let r = weighted_normal_blend(&a, &b, 0.5);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_smooth_blend_normals() {
        let a = vec![[1.0f32, 0.0, 0.0]];
        let b = vec![[0.0f32, 1.0, 0.0]];
        let w = vec![0.5f32];
        let r = smooth_blend_normals(&a, &b, &w);
        let len = (r[0][0]*r[0][0] + r[0][1]*r[0][1] + r[0][2]*r[0][2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_normal_map_count() {
        let a = vec![[1.0f32, 0.0, 0.0]; 4];
        let b = vec![[0.0f32, 1.0, 0.0]; 4];
        let r = blend_normal_map(&a, &b, 0.3);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_normal_blend_lerp_normalized() {
        let a = [0.0, 0.0, 1.0];
        let b = [0.0, 1.0, 0.0];
        let r = normal_blend_lerp(a, b, 0.5);
        let len = (r[0]*r[0] + r[1]*r[1] + r[2]*r[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_normals_mid_unit() {
        let a = [1.0, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let r = blend_normals(a, b, 0.5);
        let len = (r[0]*r[0]+r[1]*r[1]+r[2]*r[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_normal_blend_struct() {
        let nb = NormalBlend { weight: 0.5 };
        assert!((nb.weight - 0.5).abs() < 1e-6);
    }
}
