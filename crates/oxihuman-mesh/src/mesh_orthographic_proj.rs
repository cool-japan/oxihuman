// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Orthographic projection of mesh vertices onto 2D planes.
#[allow(dead_code)]
pub enum OrthoProjectionAxis {
    XY,
    XZ,
    YZ,
}

#[allow(dead_code)]
pub struct OrthoProjectionResult {
    pub projected: Vec<[f32; 2]>,
    pub axis: OrthoProjectionAxis,
    pub bounds_2d: ([f32; 2], [f32; 2]),
}

#[allow(dead_code)]
pub fn project_orthographic(
    positions: &[[f32; 3]],
    axis: OrthoProjectionAxis,
) -> OrthoProjectionResult {
    let projected: Vec<[f32; 2]> = positions
        .iter()
        .map(|&p| match axis {
            OrthoProjectionAxis::XY => [p[0], p[1]],
            OrthoProjectionAxis::XZ => [p[0], p[2]],
            OrthoProjectionAxis::YZ => [p[1], p[2]],
        })
        .collect();

    let bounds_2d = if projected.is_empty() {
        ([0.0f32; 2], [0.0f32; 2])
    } else {
        let mut mn = projected[0];
        let mut mx = projected[0];
        for &p in &projected {
            if p[0] < mn[0] {
                mn[0] = p[0];
            }
            if p[1] < mn[1] {
                mn[1] = p[1];
            }
            if p[0] > mx[0] {
                mx[0] = p[0];
            }
            if p[1] > mx[1] {
                mx[1] = p[1];
            }
        }
        (mn, mx)
    };

    OrthoProjectionResult {
        projected,
        axis,
        bounds_2d,
    }
}

#[allow(dead_code)]
pub fn projected_area(r: &OrthoProjectionResult) -> f32 {
    let (mn, mx) = r.bounds_2d;
    let w = (mx[0] - mn[0]).abs();
    let h = (mx[1] - mn[1]).abs();
    w * h
}

#[allow(dead_code)]
pub fn projected_centroid(r: &OrthoProjectionResult) -> [f32; 2] {
    if r.projected.is_empty() {
        return [0.0; 2];
    }
    let n = r.projected.len() as f32;
    let mut s = [0.0f32; 2];
    for &p in &r.projected {
        s[0] += p[0];
        s[1] += p[1];
    }
    [s[0] / n, s[1] / n]
}

#[allow(dead_code)]
pub fn normalize_projected(r: &mut OrthoProjectionResult) {
    let (mn, mx) = r.bounds_2d;
    let w = (mx[0] - mn[0]).abs().max(1e-10);
    let h = (mx[1] - mn[1]).abs().max(1e-10);
    for p in &mut r.projected {
        p[0] = (p[0] - mn[0]) / w;
        p[1] = (p[1] - mn[1]) / h;
    }
    r.bounds_2d = ([0.0, 0.0], [1.0, 1.0]);
}

#[allow(dead_code)]
pub fn projected_to_json(r: &OrthoProjectionResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"area\":{}}}",
        r.projected.len(),
        projected_area(r)
    )
}

#[allow(dead_code)]
pub fn project_to_image_space(r: &OrthoProjectionResult, width: u32, height: u32) -> Vec<[u32; 2]> {
    let (mn, mx) = r.bounds_2d;
    let rw = (mx[0] - mn[0]).abs().max(1e-10);
    let rh = (mx[1] - mn[1]).abs().max(1e-10);
    r.projected
        .iter()
        .map(|&p| {
            let u = ((p[0] - mn[0]) / rw * (width as f32 - 1.0)).round() as u32;
            let v = ((p[1] - mn[1]) / rh * (height as f32 - 1.0)).round() as u32;
            [u.min(width - 1), v.min(height - 1)]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn test_xy_projection_count() {
        let verts = cube_verts();
        let r = project_orthographic(&verts, OrthoProjectionAxis::XY);
        assert_eq!(r.projected.len(), 4);
    }

    #[test]
    fn test_xz_projection_z_coordinate() {
        let verts = cube_verts();
        let r = project_orthographic(&verts, OrthoProjectionAxis::XZ);
        for p in &r.projected {
            assert!((p[1] - 0.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_yz_projection_x_maps_to_y() {
        let verts = vec![[0.0, 2.0, 3.0]];
        let r = project_orthographic(&verts, OrthoProjectionAxis::YZ);
        assert!((r.projected[0][0] - 2.0).abs() < 1e-6);
        assert!((r.projected[0][1] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_projected_area_positive() {
        let verts = cube_verts();
        let r = project_orthographic(&verts, OrthoProjectionAxis::XY);
        assert!(projected_area(&r) > 0.0);
    }

    #[test]
    fn test_projected_centroid() {
        let verts = cube_verts();
        let r = project_orthographic(&verts, OrthoProjectionAxis::XY);
        let c = projected_centroid(&r);
        assert!((c[0] - 0.5).abs() < 1e-5);
        assert!((c[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_empty_projection() {
        let r = project_orthographic(&[], OrthoProjectionAxis::XY);
        assert_eq!(r.projected.len(), 0);
    }

    #[test]
    fn test_normalize_bounds_0_to_1() {
        let verts = cube_verts();
        let mut r = project_orthographic(&verts, OrthoProjectionAxis::XY);
        normalize_projected(&mut r);
        let (mn, mx) = r.bounds_2d;
        assert!((mn[0]).abs() < 1e-5);
        assert!((mx[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_image_space_bounds() {
        let verts = cube_verts();
        let r = project_orthographic(&verts, OrthoProjectionAxis::XY);
        let pixels = project_to_image_space(&r, 64, 64);
        for p in pixels {
            assert!(p[0] < 64);
            assert!(p[1] < 64);
        }
    }

    #[test]
    fn test_to_json() {
        let verts = cube_verts();
        let r = project_orthographic(&verts, OrthoProjectionAxis::XY);
        let j = projected_to_json(&r);
        assert!(j.contains("vertex_count"));
    }

    #[test]
    fn test_bounds_single_vertex() {
        let r = project_orthographic(&[[1.0, 2.0, 3.0]], OrthoProjectionAxis::XY);
        let (mn, mx) = r.bounds_2d;
        assert!((mn[0] - 1.0).abs() < 1e-6);
        assert!((mx[0] - 1.0).abs() < 1e-6);
    }
}
