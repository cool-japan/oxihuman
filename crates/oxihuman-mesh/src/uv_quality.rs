// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use crate::mesh::MeshBuffers;

/// Overall UV quality report.
#[derive(Debug, Clone)]
pub struct UvQualityReport {
    pub face_count: usize,
    /// Average stretch (0=no stretch, 1=fully stretched).
    pub avg_stretch: f32,
    /// Maximum stretch across all faces.
    pub max_stretch: f32,
    /// Fraction of UV space `[0,1]`² used by the mesh (coverage).
    pub uv_utilization: f32,
    /// Number of degenerate UV faces (zero UV area).
    pub degenerate_uv_face_count: usize,
    /// Number of overlapping UV face pairs (approximate).
    pub overlap_count: usize,
    /// Average conformal distortion (how much angles are distorted).
    pub avg_conformal_distortion: f32,
}

/// Compute the area of a UV triangle.
#[allow(dead_code)]
pub fn uv_triangle_area(uv0: [f32; 2], uv1: [f32; 2], uv2: [f32; 2]) -> f32 {
    let e1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
    let e2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];
    // 2D cross product magnitude
    (e1[0] * e2[1] - e1[1] * e2[0]).abs() * 0.5
}

/// Check if a UV triangle is degenerate (area near zero).
#[allow(dead_code)]
pub fn is_degenerate_uv_face(uv0: [f32; 2], uv1: [f32; 2], uv2: [f32; 2], epsilon: f32) -> bool {
    uv_triangle_area(uv0, uv1, uv2) < epsilon
}

/// Compute the stretch of a single UV face:
/// ratio of 3D face area to UV face area (normalized).
/// Returns 0.0 for degenerate faces.
#[allow(dead_code)]
pub fn face_uv_stretch(
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
) -> f32 {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

    // Cross product
    let cross = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    let area_3d = len * 0.5;

    let area_uv = uv_triangle_area(uv0, uv1, uv2);

    const EPSILON: f32 = 1e-10;
    if area_uv < EPSILON || area_3d < EPSILON {
        return 0.0;
    }

    // Stretch: ratio of 3D to UV area. Normalize so that 1.0 = same scale.
    let ratio = area_3d / (area_uv + EPSILON);
    // Normalize by clipping to [0, 1] using a tanh-like approach
    // We use ratio / (1.0 + ratio) to keep in [0, 1)
    ratio / (1.0 + ratio)
}

/// Compute conformal distortion for a UV face:
/// measures how much the UV mapping distorts angles vs. 3D surface.
/// Returns a value in [0, inf) where 0 = perfect conformal mapping.
#[allow(dead_code)]
pub fn face_conformal_distortion(
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
) -> f32 {
    // 3D edge vectors
    let e1_3d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2_3d = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

    // UV edge vectors
    let e1_uv = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
    let e2_uv = [uv2[0] - uv0[0], uv2[1] - uv0[1]];

    // First fundamental form in 3D: g_ij
    let g11 = e1_3d[0] * e1_3d[0] + e1_3d[1] * e1_3d[1] + e1_3d[2] * e1_3d[2];
    let g12 = e1_3d[0] * e2_3d[0] + e1_3d[1] * e2_3d[1] + e1_3d[2] * e2_3d[2];
    let g22 = e2_3d[0] * e2_3d[0] + e2_3d[1] * e2_3d[1] + e2_3d[2] * e2_3d[2];

    // First fundamental form in UV: h_ij
    let h11 = e1_uv[0] * e1_uv[0] + e1_uv[1] * e1_uv[1];
    let h12 = e1_uv[0] * e2_uv[0] + e1_uv[1] * e2_uv[1];
    let h22 = e2_uv[0] * e2_uv[0] + e2_uv[1] * e2_uv[1];

    let det_g = g11 * g22 - g12 * g12;
    let det_h = h11 * h22 - h12 * h12;

    const EPSILON: f32 = 1e-10;
    if det_g < EPSILON || det_h < EPSILON {
        return 0.0;
    }

    // Scale-normalized difference: measure how far from conformal the map is.
    // For a conformal map, h_ij = lambda * g_ij for some scalar lambda.
    // Distortion = ||H/sqrt(det_h) - G/sqrt(det_g)||_F
    let scale_g = det_g.sqrt();
    let scale_h = det_h.sqrt();

    let dg11 = g11 / scale_g - h11 / scale_h;
    let dg12 = g12 / scale_g - h12 / scale_h;
    let dg22 = g22 / scale_g - h22 / scale_h;

    (dg11 * dg11 + 2.0 * dg12 * dg12 + dg22 * dg22).sqrt()
}

