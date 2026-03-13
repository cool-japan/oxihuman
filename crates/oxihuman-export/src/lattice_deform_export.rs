// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lattice deform export.

/// Lattice deform export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LatticeDeformExport {
    pub resolution: [usize; 3],
    pub points: Vec<[f32; 3]>,
    pub origin: [f32; 3],
    pub size: [f32; 3],
}

#[allow(dead_code)]
pub fn new_lattice_deform(res_u: usize, res_v: usize, res_w: usize) -> LatticeDeformExport {
    let total = res_u * res_v * res_w;
    let mut points = Vec::with_capacity(total);
    for w in 0..res_w {
        for v in 0..res_v {
            for u in 0..res_u {
                points.push([
                    u as f32 / (res_u.max(1) - 1).max(1) as f32,
                    v as f32 / (res_v.max(1) - 1).max(1) as f32,
                    w as f32 / (res_w.max(1) - 1).max(1) as f32,
                ]);
            }
        }
    }
    LatticeDeformExport {
        resolution: [res_u, res_v, res_w],
        points,
        origin: [0.0; 3],
        size: [1.0; 3],
    }
}

#[allow(dead_code)]
pub fn lattice_point_count(l: &LatticeDeformExport) -> usize {
    l.points.len()
}

#[allow(dead_code)]
pub fn lattice_resolution(l: &LatticeDeformExport) -> [usize; 3] {
    l.resolution
}

#[allow(dead_code)]
pub fn lattice_set_point(l: &mut LatticeDeformExport, idx: usize, pos: [f32; 3]) {
    if idx < l.points.len() {
        l.points[idx] = pos;
    }
}

#[allow(dead_code)]
pub fn lattice_get_point(l: &LatticeDeformExport, idx: usize) -> Option<[f32; 3]> {
    l.points.get(idx).copied()
}

#[allow(dead_code)]
pub fn lattice_set_origin(l: &mut LatticeDeformExport, o: [f32; 3]) {
    l.origin = o;
}

#[allow(dead_code)]
pub fn lattice_set_size(l: &mut LatticeDeformExport, s: [f32; 3]) {
    l.size = s;
}

#[allow(dead_code)]
pub fn lattice_to_json(l: &LatticeDeformExport) -> String {
    format!(
        "{{\"resolution\":[{},{},{}],\"points\":{}}}",
        l.resolution[0],
        l.resolution[1],
        l.resolution[2],
        l.points.len()
    )
}

#[allow(dead_code)]
pub fn lattice_validate(l: &LatticeDeformExport) -> bool {
    l.resolution[0] > 0
        && l.resolution[1] > 0
        && l.resolution[2] > 0
        && l.points.len() == l.resolution[0] * l.resolution[1] * l.resolution[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_2x2x2() {
        let l = new_lattice_deform(2, 2, 2);
        assert_eq!(lattice_point_count(&l), 8);
    }

    #[test]
    fn test_resolution() {
        let l = new_lattice_deform(3, 4, 5);
        assert_eq!(lattice_resolution(&l), [3, 4, 5]);
    }

    #[test]
    fn test_set_get_point() {
        let mut l = new_lattice_deform(2, 2, 2);
        lattice_set_point(&mut l, 0, [1.0, 2.0, 3.0]);
        let p = lattice_get_point(&l, 0).expect("should succeed");
        assert!((p[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_oob() {
        assert!(lattice_get_point(&new_lattice_deform(2, 2, 2), 100).is_none());
    }

    #[test]
    fn test_set_origin() {
        let mut l = new_lattice_deform(2, 2, 2);
        lattice_set_origin(&mut l, [1.0; 3]);
        assert!((l.origin[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_size() {
        let mut l = new_lattice_deform(2, 2, 2);
        lattice_set_size(&mut l, [2.0; 3]);
        assert!((l.size[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        assert!(lattice_to_json(&new_lattice_deform(2, 2, 2)).contains("\"points\":8"));
    }

    #[test]
    fn test_validate_ok() {
        assert!(lattice_validate(&new_lattice_deform(2, 2, 2)));
    }

    #[test]
    fn test_validate_mismatch() {
        let mut l = new_lattice_deform(2, 2, 2);
        l.points.pop();
        assert!(!lattice_validate(&l));
    }

    #[test]
    fn test_3x3x3() {
        let l = new_lattice_deform(3, 3, 3);
        assert_eq!(lattice_point_count(&l), 27);
    }
}
