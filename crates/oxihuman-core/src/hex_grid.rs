// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Hexagonal grid coordinate system (axial/cube coordinates).

#![allow(dead_code)]

/// Axial hex coordinate.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

#[allow(dead_code)]
impl HexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Convert axial to cube coordinates (s = -q - r).
    pub fn to_cube(self) -> [i32; 3] {
        [self.q, self.r, -self.q - self.r]
    }

    /// Hex distance between two axial coordinates.
    pub fn distance(self, other: HexCoord) -> i32 {
        let [aq, ar, as_] = self.to_cube();
        let [bq, br, bs] = other.to_cube();
        ((aq - bq).abs() + (ar - br).abs() + (as_ - bs).abs()) / 2
    }

    /// Return the 6 axial neighbors.
    pub fn neighbors(self) -> [HexCoord; 6] {
        const DIRS: [(i32, i32); 6] = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];
        DIRS.map(|(dq, dr)| HexCoord::new(self.q + dq, self.r + dr))
    }

    /// Convert to flat-top pixel coordinates (size = hex radius).
    pub fn to_pixel_flat(self, size: f32) -> [f32; 2] {
        let x = size * (3.0 / 2.0 * self.q as f32);
        let y = size * (3.0f32.sqrt() / 2.0 * self.q as f32 + 3.0f32.sqrt() * self.r as f32);
        [x, y]
    }

    /// Convert to pointy-top pixel coordinates.
    pub fn to_pixel_pointy(self, size: f32) -> [f32; 2] {
        let x = size * (3.0f32.sqrt() * self.q as f32 + 3.0f32.sqrt() / 2.0 * self.r as f32);
        let y = size * (3.0 / 2.0 * self.r as f32);
        [x, y]
    }
}

/// Round fractional cube coordinates to nearest integer hex.
#[allow(dead_code)]
pub fn cube_round(fq: f32, fr: f32, fs: f32) -> HexCoord {
    let mut rq = fq.round();
    let mut rr = fr.round();
    let rs = fs.round();
    let dq = (rq - fq).abs();
    let dr = (rr - fr).abs();
    let ds = (rs - fs).abs();
    if dq > dr && dq > ds {
        rq = -rr - rs;
    } else if dr > ds {
        rr = -rq - rs;
    }
    HexCoord::new(rq as i32, rr as i32)
}

/// Convert pixel (flat-top) to fractional hex coord and round.
#[allow(dead_code)]
pub fn pixel_to_hex_flat(x: f32, y: f32, size: f32) -> HexCoord {
    let fq = (2.0 / 3.0 * x) / size;
    let fr = (-1.0 / 3.0 * x + 3.0f32.sqrt() / 3.0 * y) / size;
    cube_round(fq, fr, -fq - fr)
}

/// Generate a filled hexagonal ring at radius `r` from origin.
#[allow(dead_code)]
pub fn hex_ring(center: HexCoord, radius: i32) -> Vec<HexCoord> {
    if radius == 0 {
        return vec![center];
    }
    let mut results = Vec::with_capacity(6 * radius as usize);
    let dirs: [(i32, i32); 6] = [(1, -1), (1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1)];
    let mut h = HexCoord::new(center.q + dirs[4].0 * radius, center.r + dirs[4].1 * radius);
    for (dq, dr) in dirs {
        for _ in 0..radius {
            results.push(h);
            h = HexCoord::new(h.q + dq, h.r + dr);
        }
    }
    results
}

/// Generate all hexes within `radius` from center.
#[allow(dead_code)]
pub fn hex_disk(center: HexCoord, radius: i32) -> Vec<HexCoord> {
    let mut results = Vec::new();
    for q in -radius..=radius {
        let r1 = (-radius).max(-q - radius);
        let r2 = radius.min(-q + radius);
        for r in r1..=r2 {
            results.push(HexCoord::new(center.q + q, center.r + r));
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_self_is_zero() {
        let h = HexCoord::new(2, 3);
        assert_eq!(h.distance(h), 0);
    }

    #[test]
    fn distance_to_neighbor_is_one() {
        let h = HexCoord::new(0, 0);
        for n in h.neighbors() {
            assert_eq!(h.distance(n), 1);
        }
    }

    #[test]
    fn six_neighbors() {
        let neighbors = HexCoord::new(0, 0).neighbors();
        assert_eq!(neighbors.len(), 6);
    }

    #[test]
    fn cube_coords_sum_zero() {
        let h = HexCoord::new(3, -5);
        let [q, r, s] = h.to_cube();
        assert_eq!(q + r + s, 0);
    }

    #[test]
    fn ring_size() {
        let ring = hex_ring(HexCoord::new(0, 0), 2);
        assert_eq!(ring.len(), 12);
    }

    #[test]
    fn disk_includes_center() {
        let disk = hex_disk(HexCoord::new(1, 1), 1);
        assert!(disk.contains(&HexCoord::new(1, 1)));
    }

    #[test]
    fn disk_radius_zero_is_just_center() {
        let disk = hex_disk(HexCoord::new(0, 0), 0);
        assert_eq!(disk.len(), 1);
    }

    #[test]
    fn cube_round_origin() {
        let h = cube_round(0.1, -0.1, 0.0);
        assert_eq!(h, HexCoord::new(0, 0));
    }

    #[test]
    fn pixel_to_hex_round_trip() {
        let origin = HexCoord::new(2, -1);
        let [px, py] = origin.to_pixel_flat(1.0);
        let recovered = pixel_to_hex_flat(px, py, 1.0);
        assert_eq!(recovered, origin);
    }

    #[test]
    fn disk_radius_one_has_seven_cells() {
        let disk = hex_disk(HexCoord::new(0, 0), 1);
        assert_eq!(disk.len(), 7);
    }
}
