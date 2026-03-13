// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export a lattice deformation cage.

/// Resolution of the lattice (nx × ny × nz control points).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct LatticeResV2 {
    pub nx: u32,
    pub ny: u32,
    pub nz: u32,
}

/// A lattice deformation export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeformLatticeExport {
    pub resolution: LatticeResV2,
    /// Control point positions (rest pose).
    pub rest_points: Vec<[f32; 3]>,
    /// Control point positions (deformed).
    pub deformed_points: Vec<[f32; 3]>,
}

/// Create a default (identity) lattice.
#[allow(dead_code)]
pub fn new_deform_lattice(res: LatticeResV2) -> DeformLatticeExport {
    let n = (res.nx * res.ny * res.nz) as usize;
    let mut rest = Vec::with_capacity(n);
    for iz in 0..res.nz {
        for iy in 0..res.ny {
            for ix in 0..res.nx {
                let x = if res.nx > 1 {
                    ix as f32 / (res.nx - 1) as f32
                } else {
                    0.5
                };
                let y = if res.ny > 1 {
                    iy as f32 / (res.ny - 1) as f32
                } else {
                    0.5
                };
                let z = if res.nz > 1 {
                    iz as f32 / (res.nz - 1) as f32
                } else {
                    0.5
                };
                rest.push([x, y, z]);
            }
        }
    }
    let deformed = rest.clone();
    DeformLatticeExport {
        resolution: res,
        rest_points: rest,
        deformed_points: deformed,
    }
}

/// Count control points.
#[allow(dead_code)]
pub fn lattice_control_point_count(export: &DeformLatticeExport) -> usize {
    export.rest_points.len()
}

/// Set the deformed position of a control point.
#[allow(dead_code)]
pub fn set_lattice_deformed(export: &mut DeformLatticeExport, index: usize, pos: [f32; 3]) {
    if index < export.deformed_points.len() {
        export.deformed_points[index] = pos;
    }
}

/// Compute the displacement at a control point.
#[allow(dead_code)]
pub fn lattice_displacement_at(export: &DeformLatticeExport, index: usize) -> [f32; 3] {
    if index >= export.rest_points.len() {
        return [0.0; 3];
    }
    let r = export.rest_points[index];
    let d = export.deformed_points[index];
    [d[0] - r[0], d[1] - r[1], d[2] - r[2]]
}

/// Compute the average displacement magnitude.
#[allow(dead_code)]
pub fn avg_lattice_displacement_v2(export: &DeformLatticeExport) -> f32 {
    let n = export.rest_points.len();
    if n == 0 {
        return 0.0;
    }
    (0..n)
        .map(|i| {
            let d = lattice_displacement_at(export, i);
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .sum::<f32>()
        / n as f32
}

/// Validate that rest and deformed point counts match.
#[allow(dead_code)]
pub fn validate_deform_lattice(export: &DeformLatticeExport) -> bool {
    export.rest_points.len() == export.deformed_points.len()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn deform_lattice_to_json(export: &DeformLatticeExport) -> String {
    format!(
        "{{\"nx\":{},\"ny\":{},\"nz\":{},\"points\":{}}}",
        export.resolution.nx,
        export.resolution.ny,
        export.resolution.nz,
        export.rest_points.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_two_two() -> DeformLatticeExport {
        new_deform_lattice(LatticeResV2 {
            nx: 2,
            ny: 2,
            nz: 2,
        })
    }

    #[test]
    fn test_control_point_count() {
        let e = two_two_two();
        assert_eq!(lattice_control_point_count(&e), 8);
    }

    #[test]
    fn test_identity_displacement_zero() {
        let e = two_two_two();
        let d = lattice_displacement_at(&e, 0);
        assert!(d.iter().all(|&v| v.abs() < 1e-6));
    }

    #[test]
    fn test_set_and_get_displacement() {
        let mut e = two_two_two();
        set_lattice_deformed(&mut e, 0, [0.5, 0.0, 0.0]);
        let d = lattice_displacement_at(&e, 0);
        assert!((d[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_avg_displacement_identity() {
        let e = two_two_two();
        assert!(avg_lattice_displacement_v2(&e).abs() < 1e-6);
    }

    #[test]
    fn test_validate_valid() {
        let e = two_two_two();
        assert!(validate_deform_lattice(&e));
    }

    #[test]
    fn test_validate_invalid() {
        let mut e = two_two_two();
        e.rest_points.push([0.0, 0.0, 0.0]);
        assert!(!validate_deform_lattice(&e));
    }

    #[test]
    fn test_deform_lattice_to_json() {
        let e = two_two_two();
        let j = deform_lattice_to_json(&e);
        assert!(j.contains("points"));
    }

    #[test]
    fn test_set_oob_no_panic() {
        let mut e = two_two_two();
        set_lattice_deformed(&mut e, 999, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_displacement_oob_zero() {
        let e = two_two_two();
        let d = lattice_displacement_at(&e, 999);
        assert_eq!(d, [0.0; 3]);
    }

    #[test]
    fn test_empty_lattice() {
        let e = new_deform_lattice(LatticeResV2 {
            nx: 0,
            ny: 0,
            nz: 0,
        });
        assert!(avg_lattice_displacement_v2(&e).abs() < 1e-6);
    }
}
