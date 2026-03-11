// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashSet;

use crate::mesh::MeshBuffers;

/// Comprehensive mesh statistics.
#[derive(Debug, Clone)]
pub struct MeshStats {
    pub vertex_count: usize,
    pub face_count: usize,
    pub edge_count: usize,
    pub surface_area: f32,
    pub volume_estimate: f32,
    pub bbox_diagonal: f32,
    pub avg_edge_length: f32,
    pub min_edge_length: f32,
    pub max_edge_length: f32,
    pub avg_face_area: f32,
    pub min_face_area: f32,
    pub max_face_area: f32,
    pub avg_aspect_ratio: f32,
    pub euler_characteristic: i32,
}

impl MeshStats {
    /// Returns a human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "Vertices: {}, Faces: {}, Edges: {}, Euler: {}\n\
             Surface area: {:.6}, Volume est: {:.6}, BBox diag: {:.6}\n\
             Edge length  avg={:.6} min={:.6} max={:.6}\n\
             Face area    avg={:.6} min={:.6} max={:.6}\n\
             Avg aspect ratio: {:.6}",
            self.vertex_count,
            self.face_count,
            self.edge_count,
            self.euler_characteristic,
            self.surface_area,
            self.volume_estimate,
            self.bbox_diagonal,
            self.avg_edge_length,
            self.min_edge_length,
            self.max_edge_length,
            self.avg_face_area,
            self.min_face_area,
            self.max_face_area,
            self.avg_aspect_ratio,
        )
    }
}

// ---------------------------------------------------------------------------
// Helper: cross product of two 3-vectors.
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

// ---------------------------------------------------------------------------

/// Compute the surface area of a mesh (sum of triangle areas).
pub fn surface_area(mesh: &MeshBuffers) -> f32 {
    let mut total = 0.0f32;
    for tri in mesh.indices.chunks_exact(3) {
        let p0 = mesh.positions[tri[0] as usize];
        let p1 = mesh.positions[tri[1] as usize];
        let p2 = mesh.positions[tri[2] as usize];
        let e1 = sub3(p1, p0);
        let e2 = sub3(p2, p0);
        total += length3(cross(e1, e2)) * 0.5;
    }
    total
}

/// Estimate volume using the divergence theorem (signed, works for closed meshes).
/// For open meshes the result is not physically meaningful but still computed.
pub fn volume_estimate(mesh: &MeshBuffers) -> f32 {
    let mut vol = 0.0f32;
    for tri in mesh.indices.chunks_exact(3) {
        let p0 = mesh.positions[tri[0] as usize];
        let p1 = mesh.positions[tri[1] as usize];
        let p2 = mesh.positions[tri[2] as usize];
        vol += (p0[0] * (p1[1] * p2[2] - p1[2] * p2[1])
            + p0[1] * (p1[2] * p2[0] - p1[0] * p2[2])
            + p0[2] * (p1[0] * p2[1] - p1[1] * p2[0]))
            / 6.0;
    }
    vol.abs()
}

/// Compute per-face areas. Returns a Vec of length face_count.
pub fn face_areas(mesh: &MeshBuffers) -> Vec<f32> {
    mesh.indices
        .chunks_exact(3)
        .map(|tri| {
            let p0 = mesh.positions[tri[0] as usize];
            let p1 = mesh.positions[tri[1] as usize];
            let p2 = mesh.positions[tri[2] as usize];
            let e1 = sub3(p1, p0);
            let e2 = sub3(p2, p0);
            length3(cross(e1, e2)) * 0.5
        })
        .collect()
}

/// Compute per-edge lengths for all unique edges.
pub fn edge_lengths(mesh: &MeshBuffers) -> Vec<f32> {
    let mut seen: HashSet<(u32, u32)> = HashSet::new();
    let mut lengths = Vec::new();

    for tri in mesh.indices.chunks_exact(3) {
        let verts = [tri[0], tri[1], tri[2]];
        for i in 0..3 {
            let a = verts[i];
            let b = verts[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                let pa = mesh.positions[a as usize];
                let pb = mesh.positions[b as usize];
                lengths.push(length3(sub3(pb, pa)));
            }
        }
    }
    lengths
}

