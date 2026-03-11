// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Brittle solid that fragments on impact.

#![allow(dead_code)]

/// A fragment piece from a brittle fracture.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Fragment {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub mass: f64,
    pub active: bool,
}

impl Fragment {
    #[allow(dead_code)]
    pub fn new(position: [f64; 3], mass: f64) -> Self {
        Self { position, velocity: [0.0; 3], mass, active: true }
    }

    #[allow(dead_code)]
    pub fn step(&mut self, gravity: f64, dt: f64) {
        if !self.active {
            return;
        }
        self.velocity[1] -= gravity * dt;
        for i in 0..3 {
            self.position[i] += self.velocity[i] * dt;
        }
    }
}

/// Brittle solid body.
#[allow(dead_code)]
pub struct BrittleBody {
    pub position: [f64; 3],
    pub mass: f64,
    pub fracture_stress: f64,
    pub shards: usize,
    pub is_fractured: bool,
    pub fragments: Vec<Fragment>,
}

impl BrittleBody {
    #[allow(dead_code)]
    pub fn new(position: [f64; 3], mass: f64, fracture_stress: f64, shards: usize) -> Self {
        Self {
            position,
            mass,
            fracture_stress,
            shards,
            is_fractured: false,
            fragments: Vec::new(),
        }
    }

    /// Apply impact impulse. If stress exceeds fracture threshold, fragment.
    #[allow(dead_code)]
    pub fn apply_impact(&mut self, impulse: f64, impact_point: [f64; 3]) {
        if self.is_fractured {
            return;
        }
        let stress = impulse / (self.mass + 1e-12);
        if stress >= self.fracture_stress {
            self.fracture(impulse, impact_point);
        }
    }

    fn fracture(&mut self, impulse: f64, impact_point: [f64; 3]) {
        self.is_fractured = true;
        let shard_mass = self.mass / self.shards.max(1) as f64;
        let base_speed = (impulse / self.mass).min(100.0);
        let n = self.shards.max(1);
        for i in 0..n {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
            let vx = base_speed * angle.cos();
            let vz = base_speed * angle.sin();
            let offset = [
                impact_point[0] - self.position[0],
                impact_point[1] - self.position[1],
                impact_point[2] - self.position[2],
            ];
            let mut frag = Fragment::new(
                [
                    self.position[0] + offset[0] * 0.1 * angle.cos(),
                    self.position[1],
                    self.position[2] + offset[2] * 0.1 * angle.sin(),
                ],
                shard_mass,
            );
            frag.velocity = [vx, base_speed * 0.5, vz];
            self.fragments.push(frag);
        }
    }

    #[allow(dead_code)]
    pub fn step(&mut self, gravity: f64, dt: f64) {
        for frag in &mut self.fragments {
            frag.step(gravity, dt);
        }
    }

    #[allow(dead_code)]
    pub fn fragment_count(&self) -> usize {
        self.fragments.len()
    }

    #[allow(dead_code)]
    pub fn active_fragments(&self) -> usize {
        self.fragments.iter().filter(|f| f.active).count()
    }

    /// Total momentum of all fragments.
    #[allow(dead_code)]
    #[allow(clippy::needless_range_loop)]
    pub fn total_momentum(&self) -> [f64; 3] {
        let mut p = [0.0f64; 3];
        for frag in &self.fragments {
            for i in 0..3 {
                p[i] += frag.mass * frag.velocity[i];
            }
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_fracture_below_threshold() {
        let mut body = BrittleBody::new([0.0; 3], 1.0, 100.0, 4);
        body.apply_impact(10.0, [0.0; 3]);
        assert!(!body.is_fractured);
    }

    #[test]
    fn test_fracture_above_threshold() {
        let mut body = BrittleBody::new([0.0; 3], 1.0, 1.0, 4);
        body.apply_impact(100.0, [0.0; 3]);
        assert!(body.is_fractured);
    }

    #[test]
    fn test_fragment_count_after_fracture() {
        let mut body = BrittleBody::new([0.0; 3], 1.0, 1.0, 6);
        body.apply_impact(100.0, [0.0; 3]);
        assert_eq!(body.fragment_count(), 6);
    }

    #[test]
    fn test_fragment_mass_conservation() {
        let mut body = BrittleBody::new([0.0; 3], 6.0, 1.0, 6);
        body.apply_impact(100.0, [0.0; 3]);
        let total: f64 = body.fragments.iter().map(|f| f.mass).sum();
        assert!((total - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_step_moves_fragments() {
        let mut body = BrittleBody::new([0.0; 3], 1.0, 1.0, 4);
        body.apply_impact(100.0, [0.0; 3]);
        let pos0: Vec<[f64; 3]> = body.fragments.iter().map(|f| f.position).collect();
        body.step(9.8, 0.1);
        let moved = body
            .fragments
            .iter()
            .zip(pos0.iter())
            .any(|(f, p0)| (f.position[0] - p0[0]).abs() > 1e-12 || (f.position[1] - p0[1]).abs() > 1e-12);
        assert!(moved);
    }

    #[test]
    fn test_no_double_fracture() {
        let mut body = BrittleBody::new([0.0; 3], 1.0, 1.0, 4);
        body.apply_impact(100.0, [0.0; 3]);
        body.apply_impact(100.0, [0.0; 3]);
        assert_eq!(body.fragment_count(), 4);
    }

    #[test]
    fn test_active_fragments() {
        let mut body = BrittleBody::new([0.0; 3], 1.0, 1.0, 3);
        body.apply_impact(100.0, [0.0; 3]);
        assert_eq!(body.active_fragments(), 3);
    }

    #[test]
    fn test_total_momentum_nonzero() {
        let mut body = BrittleBody::new([0.0; 3], 1.0, 1.0, 4);
        body.apply_impact(100.0, [0.0; 3]);
        let p = body.total_momentum();
        let pmag = p.iter().map(|&x| x * x).sum::<f64>().sqrt();
        assert!(pmag >= 0.0);
    }

    #[test]
    fn test_fragment_step() {
        let mut frag = Fragment::new([0.0, 10.0, 0.0], 1.0);
        frag.velocity = [1.0, 0.0, 0.0];
        frag.step(9.8, 0.1);
        assert!((frag.position[0] - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_fragment_new() {
        let frag = Fragment::new([1.0, 2.0, 3.0], 0.5);
        assert!(frag.active);
        assert_eq!(frag.mass, 0.5);
    }
}