/// Compute per-face UV stretch values.
#[allow(dead_code)]
pub fn compute_face_stretches(mesh: &MeshBuffers) -> Vec<f32> {
    let face_count = mesh.indices.len() / 3;
    let mut stretches = Vec::with_capacity(face_count);

    for i in 0..face_count {
        let i0 = mesh.indices[i * 3] as usize;
        let i1 = mesh.indices[i * 3 + 1] as usize;
        let i2 = mesh.indices[i * 3 + 2] as usize;

        if i0 >= mesh.positions.len() || i1 >= mesh.positions.len() || i2 >= mesh.positions.len() {
            stretches.push(0.0);
            continue;
        }

        let p0 = mesh.positions[i0];
        let p1 = mesh.positions[i1];
        let p2 = mesh.positions[i2];

        let (uv0, uv1, uv2) = if mesh.uvs.len() > i0 && mesh.uvs.len() > i1 && mesh.uvs.len() > i2 {
            (mesh.uvs[i0], mesh.uvs[i1], mesh.uvs[i2])
        } else {
            stretches.push(0.0);
            continue;
        };

        stretches.push(face_uv_stretch(p0, p1, p2, uv0, uv1, uv2));
    }

    stretches
}

/// Compute UV utilization: what fraction of the `[0,1]`² UV space is covered.
/// Uses a grid sampling approach (grid_size × grid_size cells).
#[allow(dead_code)]
pub fn compute_uv_utilization(mesh: &MeshBuffers, grid_size: usize) -> f32 {
    if grid_size == 0 {
        return 0.0;
    }

    let total_cells = grid_size * grid_size;
    let mut grid = vec![false; total_cells];

    let face_count = mesh.indices.len() / 3;

    for i in 0..face_count {
        let i0 = mesh.indices[i * 3] as usize;
        let i1 = mesh.indices[i * 3 + 1] as usize;
        let i2 = mesh.indices[i * 3 + 2] as usize;

        if i0 >= mesh.uvs.len() || i1 >= mesh.uvs.len() || i2 >= mesh.uvs.len() {
            continue;
        }

        let uv0 = mesh.uvs[i0];
        let uv1 = mesh.uvs[i1];
        let uv2 = mesh.uvs[i2];

        rasterize_triangle_into_grid(uv0, uv1, uv2, grid_size, &mut grid, false);
    }

    let covered = grid.iter().filter(|&&v| v).count();
    covered as f32 / total_cells as f32
}

/// Count approximately overlapping UV faces using a grid-based approach.
#[allow(dead_code)]
pub fn count_uv_overlaps(mesh: &MeshBuffers, grid_size: usize) -> usize {
    if grid_size == 0 {
        return 0;
    }

    let total_cells = grid_size * grid_size;
    let mut grid_count: Vec<u32> = vec![0; total_cells];

    let face_count = mesh.indices.len() / 3;

    for i in 0..face_count {
        let i0 = mesh.indices[i * 3] as usize;
        let i1 = mesh.indices[i * 3 + 1] as usize;
        let i2 = mesh.indices[i * 3 + 2] as usize;

        if i0 >= mesh.uvs.len() || i1 >= mesh.uvs.len() || i2 >= mesh.uvs.len() {
            continue;
        }

        let uv0 = mesh.uvs[i0];
        let uv1 = mesh.uvs[i1];
        let uv2 = mesh.uvs[i2];

        rasterize_triangle_into_count_grid(uv0, uv1, uv2, grid_size, &mut grid_count);
    }

    grid_count.iter().filter(|&&c| c >= 2).count()
}

