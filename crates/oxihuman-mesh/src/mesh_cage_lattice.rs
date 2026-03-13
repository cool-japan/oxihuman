// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lattice cage deformation (free-form deformation / FFD).

use std::f32::consts::PI;

/// A 3D lattice control point grid for FFD.
#[derive(Debug, Clone)]
pub struct LatticeCage {
    /// Number of divisions along each axis (l+1, m+1, n+1 control points).
    pub divisions: [usize; 3],
    /// Control point positions in 3D space (row-major: `[l][m][n]`).
    pub control_points: Vec<[f32; 3]>,
    /// Origin of the lattice bounding box.
    pub origin: [f32; 3],
    /// Size (extent) of the lattice box.
    pub size: [f32; 3],
}

/// Result of applying a lattice cage deformation.
#[derive(Debug, Clone)]
pub struct LatticeCageResult {
    pub positions: Vec<[f32; 3]>,
    pub moved_count: usize,
}

/// Create a new lattice cage encompassing the given bounding box.
pub fn new_lattice_cage(origin: [f32; 3], size: [f32; 3], divisions: [usize; 3]) -> LatticeCage {
    let (l, m, n) = (divisions[0] + 1, divisions[1] + 1, divisions[2] + 1);
    let total = l * m * n;
    let mut control_points = Vec::with_capacity(total);
    for li in 0..l {
        for mi in 0..m {
            for ni in 0..n {
                let x = origin[0] + size[0] * li as f32 / divisions[0].max(1) as f32;
                let y = origin[1] + size[1] * mi as f32 / divisions[1].max(1) as f32;
                let z = origin[2] + size[2] * ni as f32 / divisions[2].max(1) as f32;
                control_points.push([x, y, z]);
            }
        }
    }
    LatticeCage {
        divisions,
        control_points,
        origin,
        size,
    }
}

/// Compute Bernstein polynomial B(i, n, t).
pub fn bernstein(i: usize, n: usize, t: f32) -> f32 {
    binomial(n, i) as f32 * t.powi(i as i32) * (1.0 - t).powi((n - i) as i32)
}

fn binomial(n: usize, k: usize) -> u64 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut result = 1u64;
    for i in 0..k {
        result = result * (n - i) as u64 / (i + 1) as u64;
    }
    result
}

/// Compute the (s, t, u) parametric coordinates of a point inside the lattice.
pub fn world_to_lattice_params(cage: &LatticeCage, point: [f32; 3]) -> [f32; 3] {
    [
        (point[0] - cage.origin[0]) / cage.size[0].max(1e-8),
        (point[1] - cage.origin[1]) / cage.size[1].max(1e-8),
        (point[2] - cage.origin[2]) / cage.size[2].max(1e-8),
    ]
}

fn cp_index(cage: &LatticeCage, l: usize, m: usize, n: usize) -> usize {
    let m_pts = cage.divisions[1] + 1;
    let n_pts = cage.divisions[2] + 1;
    l * m_pts * n_pts + m * n_pts + n
}

/// Evaluate the FFD at parametric coordinates (s, t, u).
pub fn evaluate_ffd(cage: &LatticeCage, stu: [f32; 3]) -> [f32; 3] {
    let l = cage.divisions[0];
    let m = cage.divisions[1];
    let n = cage.divisions[2];
    let [s, t, u] = stu;
    let mut result = [0.0f32; 3];
    for li in 0..=l {
        let bl = bernstein(li, l, s);
        for mi in 0..=m {
            let bm = bernstein(mi, m, t);
            for ni in 0..=n {
                let bn = bernstein(ni, n, u);
                let idx = cp_index(cage, li, mi, ni);
                let cp = cage.control_points[idx];
                let w = bl * bm * bn;
                result[0] += w * cp[0];
                result[1] += w * cp[1];
                result[2] += w * cp[2];
            }
        }
    }
    result
}

/// Apply the lattice cage deformation to a set of positions.
pub fn apply_lattice_deform(cage: &LatticeCage, positions: &[[f32; 3]]) -> LatticeCageResult {
    let mut out = Vec::with_capacity(positions.len());
    let mut moved_count = 0usize;
    for &pos in positions {
        let stu = world_to_lattice_params(cage, pos);
        if stu.iter().all(|&v| (0.0..=1.0).contains(&v)) {
            let new_pos = evaluate_ffd(cage, stu);
            let d = (0..3)
                .map(|i| (new_pos[i] - pos[i]).abs())
                .fold(0.0f32, f32::max);
            if d > 1e-7 {
                moved_count += 1;
            }
            out.push(new_pos);
        } else {
            out.push(pos);
        }
    }
    LatticeCageResult {
        positions: out,
        moved_count,
    }
}

/// Translate a single control point by delta.
pub fn move_control_point(cage: &mut LatticeCage, index: usize, delta: [f32; 3]) {
    if let Some(cp) = cage.control_points.get_mut(index) {
        cp[0] += delta[0];
        cp[1] += delta[1];
        cp[2] += delta[2];
    }
}

/// Return control point count.
pub fn control_point_count(cage: &LatticeCage) -> usize {
    cage.control_points.len()
}

/// Compute a "twist" deformation on the control points around the Y axis.
pub fn twist_lattice(cage: &mut LatticeCage, angle_per_unit_y: f32) {
    let origin_y = cage.origin[1];
    let size_y = cage.size[1];
    for cp in cage.control_points.iter_mut() {
        let t = (cp[1] - origin_y) / size_y.max(1e-8);
        let angle = angle_per_unit_y * t;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let x = cp[0] - cage.origin[0];
        let z = cp[2] - cage.origin[2];
        cp[0] = cage.origin[0] + cos_a * x - sin_a * z;
        cp[2] = cage.origin[2] + sin_a * x + cos_a * z;
    }
}