/// Compute per-face aspect ratios (longest edge / shortest edge).
pub fn face_aspect_ratios(mesh: &MeshBuffers) -> Vec<f32> {
    mesh.indices
        .chunks_exact(3)
        .map(|tri| {
            let p0 = mesh.positions[tri[0] as usize];
            let p1 = mesh.positions[tri[1] as usize];
            let p2 = mesh.positions[tri[2] as usize];
            let e0 = length3(sub3(p1, p0));
            let e1 = length3(sub3(p2, p1));
            let e2 = length3(sub3(p0, p2));
            let max_e = e0.max(e1).max(e2);
            let min_e = e0.min(e1).min(e2);
            if min_e > 0.0 {
                max_e / min_e
            } else {
                f32::INFINITY
            }
        })
        .collect()
}

/// Compute approximate per-vertex mean curvature using the umbrella operator.
/// For each vertex v, curvature ≈ || v - avg(neighbors) || / avg_edge_length.
/// Returns a Vec of length vertex_count.
pub fn vertex_mean_curvature(mesh: &MeshBuffers) -> Vec<f32> {
    let n = mesh.positions.len();
    let mut neighbor_sum: Vec<[f32; 3]> = vec![[0.0; 3]; n];
    let mut neighbor_count: Vec<u32> = vec![0u32; n];

    for tri in mesh.indices.chunks_exact(3) {
        let verts = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        for i in 0..3 {
            let vi = verts[i];
            let vj = verts[(i + 1) % 3];
            let vk = verts[(i + 2) % 3];
            // vi neighbors: vj and vk
            let pj = mesh.positions[vj];
            let pk = mesh.positions[vk];
            neighbor_sum[vi][0] += pj[0] + pk[0];
            neighbor_sum[vi][1] += pj[1] + pk[1];
            neighbor_sum[vi][2] += pj[2] + pk[2];
            neighbor_count[vi] += 2;
        }
    }

    let el = edge_lengths(mesh);
    let avg_edge = if el.is_empty() {
        1.0
    } else {
        el.iter().sum::<f32>() / el.len() as f32
    };
    let denom = if avg_edge > 0.0 { avg_edge } else { 1.0 };

    (0..n)
        .map(|vi| {
            let cnt = neighbor_count[vi] as f32;
            if cnt == 0.0 {
                return 0.0;
            }
            let avg_nb = [
                neighbor_sum[vi][0] / cnt,
                neighbor_sum[vi][1] / cnt,
                neighbor_sum[vi][2] / cnt,
            ];
            let p = mesh.positions[vi];
            let diff = sub3(p, avg_nb);
            length3(diff) / denom
        })
        .collect()
}

// ---------------------------------------------------------------------------

