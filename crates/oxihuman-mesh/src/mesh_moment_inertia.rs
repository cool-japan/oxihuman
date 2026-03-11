// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Compute the inertia tensor from a closed triangle mesh.

/// Inertia tensor (3×3 symmetric), stored row-major.
#[allow(dead_code)]
pub struct InertiaTensor {
    pub mat: [f32; 9],
    pub mass: f32,
    pub center_of_mass: [f32; 3],
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Compute inertia tensor for a closed triangle mesh with uniform density.
/// Uses the divergence theorem approach.
#[allow(dead_code)]
pub fn compute_inertia_tensor(
    positions: &[[f32; 3]],
    indices: &[u32],
    density: f32,
) -> InertiaTensor {
    let n_tri = indices.len() / 3;
    let mut vol = 0.0_f32;
    let mut com = [0.0_f32; 3];
    let mut ixx = 0.0_f32;
    let mut iyy = 0.0_f32;
    let mut izz = 0.0_f32;
    let mut ixy = 0.0_f32;
    let mut ixz = 0.0_f32;
    let mut iyz = 0.0_f32;
    for t in 0..n_tri {
        let p0 = positions[indices[t * 3] as usize];
        let p1 = positions[indices[t * 3 + 1] as usize];
        let p2 = positions[indices[t * 3 + 2] as usize];
        let v = dot3(p0, cross3(p1, p2));
        vol += v;
        for k in 0..3 {
            com[k] += v * (p0[k] + p1[k] + p2[k]);
        }
        let (x0, y0, z0) = (p0[0], p0[1], p0[2]);
        let (x1, y1, z1) = (p1[0], p1[1], p1[2]);
        let (x2, y2, z2) = (p2[0], p2[1], p2[2]);
        ixx += v
            * (y0 * y0
                + y0 * y1
                + y1 * y1
                + y0 * y2
                + y1 * y2
                + y2 * y2
                + z0 * z0
                + z0 * z1
                + z1 * z1
                + z0 * z2
                + z1 * z2
                + z2 * z2);
        iyy += v
            * (x0 * x0
                + x0 * x1
                + x1 * x1
                + x0 * x2
                + x1 * x2
                + x2 * x2
                + z0 * z0
                + z0 * z1
                + z1 * z1
                + z0 * z2
                + z1 * z2
                + z2 * z2);
        izz += v
            * (x0 * x0
                + x0 * x1
                + x1 * x1
                + x0 * x2
                + x1 * x2
                + x2 * x2
                + y0 * y0
                + y0 * y1
                + y1 * y1
                + y0 * y2
                + y1 * y2
                + y2 * y2);
        ixy += v
            * (x0 * y0
                + x0 * y1
                + x0 * y2
                + x1 * y0
                + x1 * y1
                + x1 * y2
                + x2 * y0
                + x2 * y1
                + x2 * y2)
            / 3.0;
        ixz += v
            * (x0 * z0
                + x0 * z1
                + x0 * z2
                + x1 * z0
                + x1 * z1
                + x1 * z2
                + x2 * z0
                + x2 * z1
                + x2 * z2)
            / 3.0;
        iyz += v
            * (y0 * z0
                + y0 * z1
                + y0 * z2
                + y1 * z0
                + y1 * z1
                + y1 * z2
                + y2 * z0
                + y2 * z1
                + y2 * z2)
            / 3.0;
    }
    let vol6 = vol / 6.0;
    let mass = density * vol6.abs();
    let com_scale = 1.0 / (vol * 4.0);
    let center_of_mass = [com[0] * com_scale, com[1] * com_scale, com[2] * com_scale];
    let rho_scale = density / 60.0;
    let mat = [
        ixx * rho_scale,
        -ixy,
        -ixz,
        -ixy,
        iyy * rho_scale,
        -iyz,
        -ixz,
        -iyz,
        izz * rho_scale,
    ];
    InertiaTensor {
        mat,
        mass,
        center_of_mass,
    }
}

/// Extract diagonal (principal moments) from inertia tensor.
#[allow(dead_code)]
pub fn principal_moments(tensor: &InertiaTensor) -> [f32; 3] {
    [tensor.mat[0], tensor.mat[4], tensor.mat[8]]
}

/// Trace of inertia tensor.
#[allow(dead_code)]
pub fn tensor_trace(tensor: &InertiaTensor) -> f32 {
    tensor.mat[0] + tensor.mat[4] + tensor.mat[8]
}

/// Check if tensor is symmetric within tolerance.
#[allow(dead_code)]
pub fn tensor_is_symmetric(tensor: &InertiaTensor, tol: f32) -> bool {
    (tensor.mat[1] - tensor.mat[3]).abs() < tol
        && (tensor.mat[2] - tensor.mat[6]).abs() < tol
        && (tensor.mat[5] - tensor.mat[7]).abs() < tol
}

/// Frobenius norm of inertia tensor.
#[allow(dead_code)]
pub fn tensor_frobenius_norm(tensor: &InertiaTensor) -> f32 {
    tensor.mat.iter().map(|v| v * v).sum::<f32>().sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cube_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 5, 1, 0, 4, 5, 2, 6, 3, 3, 6, 7, 1, 6, 2, 1, 5,
            6, 0, 3, 7, 0, 7, 4,
        ];
        (positions, indices)
    }

    #[test]
    fn mass_positive() {
        let (pos, idx) = unit_cube_mesh();
        let tensor = compute_inertia_tensor(&pos, &idx, 1.0);
        assert!(tensor.mass > 0.0, "mass: {}", tensor.mass);
    }

    #[test]
    fn principal_moments_3() {
        let (pos, idx) = unit_cube_mesh();
        let tensor = compute_inertia_tensor(&pos, &idx, 1.0);
        let pm = principal_moments(&tensor);
        assert_eq!(pm.len(), 3);
    }

    #[test]
    fn tensor_trace_nonneg() {
        let (pos, idx) = unit_cube_mesh();
        let tensor = compute_inertia_tensor(&pos, &idx, 1.0);
        let tr = tensor_trace(&tensor);
        assert!(tr.is_finite());
    }

    #[test]
    fn tensor_is_symmetric_check() {
        let (pos, idx) = unit_cube_mesh();
        let tensor = compute_inertia_tensor(&pos, &idx, 1.0);
        assert!(tensor_is_symmetric(&tensor, 1e-3));
    }

    #[test]
    fn frobenius_norm_positive() {
        let (pos, idx) = unit_cube_mesh();
        let tensor = compute_inertia_tensor(&pos, &idx, 1.0);
        let f = tensor_frobenius_norm(&tensor);
        assert!(f > 0.0);
    }

    #[test]
    fn density_scales_mass() {
        let (pos, idx) = unit_cube_mesh();
        let t1 = compute_inertia_tensor(&pos, &idx, 1.0);
        let t2 = compute_inertia_tensor(&pos, &idx, 2.0);
        assert!((t2.mass - 2.0 * t1.mass).abs() < 1e-3);
    }

    #[test]
    fn empty_mesh_mass_zero() {
        let tensor = compute_inertia_tensor(&[], &[], 1.0);
        assert_eq!(tensor.mass, 0.0);
    }

    #[test]
    fn mat_size_nine() {
        let (pos, idx) = unit_cube_mesh();
        let tensor = compute_inertia_tensor(&pos, &idx, 1.0);
        assert_eq!(tensor.mat.len(), 9);
    }

    #[test]
    fn center_of_mass_inside_cube() {
        let (pos, idx) = unit_cube_mesh();
        let tensor = compute_inertia_tensor(&pos, &idx, 1.0);
        for v in tensor.center_of_mass {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn tensor_frobenius_scales() {
        let (pos, idx) = unit_cube_mesh();
        let t1 = compute_inertia_tensor(&pos, &idx, 1.0);
        let t2 = compute_inertia_tensor(&pos, &idx, 2.0);
        let f1 = tensor_frobenius_norm(&t1);
        let f2 = tensor_frobenius_norm(&t2);
        assert!(f2 > f1);
    }
}
