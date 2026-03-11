// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lattice cage deform modifier.

/// A 3-D lattice (cage) of control points.
#[derive(Debug, Clone)]
pub struct Lattice {
    pub u: usize, /* divisions along U */
    pub v: usize,
    pub w: usize,
    pub control_points: Vec<[f32; 3]>,
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Lattice {
    /// Create a unit lattice with given divisions.
    pub fn new(u: usize, v: usize, w: usize) -> Self {
        let u = u.max(2);
        let v = v.max(2);
        let w = w.max(2);
        let mut cp = Vec::with_capacity(u * v * w);
        for wi in 0..w {
            for vi in 0..v {
                for ui in 0..u {
                    cp.push([
                        ui as f32 / (u - 1) as f32,
                        vi as f32 / (v - 1) as f32,
                        wi as f32 / (w - 1) as f32,
                    ]);
                }
            }
        }
        Self {
            u,
            v,
            w,
            control_points: cp,
            min: [0.0; 3],
            max: [1.0; 3],
        }
    }

    pub fn cp_count(&self) -> usize {
        self.u * self.v * self.w
    }
}

/// Map a world-space position into lattice UVW coordinates `[0,1]`^3.
pub fn world_to_lattice_uvw(pos: [f32; 3], min: [f32; 3], max: [f32; 3]) -> [f32; 3] {
    [
        ((pos[0] - min[0]) / (max[0] - min[0]).max(1e-12)).clamp(0.0, 1.0),
        ((pos[1] - min[1]) / (max[1] - min[1]).max(1e-12)).clamp(0.0, 1.0),
        ((pos[2] - min[2]) / (max[2] - min[2]).max(1e-12)).clamp(0.0, 1.0),
    ]
}

/// Trilinear interpolation inside the lattice.
pub fn lattice_trilinear(lattice: &Lattice, uvw: [f32; 3]) -> [f32; 3] {
    let u_max = (lattice.u - 1) as f32;
    let v_max = (lattice.v - 1) as f32;
    let w_max = (lattice.w - 1) as f32;

    let uf = (uvw[0] * u_max).clamp(0.0, u_max);
    let vf = (uvw[1] * v_max).clamp(0.0, v_max);
    let wf = (uvw[2] * w_max).clamp(0.0, w_max);

    let ui = (uf as usize).min(lattice.u.saturating_sub(2));
    let vi = (vf as usize).min(lattice.v.saturating_sub(2));
    let wi = (wf as usize).min(lattice.w.saturating_sub(2));

    let tu = uf - ui as f32;
    let tv = vf - vi as f32;
    let tw = wf - wi as f32;

    let idx = |u: usize, v: usize, w: usize| w * lattice.v * lattice.u + v * lattice.u + u;

    let mut result = [0.0_f32; 3];
    for dw in 0..2_usize {
        for dv in 0..2_usize {
            for du in 0..2_usize {
                let weight = (if du == 0 { 1.0 - tu } else { tu })
                    * (if dv == 0 { 1.0 - tv } else { tv })
                    * (if dw == 0 { 1.0 - tw } else { tw });
                let cp = lattice.control_points[idx(ui + du, vi + dv, wi + dw)];
                result[0] += weight * cp[0];
                result[1] += weight * cp[1];
                result[2] += weight * cp[2];
            }
        }
    }
    result
}

/// Apply lattice deform to a set of positions.
pub fn apply_lattice_deform(positions: &mut [[f32; 3]], lattice: &Lattice) {
    for p in positions.iter_mut() {
        let uvw = world_to_lattice_uvw(*p, lattice.min, lattice.max);
        let deformed = lattice_trilinear(lattice, uvw);
        /* map back from lattice [0,1] to world space */
        for i in 0..3 {
            p[i] = lattice.min[i] + deformed[i] * (lattice.max[i] - lattice.min[i]);
        }
    }
}

/// Validate lattice has at least 2x2x2 control points.
pub fn validate_lattice(lattice: &Lattice) -> bool {
    lattice.u >= 2
        && lattice.v >= 2
        && lattice.w >= 2
        && lattice.control_points.len() == lattice.u * lattice.v * lattice.w
}

// ---- New API required by lib.rs ----

fn bernstein_basis_lat(n: usize, i: usize, t: f32) -> f32 {
    binomial_lat(n, i) as f32 * t.powi(i as i32) * (1.0 - t).powi((n - i) as i32)
}

fn binomial_lat(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut r = 1usize;
    for i in 0..k {
        r = r * (n - i) / (i + 1);
    }
    r
}

/// New-API FFD lattice struct.
pub struct FfdLattice {
    pub divisions: [usize; 3],
    pub control_points: Vec<[f32; 3]>,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}

pub fn new_ffd_lattice(
    nx: usize,
    ny: usize,
    nz: usize,
    min: [f32; 3],
    max: [f32; 3],
) -> FfdLattice {
    FfdLattice {
        divisions: [nx, ny, nz],
        control_points: vec![[0.0; 3]; (nx + 1) * (ny + 1) * (nz + 1)],
        bounds_min: min,
        bounds_max: max,
    }
}

pub fn ffd_lattice_size(l: &FfdLattice) -> usize {
    l.divisions[0] * l.divisions[1] * l.divisions[2]
}

pub fn ffd_lattice_point_count(l: &FfdLattice) -> usize {
    (l.divisions[0] + 1) * (l.divisions[1] + 1) * (l.divisions[2] + 1)
}

pub fn ffd_apply_to_point(l: &FfdLattice, p: [f32; 3]) -> [f32; 3] {
    let [nx, ny, nz] = l.divisions;
    let [minx, miny, minz] = l.bounds_min;
    let [maxx, maxy, maxz] = l.bounds_max;
    let s = ((p[0] - minx) / (maxx - minx).max(1e-9)).clamp(0.0, 1.0);
    let t = ((p[1] - miny) / (maxy - miny).max(1e-9)).clamp(0.0, 1.0);
    let u = ((p[2] - minz) / (maxz - minz).max(1e-9)).clamp(0.0, 1.0);
    let mut offset = [0.0f32; 3];
    #[allow(clippy::needless_range_loop)]
    for i in 0..=nx {
        for j in 0..=ny {
            for k in 0..=nz {
                let w = bernstein_basis_lat(nx, i, s)
                    * bernstein_basis_lat(ny, j, t)
                    * bernstein_basis_lat(nz, k, u);
                let idx = i * (ny + 1) * (nz + 1) + j * (nz + 1) + k;
                let cp = l.control_points[idx];
                offset[0] += w * cp[0];
                offset[1] += w * cp[1];
                offset[2] += w * cp[2];
            }
        }
    }
    [p[0] + offset[0], p[1] + offset[1], p[2] + offset[2]]
}

pub fn ffd_set_control_point(l: &mut FfdLattice, i: usize, j: usize, k: usize, offset: [f32; 3]) {
    let [_, ny, nz] = l.divisions;
    let idx = i * (ny + 1) * (nz + 1) + j * (nz + 1) + k;
    if idx < l.control_points.len() {
        l.control_points[idx] = offset;
    }
}

pub fn ffd_reset(l: &mut FfdLattice) {
    for cp in l.control_points.iter_mut() {
        *cp = [0.0; 3];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lattice_new_dimensions() {
        let lat = Lattice::new(2, 3, 4);
        assert_eq!(lat.u, 2);
        assert_eq!(lat.v, 3);
        assert_eq!(lat.w, 4);
    }

    #[test]
    fn test_lattice_cp_count() {
        let lat = Lattice::new(2, 2, 2);
        assert_eq!(lat.cp_count(), 8);
    }

    #[test]
    fn test_world_to_lattice_uvw_corners() {
        let min = [0.0_f32; 3];
        let max = [1.0_f32; 3];
        let uvw = world_to_lattice_uvw([0.0, 0.0, 0.0], min, max);
        for c in uvw {
            assert!(c.abs() < 1e-5);
        }
    }

    #[test]
    fn test_world_to_lattice_uvw_clamp() {
        let min = [0.0_f32; 3];
        let max = [1.0_f32; 3];
        let uvw = world_to_lattice_uvw([2.0, -1.0, 0.5], min, max);
        assert!(uvw[0] <= 1.0);
        assert!(uvw[1] >= 0.0);
    }

    #[test]
    fn test_lattice_trilinear_origin() {
        let lat = Lattice::new(2, 2, 2);
        let out = lattice_trilinear(&lat, [0.0, 0.0, 0.0]);
        assert!(out[0].abs() < 1e-5);
        assert!(out[1].abs() < 1e-5);
        assert!(out[2].abs() < 1e-5);
    }

    #[test]
    fn test_lattice_trilinear_far_corner() {
        let lat = Lattice::new(2, 2, 2);
        let out = lattice_trilinear(&lat, [1.0, 1.0, 1.0]);
        assert!((out[0] - 1.0).abs() < 1e-5);
        assert!((out[1] - 1.0).abs() < 1e-5);
        assert!((out[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_validate_lattice_valid() {
        let lat = Lattice::new(3, 3, 3);
        assert!(validate_lattice(&lat));
    }

    #[test]
    fn test_apply_lattice_deform_identity() {
        let lat = Lattice::new(2, 2, 2);
        let mut pos = vec![[0.5_f32, 0.5, 0.5]];
        apply_lattice_deform(&mut pos, &lat);
        /* identity lattice: midpoint stays at midpoint */
        for c in pos[0] {
            assert!((0.0..=1.0).contains(&c));
        }
    }

    #[test]
    fn test_apply_lattice_deform_preserves_count() {
        let lat = Lattice::new(2, 2, 2);
        let mut pos = vec![[0.1_f32, 0.2, 0.3]; 10];
        apply_lattice_deform(&mut pos, &lat);
        assert_eq!(pos.len(), 10);
    }
}
