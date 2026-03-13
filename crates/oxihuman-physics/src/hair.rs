// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Hair strand simulation using Verlet integration with distance constraints.
//!
//! Each [`HairStrand`] is a chain of particles from root (index 0, pinned) to tip.
//! [`HairSystem`] advances all strands in time using gravity, damping, distance
//! constraints, and a stiffness spring back to the rest pose.

// ── Private vector math helpers ───────────────────────────────────────────────

#[inline]
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn length(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

#[inline]
fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

// ── HairStrand ────────────────────────────────────────────────────────────────

/// A single hair strand: a chain of particles from root to tip.
pub struct HairStrand {
    /// Current positions (root is index 0, tip is last).
    pub positions: Vec<[f32; 3]>,
    /// Previous positions for Verlet integration.
    prev_positions: Vec<[f32; 3]>,
    /// Rest pose positions (for stiffness spring back).
    rest_positions: Vec<[f32; 3]>,
    /// Length of each segment (between consecutive particles).
    segment_lengths: Vec<f32>,
    /// Root is always pinned to its original position.
    pub root: [f32; 3],
}

impl HairStrand {
    /// Create a strand from `root` position with `segments` particles.
    ///
    /// Particles are placed along the direction `dir` (should be normalized).
    /// Segment length = `total_length / segments`.
    pub fn new(root: [f32; 3], dir: [f32; 3], total_length: f32, segments: usize) -> Self {
        let seg_len = total_length / segments as f32;
        let mut positions = vec![root];
        for i in 1..=segments {
            let p = [
                root[0] + dir[0] * seg_len * i as f32,
                root[1] + dir[1] * seg_len * i as f32,
                root[2] + dir[2] * seg_len * i as f32,
            ];
            positions.push(p);
        }
        let segment_lengths = vec![seg_len; segments];
        let rest_positions = positions.clone();
        let prev_positions = positions.clone();
        Self {
            positions,
            prev_positions,
            rest_positions,
            segment_lengths,
            root,
        }
    }

    /// Number of particles in this strand (including root).
    pub fn particle_count(&self) -> usize {
        self.positions.len()
    }

    /// Tip position.
    pub fn tip(&self) -> [f32; 3] {
        *self.positions.last().unwrap_or(&self.root)
    }
}

// ── HairConfig ────────────────────────────────────────────────────────────────

/// Configuration for hair simulation.
pub struct HairConfig {
    /// Gravity acceleration vector (m/s²).
    pub gravity: [f32; 3],
    /// Velocity damping factor in `[0..1]`.
    pub damping: f32,
    /// Fraction in `[0..1]` to spring back toward the rest pose each step.
    pub stiffness: f32,
    /// Number of constraint-satisfaction iterations per step.
    pub constraint_iters: usize,
}

impl Default for HairConfig {
    fn default() -> Self {
        Self {
            gravity: [0.0, -9.8, 0.0],
            damping: 0.98,
            stiffness: 0.1,
            constraint_iters: 3,
        }
    }
}

// ── HairSystem ────────────────────────────────────────────────────────────────

/// A collection of hair strands forming a hair system.
pub struct HairSystem {
    /// All strands in this system.
    pub strands: Vec<HairStrand>,
    /// Simulation configuration.
    pub config: HairConfig,
}

impl HairSystem {
    /// Create an empty system with the given configuration.
    pub fn new(config: HairConfig) -> Self {
        Self {
            strands: Vec::new(),
            config,
        }
    }

    /// Add a strand to this system.
    pub fn add_strand(&mut self, strand: HairStrand) {
        self.strands.push(strand);
    }

    /// Advance all strands by `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        let config = &self.config;
        let dt2 = dt * dt;

        for strand in &mut self.strands {
            // 1. Verlet integration (skip particle 0 = root)
            for i in 1..strand.positions.len() {
                let vel = sub(strand.positions[i], strand.prev_positions[i]);
                let new_pos = add(
                    add(strand.positions[i], scale(vel, config.damping)),
                    scale(config.gravity, dt2),
                );
                strand.prev_positions[i] = strand.positions[i];
                strand.positions[i] = new_pos;
            }

            // 2. Distance constraints
            for _ in 0..config.constraint_iters {
                for i in 0..strand.segment_lengths.len() {
                    let a = i;
                    let b = i + 1;
                    let d = sub(strand.positions[b], strand.positions[a]);
                    let len = length(d);
                    if len < 1e-10 {
                        continue;
                    }
                    let corr = scale(d, (1.0 - strand.segment_lengths[i] / len) * 0.5);
                    if a == 0 {
                        // root is pinned
                        strand.positions[b] = sub(strand.positions[b], scale(corr, 2.0));
                    } else {
                        strand.positions[a] = add(strand.positions[a], corr);
                        strand.positions[b] = sub(strand.positions[b], corr);
                    }
                }
            }

            // 3. Stiffness: pull toward rest pose
            for i in 1..strand.positions.len() {
                strand.positions[i] = lerp3(
                    strand.positions[i],
                    strand.rest_positions[i],
                    config.stiffness,
                );
            }

            // 4. Pin root
            strand.positions[0] = strand.root;
        }
    }

    /// Get all strand tip positions.
    pub fn tip_positions(&self) -> Vec<[f32; 3]> {
        self.strands.iter().map(|s| s.tip()).collect()
    }

    /// Get all particle positions (flattened across all strands).
    pub fn all_positions(&self) -> Vec<[f32; 3]> {
        self.strands
            .iter()
            .flat_map(|s| s.positions.iter().copied())
            .collect()
    }

    /// Create a hair system from scalp surface positions and normals.
    ///
    /// Each `(position, normal)` pair seeds one strand. The strand grows
    /// along the outward normal with the given `segments` count and `length`.
    pub fn from_scalp(
        scalp_positions: &[[f32; 3]],
        scalp_normals: &[[f32; 3]],
        segments: usize,
        length: f32,
        config: HairConfig,
    ) -> Self {
        let mut system = Self::new(config);
        for (pos, nor) in scalp_positions.iter().zip(scalp_normals.iter()) {
            let strand = HairStrand::new(*pos, *nor, length, segments);
            system.add_strand(strand);
        }
        system
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn strand_particle_count() {
        let strand = HairStrand::new([0.0, 0.0, 0.0], [0.0, -1.0, 0.0], 1.0, 4);
        assert_eq!(strand.particle_count(), 5);
    }

    #[test]
    fn strand_root_is_first() {
        let root = [1.0_f32, 2.0, 3.0];
        let strand = HairStrand::new(root, [0.0, 1.0, 0.0], 0.5, 3);
        assert_eq!(strand.positions[0], root);
    }

    #[test]
    fn strand_tip_at_correct_distance() {
        let root = [0.0_f32, 0.0, 0.0];
        let total_length = 1.0_f32;
        let strand = HairStrand::new(root, [0.0, -1.0, 0.0], total_length, 4);
        let tip = strand.tip();
        let dist = length(sub(tip, root));
        assert!(
            approx_eq(dist, total_length, 1e-4),
            "expected tip distance {total_length}, got {dist}"
        );
    }

    #[test]
    fn hair_system_step_moves_tip() {
        let mut system = HairSystem::new(HairConfig::default());
        let strand = HairStrand::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.5, 4);
        let tip_before = strand.tip();
        system.add_strand(strand);
        system.step(0.016);
        let tip_after = system.strands[0].tip();
        // Gravity pulls downward; tip should have moved
        assert!(
            tip_after[1] < tip_before[1],
            "tip should move downward under gravity, before={}, after={}",
            tip_before[1],
            tip_after[1]
        );
    }

    #[test]
    fn hair_system_root_pinned() {
        let root = [0.0_f32, 1.0, 0.0];
        let mut system = HairSystem::new(HairConfig::default());
        let strand = HairStrand::new(root, [0.0, 1.0, 0.0], 0.5, 4);
        system.add_strand(strand);
        for _ in 0..20 {
            system.step(0.016);
        }
        assert_eq!(
            system.strands[0].positions[0], root,
            "root must remain pinned"
        );
    }

    #[test]
    fn from_scalp_creates_strands() {
        let positions: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0], [0.1, 1.0, 0.0], [0.0, 1.0, 0.1]];
        let normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let system = HairSystem::from_scalp(&positions, &normals, 4, 0.1, Default::default());
        assert_eq!(system.strands.len(), positions.len());
    }

    #[test]
    fn all_positions_count() {
        let mut system = HairSystem::new(HairConfig::default());
        system.add_strand(HairStrand::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, 3)); // 4 particles
        system.add_strand(HairStrand::new([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, 5)); // 6 particles
        let all = system.all_positions();
        assert_eq!(
            all.len(),
            4 + 6,
            "expected 10 total positions, got {}",
            all.len()
        );
    }
}
