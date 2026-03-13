// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-face-corner UV coordinate export (loop UV layout).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceCornerUv {
    pub face_index: u32,
    pub corner_index: u8,
    pub uv: [f32; 2],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceCornerUvExport {
    pub corners: Vec<FaceCornerUv>,
}

#[allow(dead_code)]
pub fn new_face_corner_uv_export() -> FaceCornerUvExport {
    FaceCornerUvExport {
        corners: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_corner_uv(exp: &mut FaceCornerUvExport, face: u32, corner: u8, uv: [f32; 2]) {
    exp.corners.push(FaceCornerUv {
        face_index: face,
        corner_index: corner,
        uv,
    });
}

#[allow(dead_code)]
pub fn corner_count(exp: &FaceCornerUvExport) -> usize {
    exp.corners.len()
}

#[allow(dead_code)]
pub fn corners_for_face(exp: &FaceCornerUvExport, face: u32) -> Vec<&FaceCornerUv> {
    exp.corners
        .iter()
        .filter(|c| c.face_index == face)
        .collect()
}

#[allow(dead_code)]
pub fn uv_bounds(exp: &FaceCornerUvExport) -> ([f32; 2], [f32; 2]) {
    if exp.corners.is_empty() {
        return ([0.0; 2], [0.0; 2]);
    }
    let mut min = [f32::MAX; 2];
    let mut max = [f32::MIN; 2];
    for c in &exp.corners {
        min[0] = min[0].min(c.uv[0]);
        min[1] = min[1].min(c.uv[1]);
        max[0] = max[0].max(c.uv[0]);
        max[1] = max[1].max(c.uv[1]);
    }
    (min, max)
}

#[allow(dead_code)]
pub fn face_count_fcuv(exp: &FaceCornerUvExport) -> usize {
    let mut faces: Vec<u32> = exp.corners.iter().map(|c| c.face_index).collect();
    faces.sort_unstable();
    faces.dedup();
    faces.len()
}

#[allow(dead_code)]
pub fn uvs_in_unit_range(exp: &FaceCornerUvExport) -> bool {
    exp.corners
        .iter()
        .all(|c| (0.0..=1.0).contains(&c.uv[0]) && (0.0..=1.0).contains(&c.uv[1]))
}

#[allow(dead_code)]
pub fn face_corner_uv_to_json(exp: &FaceCornerUvExport) -> String {
    format!(
        "{{\"corner_count\":{},\"face_count\":{}}}",
        corner_count(exp),
        face_count_fcuv(exp)
    )
}

#[allow(dead_code)]
pub fn avg_uv(exp: &FaceCornerUvExport) -> [f32; 2] {
    if exp.corners.is_empty() {
        return [0.0; 2];
    }
    let n = exp.corners.len() as f32;
    let su: f32 = exp.corners.iter().map(|c| c.uv[0]).sum();
    let sv: f32 = exp.corners.iter().map(|c| c.uv[1]).sum();
    [su / n, sv / n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let exp = new_face_corner_uv_export();
        assert_eq!(corner_count(&exp), 0);
    }

    #[test]
    fn test_add_corner() {
        let mut exp = new_face_corner_uv_export();
        add_corner_uv(&mut exp, 0, 0, [0.0, 0.0]);
        assert_eq!(corner_count(&exp), 1);
    }

    #[test]
    fn test_corners_for_face() {
        let mut exp = new_face_corner_uv_export();
        add_corner_uv(&mut exp, 0, 0, [0.0, 0.0]);
        add_corner_uv(&mut exp, 0, 1, [1.0, 0.0]);
        add_corner_uv(&mut exp, 1, 0, [0.5, 0.5]);
        assert_eq!(corners_for_face(&exp, 0).len(), 2);
    }

    #[test]
    fn test_uv_bounds() {
        let mut exp = new_face_corner_uv_export();
        add_corner_uv(&mut exp, 0, 0, [0.2, 0.3]);
        add_corner_uv(&mut exp, 0, 1, [0.8, 0.7]);
        let (min, max) = uv_bounds(&exp);
        assert!((min[0] - 0.2).abs() < 1e-5);
        assert!((max[0] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_face_count() {
        let mut exp = new_face_corner_uv_export();
        add_corner_uv(&mut exp, 0, 0, [0.0, 0.0]);
        add_corner_uv(&mut exp, 1, 0, [1.0, 1.0]);
        assert_eq!(face_count_fcuv(&exp), 2);
    }

    #[test]
    fn test_uvs_in_unit_range() {
        let mut exp = new_face_corner_uv_export();
        add_corner_uv(&mut exp, 0, 0, [0.5, 0.5]);
        assert!(uvs_in_unit_range(&exp));
    }

    #[test]
    fn test_uvs_out_of_range() {
        let mut exp = new_face_corner_uv_export();
        add_corner_uv(&mut exp, 0, 0, [1.5, 0.5]);
        assert!(!uvs_in_unit_range(&exp));
    }

    #[test]
    fn test_avg_uv() {
        let mut exp = new_face_corner_uv_export();
        add_corner_uv(&mut exp, 0, 0, [0.0, 0.0]);
        add_corner_uv(&mut exp, 0, 1, [1.0, 1.0]);
        let avg = avg_uv(&exp);
        assert!((avg[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_json_output() {
        let exp = new_face_corner_uv_export();
        let j = face_corner_uv_to_json(&exp);
        assert!(j.contains("corner_count"));
    }

    #[test]
    fn test_avg_empty() {
        let exp = new_face_corner_uv_export();
        let avg = avg_uv(&exp);
        assert!((avg[0]).abs() < 1e-6);
    }
}