/// Reset the cage control points to their default (undeformed) positions.
pub fn reset_lattice(cage: &mut LatticeCage) {
    *cage = new_lattice_cage(cage.origin, cage.size, cage.divisions);
}

/// Validate that the cage has the correct number of control points.
pub fn validate_lattice(cage: &LatticeCage) -> bool {
    let expected = (cage.divisions[0] + 1) * (cage.divisions[1] + 1) * (cage.divisions[2] + 1);
    cage.control_points.len() == expected
}

/// Compute the bounding box of the deformed control points.
pub fn lattice_deformed_bounds(cage: &LatticeCage) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for cp in &cage.control_points {
        for i in 0..3 {
            mn[i] = mn[i].min(cp[i]);
            mx[i] = mx[i].max(cp[i]);
        }
    }
    (mn, mx)
}

/// Compute the approximate volume of the lattice bounding box.
pub fn lattice_volume(cage: &LatticeCage) -> f32 {
    cage.size[0] * cage.size[1] * cage.size[2]
}

/// Compute a bend deformation on control points (bending around Z).
pub fn bend_lattice(cage: &mut LatticeCage, bend_angle_rad: f32) {
    let _ = PI; // use constant
    let origin_x = cage.origin[0];
    let size_x = cage.size[0];
    for cp in cage.control_points.iter_mut() {
        let t = (cp[0] - origin_x) / size_x.max(1e-8);
        let angle = bend_angle_rad * t;
        let radius = size_x / bend_angle_rad.abs().max(1e-8);
        cp[1] += radius * (1.0 - angle.cos());
        cp[0] = origin_x + radius * angle.sin() + (cp[0] - origin_x) * (1.0 - t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /* basic cage creation */
    #[test]
    fn test_new_cage_point_count() {
        let cage = new_lattice_cage([0.0; 3], [1.0; 3], [2, 2, 2]);
        assert_eq!(cage.control_points.len(), 3 * 3 * 3);
    }

    /* validate */
    #[test]
    fn test_validate_lattice() {
        let cage = new_lattice_cage([0.0; 3], [1.0; 3], [1, 1, 1]);
        assert!(validate_lattice(&cage));
    }

    /* bernstein at endpoints */
    #[test]
    fn test_bernstein_endpoints() {
        assert!((bernstein(0, 2, 0.0) - 1.0).abs() < 1e-6);
        assert!((bernstein(2, 2, 1.0) - 1.0).abs() < 1e-6);
    }

    /* evaluate ffd at origin → should match first cp */
    #[test]
    fn test_evaluate_ffd_at_origin() {
        let cage = new_lattice_cage([0.0; 3], [2.0; 3], [1, 1, 1]);
        let result = evaluate_ffd(&cage, [0.0, 0.0, 0.0]);
        assert!(result[0].abs() < 1e-5 && result[1].abs() < 1e-5 && result[2].abs() < 1e-5);
    }

    /* apply deform returns same count */
    #[test]
    fn test_apply_deform_count() {
        let cage = new_lattice_cage([0.0; 3], [1.0; 3], [1, 1, 1]);
        let pts = vec![[0.5, 0.5, 0.5], [0.2, 0.3, 0.4]];
        let res = apply_lattice_deform(&cage, &pts);
        assert_eq!(res.positions.len(), 2);
    }

    /* move control point */
    #[test]
    fn test_move_control_point() {
        let mut cage = new_lattice_cage([0.0; 3], [1.0; 3], [1, 1, 1]);
        let before = cage.control_points[0];
        move_control_point(&mut cage, 0, [0.1, 0.2, 0.3]);
        let after = cage.control_points[0];
        assert!((after[0] - before[0] - 0.1).abs() < 1e-6);
    }

    /* reset lattice */
    #[test]
    fn test_reset_lattice() {
        let mut cage = new_lattice_cage([0.0; 3], [1.0; 3], [1, 1, 1]);
        move_control_point(&mut cage, 0, [5.0, 5.0, 5.0]);
        reset_lattice(&mut cage);
        assert!(validate_lattice(&cage));
        let cp = cage.control_points[0];
        assert!(cp[0].abs() < 1e-5 && cp[1].abs() < 1e-5 && cp[2].abs() < 1e-5);
    }

    /* control_point_count */
    #[test]
    fn test_control_point_count() {
        let cage = new_lattice_cage([0.0; 3], [1.0; 3], [3, 3, 3]);
        assert_eq!(control_point_count(&cage), 4 * 4 * 4);
    }

    /* lattice_volume */
    #[test]
    fn test_lattice_volume() {
        let cage = new_lattice_cage([0.0; 3], [2.0, 3.0, 4.0], [1, 1, 1]);
        assert!((lattice_volume(&cage) - 24.0).abs() < 1e-5);
    }

    /* world_to_lattice_params */
    #[test]
    fn test_world_to_lattice_params_center() {
        let cage = new_lattice_cage([0.0; 3], [2.0; 3], [1, 1, 1]);
        let stu = world_to_lattice_params(&cage, [1.0, 1.0, 1.0]);
        for &v in &stu {
            assert!((v - 0.5).abs() < 1e-5);
        }
    }
}
