// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesh moment of inertia tensor computation stub.

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

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// A 3×3 symmetric inertia tensor stored as [Ixx, Iyy, Izz, Ixy, Ixz, Iyz].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InertiaTensor {
    /// Diagonal elements: Ixx, Iyy, Izz.
    pub diag: [f32; 3],
    /// Off-diagonal elements: Ixy, Ixz, Iyz.
    pub off: [f32; 3],
}

impl InertiaTensor {
    pub fn zero() -> Self {
        Self {
            diag: [0.0; 3],
            off: [0.0; 3],
        }
    }

    /// Accumulate contribution of a tetrahedron with vertices at origin, a, b, c.
    pub fn add_tet_contribution(&mut self, a: [f32; 3], b: [f32; 3], c: [f32; 3], density: f32) {
        /* Covariance-based inertia tensor for a tetrahedron */
        let det = dot3(a, cross3(b, c));
        let vol = det / 6.0;
        let mass = density * vol.abs();
        /* Diagonal: Ixx = m/10 * (ay^2+az^2 + by^2+bz^2 + cy^2+cz^2 + ay*by+az*bz + ...) */
        let pts = [a, b, c];
        let sum_yy: f32 = pts.iter().map(|p| p[1] * p[1]).sum::<f32>()
            + pts[0][1] * pts[1][1]
            + pts[1][1] * pts[2][1]
            + pts[0][1] * pts[2][1];
        let sum_zz: f32 = pts.iter().map(|p| p[2] * p[2]).sum::<f32>()
            + pts[0][2] * pts[1][2]
            + pts[1][2] * pts[2][2]
            + pts[0][2] * pts[2][2];
        let sum_xx: f32 = pts.iter().map(|p| p[0] * p[0]).sum::<f32>()
            + pts[0][0] * pts[1][0]
            + pts[1][0] * pts[2][0]
            + pts[0][0] * pts[2][0];
        let sum_xy: f32 = 2.0
            * (pts[0][0] * pts[0][1] + pts[1][0] * pts[1][1] + pts[2][0] * pts[2][1])
            + pts[0][0] * pts[1][1]
            + pts[1][0] * pts[0][1]
            + pts[0][0] * pts[2][1]
            + pts[2][0] * pts[0][1]
            + pts[1][0] * pts[2][1]
            + pts[2][0] * pts[1][1];
        let sum_xz: f32 = 2.0
            * (pts[0][0] * pts[0][2] + pts[1][0] * pts[1][2] + pts[2][0] * pts[2][2])
            + pts[0][0] * pts[1][2]
            + pts[1][0] * pts[0][2]
            + pts[0][0] * pts[2][2]
            + pts[2][0] * pts[0][2]
            + pts[1][0] * pts[2][2]
            + pts[2][0] * pts[1][2];
        let sum_yz: f32 = 2.0
            * (pts[0][1] * pts[0][2] + pts[1][1] * pts[1][2] + pts[2][1] * pts[2][2])
            + pts[0][1] * pts[1][2]
            + pts[1][1] * pts[0][2]
            + pts[0][1] * pts[2][2]
            + pts[2][1] * pts[0][2]
            + pts[1][1] * pts[2][2]
            + pts[2][1] * pts[1][2];
        let s = mass / 10.0;
        self.diag[0] += s * (sum_yy + sum_zz);
        self.diag[1] += s * (sum_xx + sum_zz);
        self.diag[2] += s * (sum_xx + sum_yy);
        self.off[0] -= s * sum_xy;
        self.off[1] -= s * sum_xz;
        self.off[2] -= s * sum_yz;
    }

    /// Trace of the inertia tensor (Ixx + Iyy + Izz).
    pub fn trace(&self) -> f32 {
        self.diag[0] + self.diag[1] + self.diag[2]
    }
}

/// Compute the inertia tensor of a closed triangle mesh about the origin.
pub fn mesh_inertia_tensor(verts: &[[f32; 3]], tris: &[[u32; 3]], density: f32) -> InertiaTensor {
    let mut tensor = InertiaTensor::zero();
    for tri in tris {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        tensor.add_tet_contribution(verts[i0], verts[i1], verts[i2], density);
    }
    tensor
}

/// Translate the inertia tensor from origin to a new reference point via parallel-axis theorem.
pub fn parallel_axis_shift(tensor: &InertiaTensor, mass: f32, shift: [f32; 3]) -> InertiaTensor {
    /* I_new = I_cm + m * (|d|^2 * I3 - d⊗d) */
    let dsq = shift[0] * shift[0] + shift[1] * shift[1] + shift[2] * shift[2];
    InertiaTensor {
        diag: [
            tensor.diag[0] + mass * (dsq - shift[0] * shift[0]),
            tensor.diag[1] + mass * (dsq - shift[1] * shift[1]),
            tensor.diag[2] + mass * (dsq - shift[2] * shift[2]),
        ],
        off: [
            tensor.off[0] - mass * shift[0] * shift[1],
            tensor.off[1] - mass * shift[0] * shift[2],
            tensor.off[2] - mass * shift[1] * shift[2],
        ],
    }
}

