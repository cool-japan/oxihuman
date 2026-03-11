// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Distance-field based collision: sample SDF and compute contact data.

/// Result of an SDF contact query.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct SdfContact {
    pub signed_distance: f32,
    pub normal: [f32; 3],
    pub contact_point: [f32; 3],
}

/// A uniform-grid signed distance field.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DistanceField {
    pub data: Vec<f32>,
    pub resolution: [usize; 3],
    pub origin: [f32; 3],
    pub cell_size: f32,
}

#[allow(dead_code)]
impl DistanceField {
    pub fn new(resolution: [usize; 3], origin: [f32; 3], cell_size: f32) -> Self {
        let n = resolution[0] * resolution[1] * resolution[2];
        Self {
            data: vec![f32::INFINITY; n],
            resolution,
            origin,
            cell_size,
        }
    }

    fn idx(&self, ix: usize, iy: usize, iz: usize) -> usize {
        ix * self.resolution[1] * self.resolution[2] + iy * self.resolution[2] + iz
    }

    pub fn set(&mut self, ix: usize, iy: usize, iz: usize, val: f32) {
        let i = self.idx(ix, iy, iz);
        self.data[i] = val;
    }

    pub fn get_grid(&self, ix: usize, iy: usize, iz: usize) -> f32 {
        self.data[self.idx(ix, iy, iz)]
    }

    /// Trilinear interpolation at world position.
    pub fn sample(&self, pos: [f32; 3]) -> f32 {
        let fx = (pos[0] - self.origin[0]) / self.cell_size;
        let fy = (pos[1] - self.origin[1]) / self.cell_size;
        let fz = (pos[2] - self.origin[2]) / self.cell_size;

        let ix = fx.floor() as isize;
        let iy = fy.floor() as isize;
        let iz = fz.floor() as isize;

        let tx = fx - fx.floor();
        let ty = fy - fy.floor();
        let tz = fz - fz.floor();

        let rx = self.resolution[0] as isize;
        let ry = self.resolution[1] as isize;
        let rz = self.resolution[2] as isize;

        let get = |dx: isize, dy: isize, dz: isize| -> f32 {
            let x = (ix + dx).clamp(0, rx - 1) as usize;
            let y = (iy + dy).clamp(0, ry - 1) as usize;
            let z = (iz + dz).clamp(0, rz - 1) as usize;
            self.get_grid(x, y, z)
        };

        let c000 = get(0, 0, 0);
        let c100 = get(1, 0, 0);
        let c010 = get(0, 1, 0);
        let c110 = get(1, 1, 0);
        let c001 = get(0, 0, 1);
        let c101 = get(1, 0, 1);
        let c011 = get(0, 1, 1);
        let c111 = get(1, 1, 1);

        let c00 = c000 * (1.0 - tx) + c100 * tx;
        let c10 = c010 * (1.0 - tx) + c110 * tx;
        let c01 = c001 * (1.0 - tx) + c101 * tx;
        let c11 = c011 * (1.0 - tx) + c111 * tx;

        let c0 = c00 * (1.0 - ty) + c10 * ty;
        let c1 = c01 * (1.0 - ty) + c11 * ty;

        c0 * (1.0 - tz) + c1 * tz
    }

    /// Numerical gradient (approximate normal) at world position.
    pub fn gradient(&self, pos: [f32; 3]) -> [f32; 3] {
        let h = self.cell_size * 0.5;
        let gx =
            self.sample([pos[0] + h, pos[1], pos[2]]) - self.sample([pos[0] - h, pos[1], pos[2]]);
        let gy =
            self.sample([pos[0], pos[1] + h, pos[2]]) - self.sample([pos[0], pos[1] - h, pos[2]]);
        let gz =
            self.sample([pos[0], pos[1], pos[2] + h]) - self.sample([pos[0], pos[1], pos[2] - h]);
        let len = (gx * gx + gy * gy + gz * gz).sqrt();
        if len > 1e-9 {
            [gx / len, gy / len, gz / len]
        } else {
            [0.0, 1.0, 0.0]
        }
    }
}

