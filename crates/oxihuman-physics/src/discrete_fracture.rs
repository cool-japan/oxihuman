// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Discrete fracture network (DFN) model for flow simulation.

use std::f32::consts::PI;

/// A single fracture in the network (2D line segment).
#[derive(Debug, Clone)]
pub struct Fracture {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub aperture: f32,
    pub id: usize,
}

impl Fracture {
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32, aperture: f32, id: usize) -> Self {
        Fracture {
            x0,
            y0,
            x1,
            y1,
            aperture,
            id,
        }
    }

    pub fn length(&self) -> f32 {
        let dx = self.x1 - self.x0;
        let dy = self.y1 - self.y0;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn orientation(&self) -> f32 {
        (self.y1 - self.y0).atan2(self.x1 - self.x0)
    }

    /// Cubic law transmissivity: T = aperture^3 / 12.
    pub fn transmissivity(&self) -> f32 {
        self.aperture.powi(3) / 12.0
    }

    /// Check intersection with another fracture.
    pub fn intersects(&self, other: &Fracture) -> bool {
        /* Line segment intersection test */
        let (ax, ay) = (self.x0, self.y0);
        let (bx, by) = (self.x1, self.y1);
        let (cx, cy) = (other.x0, other.y0);
        let (dx, dy) = (other.x1, other.y1);
        let denom = (bx - ax) * (dy - cy) - (by - ay) * (dx - cx);
        if denom.abs() < 1e-12 {
            return false;
        }
        let t = ((cx - ax) * (dy - cy) - (cy - ay) * (dx - cx)) / denom;
        let s = ((cx - ax) * (by - ay) - (cy - ay) * (bx - ax)) / denom;
        (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&s)
    }
}

/// Discrete fracture network.
pub struct DiscreteFractureNetwork {
    pub fractures: Vec<Fracture>,
    pub domain: (f32, f32), /* width, height */
}

impl DiscreteFractureNetwork {
    pub fn new(width: f32, height: f32) -> Self {
        DiscreteFractureNetwork {
            fractures: Vec::new(),
            domain: (width, height),
        }
    }

    pub fn add_fracture(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, aperture: f32) -> usize {
        let id = self.fractures.len();
        self.fractures
            .push(Fracture::new(x0, y0, x1, y1, aperture, id));
        id
    }

    /// Count intersections between all fracture pairs.
    pub fn intersection_count(&self) -> usize {
        let n = self.fractures.len();
        let mut count = 0;
        for i in 0..n {
            for j in i + 1..n {
                if self.fractures[i].intersects(&self.fractures[j]) {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn fracture_count(&self) -> usize {
        self.fractures.len()
    }

    pub fn total_length(&self) -> f32 {
        self.fractures.iter().map(|f| f.length()).sum()
    }

    pub fn mean_aperture(&self) -> f32 {
        if self.fractures.is_empty() {
            return 0.0;
        }
        self.fractures.iter().map(|f| f.aperture).sum::<f32>() / self.fractures.len() as f32
    }

    pub fn mean_transmissivity(&self) -> f32 {
        if self.fractures.is_empty() {
            return 0.0;
        }
        self.fractures
            .iter()
            .map(|f| f.transmissivity())
            .sum::<f32>()
            / self.fractures.len() as f32
    }

    /// Generate a random (deterministic) DFN with n fractures.
    pub fn generate(&mut self, n: usize, mean_length: f32, mean_aperture: f32) {
        let w = self.domain.0;
        let h = self.domain.1;
        for i in 0..n {
            let seed = i as u64;
            let cx = pseudo_rand(seed ^ 0xA5A5A5A5) * w;
            let cy = pseudo_rand(seed ^ 0x5A5A5A5A) * h;
            let angle = pseudo_rand(seed ^ 0xDEADBEEF) * PI;
            let half_len = mean_length * 0.5;
            let aperture = mean_aperture * (0.5 + pseudo_rand(seed ^ 0x12345678));
            let x0 = cx - half_len * angle.cos();
            let y0 = cy - half_len * angle.sin();
            let x1 = cx + half_len * angle.cos();
            let y1 = cy + half_len * angle.sin();
            self.add_fracture(x0, y0, x1, y1, aperture);
        }
    }

    /// Fracture intensity P21 = total_length / domain_area.
    pub fn p21_intensity(&self) -> f32 {
        let area = self.domain.0 * self.domain.1;
        if area < 1e-12 {
            return 0.0;
        }
        self.total_length() / area
    }
}

fn pseudo_rand(seed: u64) -> f32 {
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    (x & 0xFFFFFF) as f32 / 0xFFFFFF as f32
}

/// Cubic law flow rate: Q = T * L * (dP / L) = T * dP
pub fn cubic_law_flow(aperture: f32, dp: f32, length: f32) -> f32 {
    let t = aperture.powi(3) / 12.0;
    t * dp / length
}

pub fn new_dfn(width: f32, height: f32) -> DiscreteFractureNetwork {
    DiscreteFractureNetwork::new(width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fracture_length() {
        let f = Fracture::new(0.0, 0.0, 3.0, 4.0, 0.001, 0);
        assert!((f.length() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_transmissivity() {
        let f = Fracture::new(0.0, 0.0, 1.0, 0.0, 0.01, 0);
        let expected = 0.01f32.powi(3) / 12.0;
        assert!((f.transmissivity() - expected).abs() < 1e-12);
    }

    #[test]
    fn test_intersection_detected() {
        let mut dfn = new_dfn(10.0, 10.0);
        dfn.add_fracture(0.0, 5.0, 10.0, 5.0, 0.001); /* horizontal */
        dfn.add_fracture(5.0, 0.0, 5.0, 10.0, 0.001); /* vertical */
        assert_eq!(dfn.intersection_count(), 1);
    }

    #[test]
    fn test_no_intersection_parallel() {
        let mut dfn = new_dfn(10.0, 10.0);
        dfn.add_fracture(0.0, 3.0, 10.0, 3.0, 0.001);
        dfn.add_fracture(0.0, 7.0, 10.0, 7.0, 0.001);
        assert_eq!(dfn.intersection_count(), 0);
    }

    #[test]
    fn test_generate_dfn() {
        let mut dfn = new_dfn(100.0, 100.0);
        dfn.generate(10, 10.0, 0.001);
        assert_eq!(dfn.fracture_count(), 10);
    }

    #[test]
    fn test_p21_intensity() {
        let mut dfn = new_dfn(10.0, 10.0);
        dfn.add_fracture(0.0, 5.0, 10.0, 5.0, 0.001); /* length = 10 */
        let p21 = dfn.p21_intensity();
        assert!((p21 - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_cubic_law_flow() {
        let q = cubic_law_flow(0.001, 1.0, 1.0);
        assert!(q > 0.0);
    }

    #[test]
    fn test_mean_aperture() {
        let mut dfn = new_dfn(10.0, 10.0);
        dfn.add_fracture(0.0, 0.0, 1.0, 0.0, 0.001);
        dfn.add_fracture(0.0, 1.0, 1.0, 1.0, 0.003);
        assert!((dfn.mean_aperture() - 0.002).abs() < 1e-6);
    }
}