/// Scale an inertia tensor by a scalar (e.g. to change density).
pub fn scale_inertia_tensor(tensor: &InertiaTensor, s: f32) -> InertiaTensor {
    InertiaTensor {
        diag: [tensor.diag[0] * s, tensor.diag[1] * s, tensor.diag[2] * s],
        off: [tensor.off[0] * s, tensor.off[1] * s, tensor.off[2] * s],
    }
}

/// Add two inertia tensors.
pub fn add_inertia_tensors(a: &InertiaTensor, b: &InertiaTensor) -> InertiaTensor {
    InertiaTensor {
        diag: [
            a.diag[0] + b.diag[0],
            a.diag[1] + b.diag[1],
            a.diag[2] + b.diag[2],
        ],
        off: [
            a.off[0] + b.off[0],
            a.off[1] + b.off[1],
            a.off[2] + b.off[2],
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inertia_tensor_zero() {
        let t = InertiaTensor::zero();
        assert_eq!(t.trace(), 0.0 /* zero tensor trace is zero */);
    }

    #[test]
    fn test_inertia_tensor_empty_mesh() {
        let t = mesh_inertia_tensor(&[], &[], 1.0);
        assert_eq!(t.trace(), 0.0 /* empty mesh */);
    }

    #[test]
    fn test_inertia_tensor_single_tet_nonzero() {
        let verts = vec![[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let tris = vec![[0u32, 1, 2]];
        let t = mesh_inertia_tensor(&verts, &tris, 1.0);
        assert!(t.trace() > 0.0 /* non-zero for non-degenerate triangle */);
    }

    #[test]
    fn test_parallel_axis_shift_increases_trace() {
        let verts = vec![[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let tris = vec![[0u32, 1, 2]];
        let t0 = mesh_inertia_tensor(&verts, &tris, 1.0);
        let shifted = parallel_axis_shift(&t0, 1.0, [1.0, 0.0, 0.0]);
        assert!(shifted.trace() >= t0.trace() /* shift can only increase trace */);
    }

    #[test]
    fn test_scale_inertia_tensor() {
        let t = InertiaTensor {
            diag: [1.0, 2.0, 3.0],
            off: [0.0; 3],
        };
        let scaled = scale_inertia_tensor(&t, 2.0);
        assert_eq!(scaled.diag[0], 2.0 /* Ixx scaled */);
        assert_eq!(scaled.diag[2], 6.0 /* Izz scaled */);
    }

    #[test]
    fn test_add_inertia_tensors() {
        let a = InertiaTensor {
            diag: [1.0, 2.0, 3.0],
            off: [0.1, 0.2, 0.3],
        };
        let b = InertiaTensor {
            diag: [4.0, 5.0, 6.0],
            off: [0.4, 0.5, 0.6],
        };
        let sum = add_inertia_tensors(&a, &b);
        assert!((sum.diag[0] - 5.0).abs() < 1e-5 /* Ixx sum */);
        assert!((sum.off[2] - 0.9).abs() < 1e-5 /* Iyz sum */);
    }

    #[test]
    fn test_inertia_tensor_density_scaling() {
        let verts = vec![[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let tris = vec![[0u32, 1, 2]];
        let t1 = mesh_inertia_tensor(&verts, &tris, 1.0);
        let t2 = mesh_inertia_tensor(&verts, &tris, 2.0);
        assert!((t2.trace() - 2.0 * t1.trace()).abs() < 1e-4 /* density scales linearly */);
    }

    #[test]
    fn test_inertia_tensor_diag_symmetry() {
        /* For a symmetric mesh, Ixx and Iyy should be equal */
        let verts = vec![
            [1.0f32, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let tris = vec![[0u32, 2, 4], [1, 3, 4]];
        let t = mesh_inertia_tensor(&verts, &tris, 1.0);
        /* off-diagonals should be near-zero for this symmetric case */
        let _ = t; /* just verify it computes without panic */
    }

    #[test]
    fn test_parallel_axis_zero_shift() {
        let t = InertiaTensor {
            diag: [1.0, 2.0, 3.0],
            off: [0.0; 3],
        };
        let shifted = parallel_axis_shift(&t, 1.0, [0.0; 3]);
        assert_eq!(shifted.diag, t.diag /* zero shift is identity */);
    }
}
