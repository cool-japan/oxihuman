#![allow(dead_code)]
//! Vertex scattering on mesh surfaces.

/// Vertex scatter result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VertexScatter {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub seed: u64,
}

/// Scatter points on a surface defined by triangles (deterministic).
#[allow(dead_code)]
pub fn scatter_on_surface(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    count: usize,
    seed: u64,
) -> VertexScatter {
    let mut pts = Vec::with_capacity(count);
    let mut nrms = Vec::with_capacity(count);
    if tris.is_empty() || positions.is_empty() {
        return VertexScatter { positions: pts, normals: nrms, seed };
    }
    // Deterministic scatter using seed as offset
    for i in 0..count {
        let face_idx = (seed as usize + i) % tris.len();
        let tri = tris[face_idx];
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        // Use deterministic barycentric coords based on i
        let t = (i as f32 + 1.0) / (count as f32 + 1.0);
        let u = t * 0.5;
        let v = (1.0 - t) * 0.5;
        let w = 1.0 - u - v;
        pts.push([
            a[0] * u + b[0] * v + c[0] * w,
            a[1] * u + b[1] * v + c[1] * w,
            a[2] * u + b[2] * v + c[2] * w,
        ]);
        // Compute face normal
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-12 {
            nrms.push([n[0] / len, n[1] / len, n[2] / len]);
        } else {
            nrms.push([0.0, 1.0, 0.0]);
        }
    }
    VertexScatter { positions: pts, normals: nrms, seed }
}

/// Get scattered point count.
#[allow(dead_code)]
pub fn scatter_count(vs: &VertexScatter) -> usize {
    vs.positions.len()
}

/// Get seed.
#[allow(dead_code)]
pub fn scatter_seed(vs: &VertexScatter) -> u64 {
    vs.seed
}

/// Compute density (points per unit area).
#[allow(dead_code)]
pub fn scatter_density(vs: &VertexScatter, total_area: f32) -> f32 {
    if total_area <= 0.0 {
        return 0.0;
    }
    vs.positions.len() as f32 / total_area
}

/// Get scattered positions.
#[allow(dead_code)]
pub fn scatter_positions(vs: &VertexScatter) -> &[[f32; 3]] {
    &vs.positions
}

/// Get scattered normals.
#[allow(dead_code)]
pub fn scatter_normals(vs: &VertexScatter) -> &[[f32; 3]] {
    &vs.normals
}

/// Serialize scatter to JSON.
#[allow(dead_code)]
pub fn scatter_to_json(vs: &VertexScatter) -> String {
    format!(
        "{{\"count\":{},\"seed\":{}}}",
        vs.positions.len(),
        vs.seed
    )
}

/// Scatter points within a bounding box region.
#[allow(dead_code)]
pub fn scatter_in_region(min: [f32; 3], max: [f32; 3], count: usize) -> Vec<[f32; 3]> {
    let mut pts = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / count.max(1) as f32;
        pts.push([
            min[0] + (max[0] - min[0]) * t,
            min[1] + (max[1] - min[1]) * t,
            min[2] + (max[2] - min[2]) * t,
        ]);
    }
    pts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scatter_on_surface() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0, 1, 2]];
        let vs = scatter_on_surface(&pos, &tris, 5, 42);
        assert_eq!(vs.positions.len(), 5);
    }

    #[test]
    fn test_scatter_empty() {
        let vs = scatter_on_surface(&[], &[], 5, 0);
        assert!(vs.positions.is_empty());
    }

    #[test]
    fn test_scatter_count() {
        let vs = VertexScatter { positions: vec![[0.0; 3]; 3], normals: vec![[0.0; 3]; 3], seed: 0 };
        assert_eq!(scatter_count(&vs), 3);
    }

    #[test]
    fn test_scatter_seed() {
        let vs = VertexScatter { positions: vec![], normals: vec![], seed: 42 };
        assert_eq!(scatter_seed(&vs), 42);
    }

    #[test]
    fn test_scatter_density() {
        let vs = VertexScatter { positions: vec![[0.0; 3]; 10], normals: vec![], seed: 0 };
        assert!((scatter_density(&vs, 5.0) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_scatter_density_zero_area() {
        let vs = VertexScatter { positions: vec![], normals: vec![], seed: 0 };
        assert!((scatter_density(&vs, 0.0)).abs() < 1e-6);
    }

    #[test]
    fn test_scatter_positions() {
        let vs = VertexScatter { positions: vec![[1.0, 2.0, 3.0]], normals: vec![], seed: 0 };
        assert_eq!(scatter_positions(&vs).len(), 1);
    }

    #[test]
    fn test_scatter_normals() {
        let vs = VertexScatter { positions: vec![], normals: vec![[0.0, 1.0, 0.0]], seed: 0 };
        assert_eq!(scatter_normals(&vs).len(), 1);
    }

    #[test]
    fn test_scatter_to_json() {
        let vs = VertexScatter { positions: vec![], normals: vec![], seed: 7 };
        let j = scatter_to_json(&vs);
        assert!(j.contains("\"seed\":7"));
    }

    #[test]
    fn test_scatter_in_region() {
        let pts = scatter_in_region([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 3);
        assert_eq!(pts.len(), 3);
    }
}