/// Compute all mesh statistics.
pub fn compute_stats(mesh: &MeshBuffers) -> MeshStats {
    let vertex_count = mesh.positions.len();
    let face_count = mesh.indices.len() / 3;

    // Edge count (unique)
    let el = edge_lengths(mesh);
    let edge_count = el.len();

    let sa = surface_area(mesh);
    let vol = volume_estimate(mesh);

    // AABB diagonal
    let bbox_diagonal = {
        let min = mesh.positions.iter().fold([f32::MAX; 3], |acc, p| {
            [acc[0].min(p[0]), acc[1].min(p[1]), acc[2].min(p[2])]
        });
        let max = mesh.positions.iter().fold([f32::MIN; 3], |acc, p| {
            [acc[0].max(p[0]), acc[1].max(p[1]), acc[2].max(p[2])]
        });
        let d = [
            (max[0] - min[0]).powi(2),
            (max[1] - min[1]).powi(2),
            (max[2] - min[2]).powi(2),
        ];
        (d[0] + d[1] + d[2]).sqrt()
    };

    // Edge length stats
    let (avg_edge_length, min_edge_length, max_edge_length) = if el.is_empty() {
        (0.0, 0.0, 0.0)
    } else {
        let sum: f32 = el.iter().sum();
        let min = el.iter().cloned().fold(f32::MAX, f32::min);
        let max = el.iter().cloned().fold(f32::MIN, f32::max);
        (sum / el.len() as f32, min, max)
    };

    // Face area stats
    let fa = face_areas(mesh);
    let (avg_face_area, min_face_area, max_face_area) = if fa.is_empty() {
        (0.0, 0.0, 0.0)
    } else {
        let sum: f32 = fa.iter().sum();
        let min = fa.iter().cloned().fold(f32::MAX, f32::min);
        let max = fa.iter().cloned().fold(f32::MIN, f32::max);
        (sum / fa.len() as f32, min, max)
    };

    // Aspect ratio
    let ar = face_aspect_ratios(mesh);
    let avg_aspect_ratio = if ar.is_empty() {
        0.0
    } else {
        ar.iter().sum::<f32>() / ar.len() as f32
    };

    // Euler characteristic: V - E + F
    let euler_characteristic = vertex_count as i32 - edge_count as i32 + face_count as i32;

    MeshStats {
        vertex_count,
        face_count,
        edge_count,
        surface_area: sa,
        volume_estimate: vol,
        bbox_diagonal,
        avg_edge_length,
        min_edge_length,
        max_edge_length,
        avg_face_area,
        min_face_area,
        max_face_area,
        avg_aspect_ratio,
        euler_characteristic,
    }
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_triangle() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            tangents: vec![],
            colors: None,
            indices: vec![0, 1, 2],
            has_suit: true,
        }
    }

    fn two_triangle_mesh() -> MeshBuffers {
        // Two right-angle unit triangles forming a unit square in XY plane.
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            tangents: vec![],
            colors: None,
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: true,
        }
    }

    #[test]
    fn unit_triangle_area() {
        let mesh = unit_triangle();
        let area = surface_area(&mesh);
        assert!((area - 0.5).abs() < 1e-5, "area={area}");
    }

    #[test]
    fn unit_triangle_face_count() {
        let mesh = unit_triangle();
        assert_eq!(compute_stats(&mesh).face_count, 1);
    }

    #[test]
    fn face_areas_length() {
        let mesh = unit_triangle();
        let fa = face_areas(&mesh);
        assert_eq!(fa.len(), mesh.face_count());
    }

    #[test]
    fn edge_lengths_unit_triangle() {
        let mesh = unit_triangle();
        let el = edge_lengths(&mesh);
        assert_eq!(el.len(), 3, "unit triangle should have 3 unique edges");
    }

    #[test]
    fn stats_vertex_count_correct() {
        let mesh = unit_triangle();
        assert_eq!(compute_stats(&mesh).vertex_count, 3);
    }

    #[test]
    fn stats_euler_characteristic() {
        let mesh = unit_triangle();
        let stats = compute_stats(&mesh);
        // V=3, E=3, F=1 => χ = 1
        assert_eq!(stats.euler_characteristic, 1);
    }

    #[test]
    fn face_aspect_ratios_length() {
        let mesh = unit_triangle();
        let ar = face_aspect_ratios(&mesh);
        assert_eq!(ar.len(), 1);
    }

    #[test]
    fn surface_area_two_triangles() {
        let mesh = two_triangle_mesh();
        let area = surface_area(&mesh);
        assert!((area - 1.0).abs() < 1e-5, "area={area}");
    }

    #[test]
    fn stats_summary_nonempty() {
        let mesh = unit_triangle();
        let s = compute_stats(&mesh).summary();
        assert!(!s.is_empty(), "summary should be non-empty");
    }
}