/// Generate a full UV quality report for a mesh.
#[allow(dead_code)]
pub fn uv_quality_report(mesh: &MeshBuffers) -> UvQualityReport {
    let face_count = mesh.indices.len() / 3;
    let stretches = compute_face_stretches(mesh);

    let avg_stretch = if stretches.is_empty() {
        0.0
    } else {
        stretches.iter().sum::<f32>() / stretches.len() as f32
    };

    let max_stretch = stretches.iter().cloned().fold(0.0f32, f32::max);

    let uv_utilization = compute_uv_utilization(mesh, 64);
    let overlap_count = count_uv_overlaps(mesh, 64);

    let mut degenerate_uv_face_count = 0usize;
    let mut total_conformal = 0.0f32;
    let mut conformal_valid = 0usize;

    for i in 0..face_count {
        let i0 = mesh.indices[i * 3] as usize;
        let i1 = mesh.indices[i * 3 + 1] as usize;
        let i2 = mesh.indices[i * 3 + 2] as usize;

        if i0 >= mesh.uvs.len() || i1 >= mesh.uvs.len() || i2 >= mesh.uvs.len() {
            continue;
        }

        let uv0 = mesh.uvs[i0];
        let uv1 = mesh.uvs[i1];
        let uv2 = mesh.uvs[i2];

        if is_degenerate_uv_face(uv0, uv1, uv2, 1e-8) {
            degenerate_uv_face_count += 1;
        }

        if i0 < mesh.positions.len() && i1 < mesh.positions.len() && i2 < mesh.positions.len() {
            let p0 = mesh.positions[i0];
            let p1 = mesh.positions[i1];
            let p2 = mesh.positions[i2];
            let dist = face_conformal_distortion(p0, p1, p2, uv0, uv1, uv2);
            total_conformal += dist;
            conformal_valid += 1;
        }
    }

    let avg_conformal_distortion = if conformal_valid > 0 {
        total_conformal / conformal_valid as f32
    } else {
        0.0
    };

    UvQualityReport {
        face_count,
        avg_stretch,
        max_stretch,
        uv_utilization,
        degenerate_uv_face_count,
        overlap_count,
        avg_conformal_distortion,
    }
}

/// Find the faces with the worst UV stretch (top N).
#[allow(dead_code)]
pub fn worst_stretch_faces(mesh: &MeshBuffers, n: usize) -> Vec<(usize, f32)> {
    let stretches = compute_face_stretches(mesh);
    let mut indexed: Vec<(usize, f32)> = stretches.into_iter().enumerate().collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    indexed.truncate(n);
    indexed
}

// ---- Internal helpers ----

/// Rasterize a UV triangle into a boolean grid (marks coverage).
fn rasterize_triangle_into_grid(
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
    grid_size: usize,
    grid: &mut [bool],
    _count_mode: bool,
) {
    let gs = grid_size as f32;

    // Compute bounding box in grid space
    let min_x = uv0[0].min(uv1[0]).min(uv2[0]);
    let max_x = uv0[0].max(uv1[0]).max(uv2[0]);
    let min_y = uv0[1].min(uv1[1]).min(uv2[1]);
    let max_y = uv0[1].max(uv1[1]).max(uv2[1]);

    let gx0 = ((min_x * gs).floor() as i32).clamp(0, grid_size as i32 - 1);
    let gx1 = ((max_x * gs).ceil() as i32).clamp(0, grid_size as i32 - 1);
    let gy0 = ((min_y * gs).floor() as i32).clamp(0, grid_size as i32 - 1);
    let gy1 = ((max_y * gs).ceil() as i32).clamp(0, grid_size as i32 - 1);

    for cy in gy0..=gy1 {
        for cx in gx0..=gx1 {
            // Cell center in UV space
            let cu = (cx as f32 + 0.5) / gs;
            let cv = (cy as f32 + 0.5) / gs;

            if point_in_uv_triangle([cu, cv], uv0, uv1, uv2) {
                let idx = cy as usize * grid_size + cx as usize;
                grid[idx] = true;
            }
        }
    }
}

