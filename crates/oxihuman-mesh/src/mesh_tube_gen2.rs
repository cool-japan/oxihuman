//! Tube/pipe geometry generation (extended).
#![allow(dead_code)]

use std::f32::consts::TAU;

/// Tube generator configuration.
#[allow(dead_code)]
pub struct TubeGen2 {
    pub radius: f32,
    pub segments: usize,
    pub rings: usize,
    pub length: f32,
}

/// Generate tube mesh along the Z axis.
#[allow(dead_code)]
pub fn gen_tube2(radius: f32, length: f32, rings: usize, segments: usize) -> (Vec<[f32;3]>, Vec<u32>) {
    let rings = rings.max(2);
    let segs = segments.max(3);
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for r in 0..=rings {
        let z = (r as f32 / rings as f32) * length - length * 0.5;
        for s in 0..segs {
            let angle = (s as f32 / segs as f32) * TAU;
            positions.push([radius * angle.cos(), radius * angle.sin(), z]);
        }
    }
    for r in 0..rings {
        for s in 0..segs {
            let next_s = (s + 1) % segs;
            let i00 = (r * segs + s) as u32;
            let i10 = (r * segs + next_s) as u32;
            let i01 = ((r+1) * segs + s) as u32;
            let i11 = ((r+1) * segs + next_s) as u32;
            indices.extend_from_slice(&[i00, i10, i01, i10, i11, i01]);
        }
    }
    (positions, indices)
}

/// Generate a tube from a path of 3D points.
#[allow(dead_code)]
pub fn gen_tube2_from_path(path: &[[f32;3]], radius: f32, segments: usize) -> (Vec<[f32;3]>, Vec<u32>) {
    if path.len() < 2 { return (Vec::new(), Vec::new()); }
    let segs = segments.max(3);
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for (ri, &center) in path.iter().enumerate() {
        for s in 0..segs {
            let angle = (s as f32 / segs as f32) * TAU;
            positions.push([center[0] + radius * angle.cos(), center[1] + radius * angle.sin(), center[2]]);
        }
        if ri + 1 < path.len() {
            for s in 0..segs {
                let next_s = (s + 1) % segs;
                let i00 = (ri * segs + s) as u32;
                let i10 = (ri * segs + next_s) as u32;
                let i01 = ((ri+1) * segs + s) as u32;
                let i11 = ((ri+1) * segs + next_s) as u32;
                indices.extend_from_slice(&[i00, i10, i01, i10, i11, i01]);
            }
        }
    }
    (positions, indices)
}

/// Return the vertex count for a tube with given rings and segments.
#[allow(dead_code)]
pub fn tube2_vertex_count(rings: usize, segments: usize) -> usize {
    (rings + 1) * segments
}

/// Add flat cap faces at the ends of the tube.
#[allow(dead_code)]
pub fn tube2_cap_flat(
    positions: &mut Vec<[f32;3]>,
    indices: &mut Vec<u32>,
    rings: usize,
    segments: usize,
) {
    let segs = segments.max(3);
    // Bottom cap
    let bottom_center: u32 = positions.len() as u32;
    let first_bottom = positions[0];
    let cx0 = positions[0..segs].iter().map(|p| p[0]).sum::<f32>() / segs as f32;
    let cy0 = positions[0..segs].iter().map(|p| p[1]).sum::<f32>() / segs as f32;
    let cz0 = first_bottom[2];
    positions.push([cx0, cy0, cz0]);
    for s in 0..segs {
        let next_s = (s + 1) % segs;
        indices.extend_from_slice(&[bottom_center, s as u32, next_s as u32]);
    }
    // Top cap
    let top_start = rings * segs;
    let top_center: u32 = positions.len() as u32;
    let cx1 = positions[top_start..top_start+segs].iter().map(|p| p[0]).sum::<f32>() / segs as f32;
    let cy1 = positions[top_start..top_start+segs].iter().map(|p| p[1]).sum::<f32>() / segs as f32;
    let cz1 = positions[top_start][2];
    positions.push([cx1, cy1, cz1]);
    for s in 0..segs {
        let next_s = (s + 1) % segs;
        indices.extend_from_slice(&[top_center, (top_start + next_s) as u32, (top_start + s) as u32]);
    }
}

