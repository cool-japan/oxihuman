#![allow(dead_code)]

//! Vertex blending between meshes.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexBlend {
    pub positions: Vec<[f32; 3]>,
    pub factor: f32,
}

#[allow(dead_code)]
pub fn blend_positions(a: &[[f32; 3]], b: &[[f32; 3]], t: f32) -> Vec<[f32; 3]> {
    let n = a.len().min(b.len());
    (0..n).map(|i| {
        [a[i][0]*(1.0-t)+b[i][0]*t, a[i][1]*(1.0-t)+b[i][1]*t, a[i][2]*(1.0-t)+b[i][2]*t]
    }).collect()
}

#[allow(dead_code)]
pub fn blend_normals_vb(a: &[[f32; 3]], b: &[[f32; 3]], t: f32) -> Vec<[f32; 3]> {
    let n = a.len().min(b.len());
    (0..n).map(|i| {
        let r = [a[i][0]*(1.0-t)+b[i][0]*t, a[i][1]*(1.0-t)+b[i][1]*t, a[i][2]*(1.0-t)+b[i][2]*t];
        let len = (r[0]*r[0]+r[1]*r[1]+r[2]*r[2]).sqrt().max(1e-12);
        [r[0]/len, r[1]/len, r[2]/len]
    }).collect()
}

#[allow(dead_code)]
pub fn blend_uvs(a: &[[f32; 2]], b: &[[f32; 2]], t: f32) -> Vec<[f32; 2]> {
    let n = a.len().min(b.len());
    (0..n).map(|i| [a[i][0]*(1.0-t)+b[i][0]*t, a[i][1]*(1.0-t)+b[i][1]*t]).collect()
}

#[allow(dead_code)]
pub fn blend_colors_vb(a: &[[f32; 4]], b: &[[f32; 4]], t: f32) -> Vec<[f32; 4]> {
    let n = a.len().min(b.len());
    (0..n).map(|i| {
        [a[i][0]*(1.0-t)+b[i][0]*t, a[i][1]*(1.0-t)+b[i][1]*t,
         a[i][2]*(1.0-t)+b[i][2]*t, a[i][3]*(1.0-t)+b[i][3]*t]
    }).collect()
}

#[allow(dead_code)]
pub fn blend_weight_vb(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[allow(dead_code)]
pub fn blend_meshes(positions: &[&[[f32; 3]]], weights: &[f32]) -> Vec<[f32; 3]> {
    if positions.is_empty() { return Vec::new(); }
    let n = positions.iter().map(|p| p.len()).min().unwrap_or(0);
    let mut result = vec![[0.0f32; 3]; n];
    for (mesh_idx, &mesh_pos) in positions.iter().enumerate() {
        let w = weights.get(mesh_idx).copied().unwrap_or(0.0);
        for i in 0..n {
            result[i][0] += mesh_pos[i][0] * w;
            result[i][1] += mesh_pos[i][1] * w;
            result[i][2] += mesh_pos[i][2] * w;
        }
    }
    result
}

#[allow(dead_code)]
pub fn blend_count(a: &[[f32; 3]], b: &[[f32; 3]]) -> usize {
    a.len().min(b.len())
}

#[allow(dead_code)]
pub fn blend_to_json(blend: &VertexBlend) -> String {
    format!("{{\"vertex_count\":{},\"factor\":{}}}", blend.positions.len(), blend.factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_positions_zero() {
        let a = vec![[0.0,0.0,0.0]]; let b = vec![[1.0,1.0,1.0]];
        let r = blend_positions(&a, &b, 0.0);
        assert!((r[0][0]).abs() < 1e-6);
    }

    #[test]
    fn test_blend_positions_one() {
        let a = vec![[0.0,0.0,0.0]]; let b = vec![[1.0,1.0,1.0]];
        let r = blend_positions(&a, &b, 1.0);
        assert!((r[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_positions_half() {
        let a = vec![[0.0,0.0,0.0]]; let b = vec![[2.0,2.0,2.0]];
        let r = blend_positions(&a, &b, 0.5);
        assert!((r[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_normals() {
        let a = vec![[1.0,0.0,0.0]]; let b = vec![[0.0,1.0,0.0]];
        let r = blend_normals_vb(&a, &b, 0.5);
        let len = (r[0][0]*r[0][0]+r[0][1]*r[0][1]+r[0][2]*r[0][2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_uvs() {
        let a = vec![[0.0,0.0]]; let b = vec![[1.0,1.0]];
        let r = blend_uvs(&a, &b, 0.5);
        assert!((r[0][0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_colors() {
        let a = vec![[1.0,0.0,0.0,1.0]]; let b = vec![[0.0,1.0,0.0,1.0]];
        let r = blend_colors_vb(&a, &b, 0.5);
        assert!((r[0][0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_weight() {
        assert!((blend_weight_vb(0.0, 1.0, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_meshes() {
        let a = vec![[1.0,0.0,0.0]]; let b = vec![[0.0,1.0,0.0]];
        let r = blend_meshes(&[&a, &b], &[0.5, 0.5]);
        assert!((r[0][0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_count() {
        assert_eq!(blend_count(&[[0.0;3];3], &[[0.0;3];2]), 2);
    }

    #[test]
    fn test_blend_to_json() {
        let b = VertexBlend { positions: vec![[0.0;3]], factor: 0.5 };
        let j = blend_to_json(&b);
        assert!(j.contains("\"vertex_count\":1"));
    }
}