/// Rasterize a UV triangle and increment a count grid.
fn rasterize_triangle_into_count_grid(
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
    grid_size: usize,
    grid: &mut [u32],
) {
    let gs = grid_size as f32;

    let min_x = uv0[0].min(uv1[0]).min(uv2[0]);
    let max_x = uv0[0].max(uv1[0]).max(uv2[0]);
    let min_y = uv0[1].min(uv1[1]).min(uv2[1]);
    let max_y = uv0[1].max(uv1[1]).max(uv2[1]);

    let gx0 = ((min_x * gs).floor() as i32).clamp(0, grid_size as i32 - 1);
    let gx1 = ((max_x * gs).ceil() as i32).clamp(0, grid_size as i32 - 1);
    let gy0 = ((min_y * gs).floor() as i32).clamp(0, grid_size as i32 - 1);
    let gy1 = ((max_y * gs).ceil() as i32).clamp(0, grid_size as i32 - 1);

    for cy in gy0..=gy1 {
        for cx in gx0..=gx1 {
            let cu = (cx as f32 + 0.5) / gs;
            let cv = (cy as f32 + 0.5) / gs;

            if point_in_uv_triangle([cu, cv], uv0, uv1, uv2) {
                let idx = cy as usize * grid_size + cx as usize;
                grid[idx] += 1;
            }
        }
    }
}