/// SDF of a sphere centered at origin with given radius.
#[allow(dead_code)]
pub fn sphere_sdf(pos: [f32; 3], radius: f32) -> f32 {
    let len = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt();
    len - radius
}

/// SDF of an axis-aligned box [0, size] centered at origin.
#[allow(dead_code)]
pub fn box_sdf(pos: [f32; 3], half_extents: [f32; 3]) -> f32 {
    let qx = pos[0].abs() - half_extents[0];
    let qy = pos[1].abs() - half_extents[1];
    let qz = pos[2].abs() - half_extents[2];
    let outside =
        (qx.max(0.0) * qx.max(0.0) + qy.max(0.0) * qy.max(0.0) + qz.max(0.0) * qz.max(0.0)).sqrt();
    outside + qx.max(qy).max(qz).min(0.0)
}

/// Bake a sphere SDF into a DistanceField grid.
#[allow(dead_code)]
pub fn bake_sphere_sdf(df: &mut DistanceField, center: [f32; 3], radius: f32) {
    let [rx, ry, rz] = df.resolution;
    for ix in 0..rx {
        for iy in 0..ry {
            for iz in 0..rz {
                let px = df.origin[0] + (ix as f32 + 0.5) * df.cell_size - center[0];
                let py = df.origin[1] + (iy as f32 + 0.5) * df.cell_size - center[1];
                let pz = df.origin[2] + (iz as f32 + 0.5) * df.cell_size - center[2];
                let d = sphere_sdf([px, py, pz], radius);
                df.set(ix, iy, iz, d);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn sphere_sdf_surface_zero() {
        let d = sphere_sdf([1.0, 0.0, 0.0], 1.0);
        assert!(d.abs() < 1e-5);
    }

    #[test]
    fn sphere_sdf_inside_negative() {
        let d = sphere_sdf([0.0, 0.0, 0.0], 1.0);
        assert!(d < 0.0);
    }

    #[test]
    fn sphere_sdf_outside_positive() {
        let d = sphere_sdf([2.0, 0.0, 0.0], 1.0);
        assert!((d - 1.0).abs() < 1e-5);
    }

    #[test]
    fn box_sdf_inside_negative() {
        let d = box_sdf([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(d < 0.0);
    }

    #[test]
    fn box_sdf_outside_positive() {
        let d = box_sdf([2.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(d > 0.0);
    }

    #[test]
    fn distance_field_sample_center() {
        let mut df = DistanceField::new([4, 4, 4], [0.0, 0.0, 0.0], 1.0);
        bake_sphere_sdf(&mut df, [2.0, 2.0, 2.0], 1.0);
        // Sample at sphere center should be ~-1 (inside)
        let v = df.sample([2.0, 2.0, 2.0]);
        assert!(v < 0.0);
    }

    #[test]
    fn distance_field_gradient_unit_length() {
        let mut df = DistanceField::new([8, 8, 8], [0.0, 0.0, 0.0], 0.5);
        bake_sphere_sdf(&mut df, [2.0, 2.0, 2.0], 1.0);
        let g = df.gradient([3.5, 2.0, 2.0]);
        let len = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
        assert!((len - 1.0).abs() < 0.1);
    }

    #[test]
    fn bake_sphere_fills_grid() {
        let mut df = DistanceField::new([4, 4, 4], [0.0, 0.0, 0.0], 1.0);
        bake_sphere_sdf(&mut df, [2.0, 2.0, 2.0], 1.0);
        // Not all values should be infinity anymore
        assert!(df.data.iter().any(|&v| v != f32::INFINITY));
    }

    #[test]
    fn box_sdf_corner_distance() {
        // Corner of unit box at [1,1,1] from [0.5,0.5,0.5]-sized box
        let d = box_sdf([1.0, 0.0, 0.0], [0.5, 0.5, 0.5]);
        assert!((d - 0.5).abs() < 1e-5);
    }

    #[test]
    fn sphere_sdf_uses_pi_constant() {
        // Verify a point at distance PI from origin on a sphere of radius PI
        let d = sphere_sdf([PI, 0.0, 0.0], PI);
        assert!(d.abs() < 1e-4);
    }
}
