// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Lattice-based free-form deformation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeformLattice {
    pub dims: [usize; 3],
    pub control_points: Vec<[f32; 3]>,
    pub origin: [f32; 3],
    pub size: [f32; 3],
}

#[allow(dead_code)]
impl DeformLattice {
    /// Create a uniform lattice around given bounds.
    pub fn new(origin: [f32; 3], size: [f32; 3], dims: [usize; 3]) -> Self {
        let mut control_points = Vec::new();
        for iz in 0..dims[2] {
            for iy in 0..dims[1] {
                for ix in 0..dims[0] {
                    let x = origin[0] + size[0] * ix as f32 / (dims[0] - 1).max(1) as f32;
                    let y = origin[1] + size[1] * iy as f32 / (dims[1] - 1).max(1) as f32;
                    let z = origin[2] + size[2] * iz as f32 / (dims[2] - 1).max(1) as f32;
                    control_points.push([x, y, z]);
                }
            }
        }
        Self { dims, control_points, origin, size }
    }

    /// Total number of control points.
    pub fn point_count(&self) -> usize {
        self.control_points.len()
    }

    /// Get the index of a control point by (ix, iy, iz).
    pub fn index_of(&self, ix: usize, iy: usize, iz: usize) -> usize {
        iz * self.dims[1] * self.dims[0] + iy * self.dims[0] + ix
    }

    /// Set a control point position.
    pub fn set_point(&mut self, ix: usize, iy: usize, iz: usize, pos: [f32; 3]) {
        let idx = self.index_of(ix, iy, iz);
        self.control_points[idx] = pos;
    }

    /// Get a control point position.
    pub fn get_point(&self, ix: usize, iy: usize, iz: usize) -> [f32; 3] {
        self.control_points[self.index_of(ix, iy, iz)]
    }

    /// Evaluate the deformed position using trilinear interpolation.
    pub fn evaluate(&self, local_uvw: [f32; 3]) -> [f32; 3] {
        let u = local_uvw[0].clamp(0.0, 1.0);
        let v = local_uvw[1].clamp(0.0, 1.0);
        let w = local_uvw[2].clamp(0.0, 1.0);
        let fx = u * (self.dims[0] - 1) as f32;
        let fy = v * (self.dims[1] - 1) as f32;
        let fz = w * (self.dims[2] - 1) as f32;
        let ix = (fx as usize).min(self.dims[0] - 2);
        let iy = (fy as usize).min(self.dims[1] - 2);
        let iz = (fz as usize).min(self.dims[2] - 2);
        let tx = fx - ix as f32;
        let ty = fy - iy as f32;
        let tz = fz - iz as f32;
        let mut result = [0.0f32; 3];
        for dz in 0..2 {
            for dy in 0..2 {
                for dx in 0..2 {
                    let cp = self.get_point(ix + dx, iy + dy, iz + dz);
                    let wx = if dx == 0 { 1.0 - tx } else { tx };
                    let wy = if dy == 0 { 1.0 - ty } else { ty };
                    let wz = if dz == 0 { 1.0 - tz } else { tz };
                    let weight = wx * wy * wz;
                    result[0] += cp[0] * weight;
                    result[1] += cp[1] * weight;
                    result[2] += cp[2] * weight;
                }
            }
        }
        result
    }
}

/// Apply lattice deformation to mesh positions.
#[allow(dead_code)]
pub fn apply_lattice_deform(
    positions: &mut [[f32; 3]],
    lattice: &DeformLattice,
) {
    for pos in positions.iter_mut() {
        let u = if lattice.size[0] > 0.0 { (pos[0] - lattice.origin[0]) / lattice.size[0] } else { 0.0 };
        let v = if lattice.size[1] > 0.0 { (pos[1] - lattice.origin[1]) / lattice.size[1] } else { 0.0 };
        let w = if lattice.size[2] > 0.0 { (pos[2] - lattice.origin[2]) / lattice.size[2] } else { 0.0 };
        *pos = lattice.evaluate([u, v, w]);
    }
}

/// Compute local UVW for a world position.
#[allow(dead_code)]
pub fn world_to_local(lattice: &DeformLattice, pos: &[f32; 3]) -> [f32; 3] {
    let u = if lattice.size[0] > 0.0 { (pos[0] - lattice.origin[0]) / lattice.size[0] } else { 0.0 };
    let v = if lattice.size[1] > 0.0 { (pos[1] - lattice.origin[1]) / lattice.size[1] } else { 0.0 };
    let w = if lattice.size[2] > 0.0 { (pos[2] - lattice.origin[2]) / lattice.size[2] } else { 0.0 };
    [u, v, w]
}

/// Serialize lattice info to JSON.
#[allow(dead_code)]
pub fn lattice_to_json(lattice: &DeformLattice) -> String {
    format!(
        "{{\"dims\":[{},{},{}],\"points\":{}}}",
        lattice.dims[0], lattice.dims[1], lattice.dims[2], lattice.point_count()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_lattice() {
        let l = DeformLattice::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2, 2, 2]);
        assert_eq!(l.point_count(), 8);
    }

    #[test]
    fn test_identity_deform() {
        let l = DeformLattice::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2, 2, 2]);
        let p = l.evaluate([0.5, 0.5, 0.5]);
        assert!((p[0] - 0.5).abs() < 1e-5);
        assert!((p[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_corner_eval() {
        let l = DeformLattice::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2, 2, 2]);
        let p = l.evaluate([0.0, 0.0, 0.0]);
        assert!((p[0]).abs() < 1e-5);
    }

    #[test]
    fn test_set_point() {
        let mut l = DeformLattice::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2, 2, 2]);
        l.set_point(1, 1, 1, [2.0, 2.0, 2.0]);
        let p = l.get_point(1, 1, 1);
        assert!((p[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_deform() {
        let l = DeformLattice::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2, 2, 2]);
        let mut positions = vec![[0.5, 0.5, 0.5]];
        apply_lattice_deform(&mut positions, &l);
        assert!((positions[0][0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_world_to_local() {
        let l = DeformLattice::new([0.0, 0.0, 0.0], [2.0, 2.0, 2.0], [2, 2, 2]);
        let uvw = world_to_local(&l, &[1.0, 1.0, 1.0]);
        assert!((uvw[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_3x3x3_lattice() {
        let l = DeformLattice::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [3, 3, 3]);
        assert_eq!(l.point_count(), 27);
    }

    #[test]
    fn test_index_of() {
        let l = DeformLattice::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [3, 3, 3]);
        assert_eq!(l.index_of(0, 0, 0), 0);
        assert_eq!(l.index_of(2, 2, 2), 26);
    }

    #[test]
    fn test_to_json() {
        let l = DeformLattice::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2, 2, 2]);
        let json = lattice_to_json(&l);
        assert!(json.contains("dims"));
    }

    #[test]
    fn test_clamped_eval() {
        let l = DeformLattice::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2, 2, 2]);
        let p = l.evaluate([2.0, 2.0, 2.0]);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }
}
