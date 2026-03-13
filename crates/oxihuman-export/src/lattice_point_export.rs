// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lattice point export: FFD (Free Form Deformation) lattice control points.

/// A single FFD lattice control point.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct LatticePoint {
    pub index: [u32; 3],
    pub rest_position: [f32; 3],
    pub deformed_position: [f32; 3],
}

/// Lattice point export bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LatticePointExport {
    pub points: Vec<LatticePoint>,
    pub resolution: [u32; 3],
}

/// Create a new lattice export with uniform resolution.
#[allow(dead_code)]
pub fn new_lattice_point_export(rx: u32, ry: u32, rz: u32) -> LatticePointExport {
    let mut points = Vec::new();
    for z in 0..rz {
        for y in 0..ry {
            for x in 0..rx {
                let rest = [
                    x as f32 / (rx - 1).max(1) as f32,
                    y as f32 / (ry - 1).max(1) as f32,
                    z as f32 / (rz - 1).max(1) as f32,
                ];
                points.push(LatticePoint {
                    index: [x, y, z],
                    rest_position: rest,
                    deformed_position: rest,
                });
            }
        }
    }
    LatticePointExport {
        points,
        resolution: [rx, ry, rz],
    }
}

/// Point count.
#[allow(dead_code)]
pub fn lattice_point_count_lp(exp: &LatticePointExport) -> usize {
    exp.points.len()
}

/// Find point by index.
#[allow(dead_code)]
pub fn find_lattice_point(
    exp: &LatticePointExport,
    ix: u32,
    iy: u32,
    iz: u32,
) -> Option<&LatticePoint> {
    exp.points.iter().find(|p| p.index == [ix, iy, iz])
}

/// Displace a point.
#[allow(dead_code)]
pub fn displace_lattice_point(
    exp: &mut LatticePointExport,
    ix: u32,
    iy: u32,
    iz: u32,
    delta: [f32; 3],
) {
    if let Some(p) = exp.points.iter_mut().find(|p| p.index == [ix, iy, iz]) {
        p.deformed_position[0] = p.rest_position[0] + delta[0];
        p.deformed_position[1] = p.rest_position[1] + delta[1];
        p.deformed_position[2] = p.rest_position[2] + delta[2];
    }
}

/// Max displacement magnitude.
#[allow(dead_code)]
pub fn max_lattice_displacement(exp: &LatticePointExport) -> f32 {
    exp.points
        .iter()
        .map(|p| {
            let d = [
                p.deformed_position[0] - p.rest_position[0],
                p.deformed_position[1] - p.rest_position[1],
                p.deformed_position[2] - p.rest_position[2],
            ];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0_f32, f32::max)
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn lattice_point_to_json(exp: &LatticePointExport) -> String {
    format!(
        "{{\"point_count\":{},\"resolution\":[{},{},{}]}}",
        lattice_point_count_lp(exp),
        exp.resolution[0],
        exp.resolution[1],
        exp.resolution[2]
    )
}

/// Validate: all rest positions in `[0,1]`.
#[allow(dead_code)]
pub fn validate_lattice_points(exp: &LatticePointExport) -> bool {
    exp.points
        .iter()
        .all(|p| p.rest_position.iter().all(|&v| (0.0..=1.0).contains(&v)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_count_2x2x2() {
        let exp = new_lattice_point_export(2, 2, 2);
        assert_eq!(lattice_point_count_lp(&exp), 8);
    }

    #[test]
    fn find_corner_point() {
        let exp = new_lattice_point_export(2, 2, 2);
        assert!(find_lattice_point(&exp, 0, 0, 0).is_some());
    }

    #[test]
    fn rest_positions_in_range() {
        let exp = new_lattice_point_export(3, 3, 3);
        assert!(validate_lattice_points(&exp));
    }

    #[test]
    fn displace_changes_position() {
        let mut exp = new_lattice_point_export(2, 2, 2);
        displace_lattice_point(&mut exp, 0, 0, 0, [0.5, 0.0, 0.0]);
        let p = find_lattice_point(&exp, 0, 0, 0).expect("should succeed");
        assert!((p.deformed_position[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn max_displacement_zero_initially() {
        let exp = new_lattice_point_export(2, 2, 2);
        assert!((max_lattice_displacement(&exp)).abs() < 1e-6);
    }

    #[test]
    fn max_displacement_after_displace() {
        let mut exp = new_lattice_point_export(2, 2, 2);
        displace_lattice_point(&mut exp, 0, 0, 0, [1.0, 0.0, 0.0]);
        assert!(max_lattice_displacement(&exp) > 0.0);
    }

    #[test]
    fn resolution_stored() {
        let exp = new_lattice_point_export(4, 3, 2);
        assert_eq!(exp.resolution, [4, 3, 2]);
    }

    #[test]
    fn json_contains_point_count() {
        let exp = new_lattice_point_export(2, 2, 2);
        let j = lattice_point_to_json(&exp);
        assert!(j.contains("point_count"));
    }

    #[test]
    fn find_missing_none() {
        let exp = new_lattice_point_export(2, 2, 2);
        assert!(find_lattice_point(&exp, 5, 5, 5).is_none());
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
