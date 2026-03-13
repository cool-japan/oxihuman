// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Export per-face-corner (loop) data: UVs, normals, colors, tangents.
#[allow(dead_code)]
pub struct FaceCorner {
    pub face_idx: u32,
    pub corner_idx: u32,
    pub vertex_idx: u32,
    pub uv: [f32; 2],
    pub normal: [f32; 3],
    pub color: [f32; 4],
    pub tangent: [f32; 4],
}

#[allow(dead_code)]
pub struct FaceCornerExport {
    pub corners: Vec<FaceCorner>,
    pub face_count: usize,
}

#[allow(dead_code)]
pub fn new_face_corner_export(face_count: usize) -> FaceCornerExport {
    FaceCornerExport {
        corners: vec![],
        face_count,
    }
}

#[allow(dead_code)]
pub fn add_face_corner(export: &mut FaceCornerExport, corner: FaceCorner) {
    export.corners.push(corner);
}

#[allow(dead_code)]
pub fn corner_count(export: &FaceCornerExport) -> usize {
    export.corners.len()
}

#[allow(dead_code)]
pub fn corners_for_face(export: &FaceCornerExport, face_idx: u32) -> Vec<&FaceCorner> {
    export
        .corners
        .iter()
        .filter(|c| c.face_idx == face_idx)
        .collect()
}

#[allow(dead_code)]
pub fn validate_face_corners(export: &FaceCornerExport) -> bool {
    export
        .corners
        .iter()
        .all(|c| c.uv.iter().all(|&v| v.is_finite()) && c.normal.iter().all(|&v| v.is_finite()))
}

#[allow(dead_code)]
pub fn face_corner_to_json(export: &FaceCornerExport) -> String {
    format!(
        "{{\"corner_count\":{},\"face_count\":{}}}",
        export.corners.len(),
        export.face_count
    )
}

#[allow(dead_code)]
pub fn avg_uv(export: &FaceCornerExport) -> [f32; 2] {
    if export.corners.is_empty() {
        return [0.0; 2];
    }
    let n = export.corners.len() as f32;
    let mut s = [0.0f32; 2];
    for c in &export.corners {
        s[0] += c.uv[0];
        s[1] += c.uv[1];
    }
    [s[0] / n, s[1] / n]
}

#[allow(dead_code)]
pub fn normals_are_unit(export: &FaceCornerExport) -> bool {
    export.corners.iter().all(|c| {
        let l = (c.normal[0] * c.normal[0] + c.normal[1] * c.normal[1] + c.normal[2] * c.normal[2])
            .sqrt();
        (l - 1.0).abs() < 0.01
    })
}

#[allow(dead_code)]
pub fn uv_bounds(export: &FaceCornerExport) -> ([f32; 2], [f32; 2]) {
    if export.corners.is_empty() {
        return ([0.0; 2], [0.0; 2]);
    }
    let mut mn = export.corners[0].uv;
    let mut mx = export.corners[0].uv;
    for c in &export.corners {
        if c.uv[0] < mn[0] {
            mn[0] = c.uv[0];
        }
        if c.uv[1] < mn[1] {
            mn[1] = c.uv[1];
        }
        if c.uv[0] > mx[0] {
            mx[0] = c.uv[0];
        }
        if c.uv[1] > mx[1] {
            mx[1] = c.uv[1];
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn build_triangle_export(positions: &[[f32; 3]], indices: &[u32]) -> FaceCornerExport {
    let tri_count = indices.len() / 3;
    let mut export = new_face_corner_export(tri_count);
    for t in 0..tri_count {
        for vi in 0..3u32 {
            let idx = indices[t * 3 + vi as usize];
            let p = if (idx as usize) < positions.len() {
                positions[idx as usize]
            } else {
                [0.0; 3]
            };
            add_face_corner(
                &mut export,
                FaceCorner {
                    face_idx: t as u32,
                    corner_idx: vi,
                    vertex_idx: idx,
                    uv: [p[0], p[2]],
                    normal: [0.0, 1.0, 0.0],
                    color: [1.0; 4],
                    tangent: [1.0, 0.0, 0.0, 1.0],
                },
            );
        }
    }
    export
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_export() -> FaceCornerExport {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3u32];
        build_triangle_export(&pos, &idx)
    }

    #[test]
    fn test_corner_count() {
        let e = quad_export();
        assert_eq!(corner_count(&e), 6);
    }

    #[test]
    fn test_corners_for_face() {
        let e = quad_export();
        let face0 = corners_for_face(&e, 0);
        assert_eq!(face0.len(), 3);
    }

    #[test]
    fn test_validate_corners() {
        let e = quad_export();
        assert!(validate_face_corners(&e));
    }

    #[test]
    fn test_face_count() {
        let e = quad_export();
        assert_eq!(e.face_count, 2);
    }

    #[test]
    fn test_avg_uv_finite() {
        let e = quad_export();
        let uv = avg_uv(&e);
        assert!(uv[0].is_finite() && uv[1].is_finite());
    }

    #[test]
    fn test_normals_are_unit() {
        let e = quad_export();
        assert!(normals_are_unit(&e));
    }

    #[test]
    fn test_uv_bounds() {
        let e = quad_export();
        let (mn, mx) = uv_bounds(&e);
        assert!(mx[0] >= mn[0]);
    }

    #[test]
    fn test_to_json() {
        let e = quad_export();
        let j = face_corner_to_json(&e);
        assert!(j.contains("corner_count"));
    }

    #[test]
    fn test_empty_export() {
        let e = new_face_corner_export(0);
        assert_eq!(corner_count(&e), 0);
    }

    #[test]
    fn test_empty_avg_uv() {
        let e = new_face_corner_export(0);
        assert_eq!(avg_uv(&e), [0.0; 2]);
    }
}