/// Generate tube with variable radius per ring.
#[allow(dead_code)]
pub fn tube2_radius_vary(radii: &[f32], length: f32, segments: usize) -> (Vec<[f32;3]>, Vec<u32>) {
    if radii.is_empty() { return (Vec::new(), Vec::new()); }
    let segs = segments.max(3);
    let rings = radii.len() - 1;
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for (ri, &r) in radii.iter().enumerate() {
        let z = if rings == 0 { 0.0 } else { (ri as f32 / rings as f32) * length - length * 0.5 };
        for s in 0..segs {
            let angle = (s as f32 / segs as f32) * TAU;
            positions.push([r * angle.cos(), r * angle.sin(), z]);
        }
    }
    for r in 0..rings {
        for s in 0..segs {
            let next_s = (s + 1) % segs;
            let i00 = (r * segs + s) as u32;
            let i10 = (r * segs + next_s) as u32;
            let i01 = ((r+1) * segs + s) as u32;
            let i11 = ((r+1) * segs + next_s) as u32;
            indices.extend_from_slice(&[i00, i10, i01, i10, i11, i01]);
        }
    }
    (positions, indices)
}

/// Generate UV coordinates for a tube (u = angle/TAU, v = ring/rings).
#[allow(dead_code)]
pub fn tube2_uvs(rings: usize, segments: usize) -> Vec<[f32;2]> {
    let mut uvs = Vec::new();
    for r in 0..=rings {
        let v = r as f32 / rings as f32;
        for s in 0..segments {
            let u = s as f32 / segments as f32;
            uvs.push([u, v]);
        }
    }
    uvs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_tube2_vertex_count() {
        let (pos, _) = gen_tube2(1.0, 2.0, 4, 8);
        assert_eq!(pos.len(), tube2_vertex_count(4, 8));
    }

    #[test]
    fn test_gen_tube2_index_count() {
        let (_, idx) = gen_tube2(1.0, 2.0, 2, 6);
        assert_eq!(idx.len(), 2 * 2 * 6 * 3);
    }

    #[test]
    fn test_gen_tube2_from_path_empty() {
        let (pos, idx) = gen_tube2_from_path(&[[0.0,0.0,0.0]], 0.5, 6);
        assert!(pos.is_empty() && idx.is_empty());
    }

    #[test]
    fn test_gen_tube2_from_path_two_rings() {
        let path = vec![[0.0f32,0.0,0.0],[0.0,0.0,1.0]];
        let (pos, _) = gen_tube2_from_path(&path, 0.5, 6);
        assert_eq!(pos.len(), 12);
    }

    #[test]
    fn test_tube2_vertex_count() {
        assert_eq!(tube2_vertex_count(3, 8), 32);
    }

    #[test]
    fn test_tube2_uvs_count() {
        let uvs = tube2_uvs(4, 8);
        assert_eq!(uvs.len(), 5 * 8);
    }

    #[test]
    fn test_tube2_radius_vary() {
        let radii = vec![0.5f32, 1.0, 0.5];
        let (pos, idx) = tube2_radius_vary(&radii, 2.0, 6);
        assert_eq!(pos.len(), 3 * 6);
        assert!(!idx.is_empty());
    }

    #[test]
    fn test_gen_tube2_config() {
        let cfg = TubeGen2 { radius: 1.0, segments: 8, rings: 4, length: 2.0 };
        assert!((cfg.radius - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_tube2_uvs_range() {
        let uvs = tube2_uvs(2, 4);
        for uv in &uvs {
            assert!((0.0..=1.0).contains(&uv[0]));
            assert!((0.0..=1.0).contains(&uv[1]));
        }
    }
}