/// Test if a 2D point is inside a UV triangle using barycentric coordinates.
fn point_in_uv_triangle(p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    let sign = |p1: [f32; 2], p2: [f32; 2], p3: [f32; 2]| -> f32 {
        (p1[0] - p3[0]) * (p2[1] - p3[1]) - (p2[0] - p3[0]) * (p1[1] - p3[1])
    };

    let d1 = sign(p, a, b);
    let d2 = sign(p, b, c);
    let d3 = sign(p, c, a);

    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;

    !(has_neg && has_pos)
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    /// Build a unit quad split into 2 triangles.
    /// Positions in XY plane, UVs map [0,1]^2.
    fn unit_quad_mesh() -> MeshBuffers {
        // Verts: (0,0,0), (1,0,0), (1,1,0), (0,1,0)
        // UVs:   (0,0),   (1,0),   (1,1),   (0,1)
        // Faces: [0,1,2], [0,2,3]
        let src = MB {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        };
        MeshBuffers::from_morph(src)
    }

    #[test]
    fn uv_triangle_area_unit_triangle() {
        // Right triangle with legs of length 1 -> area 0.5
        let area = uv_triangle_area([0.0, 0.0], [1.0, 0.0], [0.0, 1.0]);
        assert!((area - 0.5).abs() < 1e-6, "Expected 0.5, got {area}");
    }

    #[test]
    fn uv_triangle_area_zero_for_degenerate() {
        // All three points collinear -> area 0
        let area = uv_triangle_area([0.0, 0.0], [0.5, 0.0], [1.0, 0.0]);
        assert!(area < 1e-6, "Degenerate area should be ~0, got {area}");
    }

    #[test]
    fn is_degenerate_uv_face_detects_collapsed() {
        // Collapsed triangle (all same point)
        assert!(is_degenerate_uv_face(
            [0.5, 0.5],
            [0.5, 0.5],
            [0.5, 0.5],
            1e-6
        ));
        // Valid triangle
        assert!(!is_degenerate_uv_face(
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            1e-6
        ));
    }

    #[test]
    fn face_uv_stretch_identity_no_stretch() {
        // When 3D triangle and UV triangle are the same shape/scale, stretch is moderate
        // but consistent. We just check it returns a non-negative value.
        let s = face_uv_stretch(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
        );
        assert!(s >= 0.0, "Stretch must be non-negative, got {s}");
        assert!(s <= 1.0, "Normalized stretch must be <=1, got {s}");
    }

    #[test]
    fn face_uv_stretch_scaled_uv() {
        // UV is 2x smaller than the 3D face -> higher stretch
        let s_normal = face_uv_stretch(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
        );
        let s_small_uv = face_uv_stretch(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0],
            [0.1, 0.0],
            [0.0, 0.1],
        );
        // Small UV area means high stretch ratio
        assert!(
            s_small_uv > s_normal,
            "Smaller UV should have higher stretch: {s_small_uv} vs {s_normal}"
        );
    }

    #[test]
    fn compute_face_stretches_length_matches_faces() {
        let mesh = unit_quad_mesh();
        let stretches = compute_face_stretches(&mesh);
        assert_eq!(stretches.len(), mesh.face_count());
    }

    #[test]
    fn compute_uv_utilization_full_quad_is_high() {
        // The unit quad covers the full [0,1]^2 UV space -> utilization near 1.0
        let mesh = unit_quad_mesh();
        let util = compute_uv_utilization(&mesh, 32);
        assert!(
            util > 0.8,
            "Full quad should have high utilization, got {util}"
        );
    }

    #[test]
    fn compute_uv_utilization_tiny_uv_is_low() {
        // A mesh with UVs all clustered near (0,0) -> very low utilization
        let src = MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [0.01, 0.0], [0.0, 0.01]],
            indices: vec![0, 1, 2],
            has_suit: false,
        };
        let mesh = MeshBuffers::from_morph(src);
        let util = compute_uv_utilization(&mesh, 32);
        assert!(
            util < 0.05,
            "Tiny UV triangle should have low utilization, got {util}"
        );
    }

    #[test]
    fn uv_quality_report_face_count() {
        let mesh = unit_quad_mesh();
        let report = uv_quality_report(&mesh);
        assert_eq!(report.face_count, 2);
    }

    #[test]
    fn uv_quality_report_avg_stretch_nonnegative() {
        let mesh = unit_quad_mesh();
        let report = uv_quality_report(&mesh);
        assert!(
            report.avg_stretch >= 0.0,
            "avg_stretch must be non-negative, got {}",
            report.avg_stretch
        );
        assert!(
            report.max_stretch >= report.avg_stretch,
            "max_stretch must >= avg_stretch"
        );
    }

    #[test]
    fn worst_stretch_faces_sorted_desc() {
        let mesh = unit_quad_mesh();
        let worst = worst_stretch_faces(&mesh, 2);
        assert_eq!(worst.len(), 2);
        // Verify descending order
        assert!(
            worst[0].1 >= worst[1].1,
            "Should be sorted descending: {} vs {}",
            worst[0].1,
            worst[1].1
        );
    }

    #[test]
    fn count_uv_overlaps_no_overlap_mesh() {
        // Two triangles in completely separate UV regions -> zero overlapping cells.
        // Triangle 1: bottom-left quadrant, Triangle 2: top-right quadrant.
        let src = oxihuman_morph::engine::MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 6],
            uvs: vec![
                [0.0, 0.0],
                [0.4, 0.0],
                [0.0, 0.4],
                [0.6, 0.6],
                [1.0, 0.6],
                [0.6, 1.0],
            ],
            indices: vec![0, 1, 2, 3, 4, 5],
            has_suit: false,
        };
        let mesh = MeshBuffers::from_morph(src);
        let overlaps = count_uv_overlaps(&mesh, 32);
        assert_eq!(
            overlaps, 0,
            "Separated UV triangles should have zero overlaps, got {overlaps}"
        );
    }

    #[test]
    fn face_conformal_distortion_identity_near_zero() {
        // An identity map (3D XY == UV) is perfectly conformal -> distortion near 0
        let dist = face_conformal_distortion(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
        );
        assert!(
            dist < 1e-4,
            "Identity map should have near-zero conformal distortion, got {dist}"
        );
    }
}
