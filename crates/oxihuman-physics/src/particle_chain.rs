// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Verlet-integrated particle chain with distance constraints.
#[allow(dead_code)]
pub struct ChainParticle {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub pinned: bool,
}

#[allow(dead_code)]
pub struct ParticleChain {
    pub particles: Vec<ChainParticle>,
    pub segment_length: f32,
    pub constraint_iters: usize,
}

#[allow(dead_code)]
impl ParticleChain {
    pub fn new(n: usize, segment_length: f32, constraint_iters: usize) -> Self {
        let particles = (0..n)
            .map(|i| {
                let p = [0.0, -(i as f32) * segment_length, 0.0];
                ChainParticle {
                    pos: p,
                    prev_pos: p,
                    pinned: i == 0,
                }
            })
            .collect();
        Self {
            particles,
            segment_length,
            constraint_iters,
        }
    }
    pub fn step(&mut self, dt: f32, gravity: [f32; 3]) {
        let n = self.particles.len();
        for p in &mut self.particles {
            if p.pinned {
                continue;
            }
            let vel = [
                p.pos[0] - p.prev_pos[0],
                p.pos[1] - p.prev_pos[1],
                p.pos[2] - p.prev_pos[2],
            ];
            p.prev_pos = p.pos;
            for i in 0..3 {
                p.pos[i] += vel[i] + gravity[i] * dt * dt;
            }
        }
        for _ in 0..self.constraint_iters {
            for j in 0..n.saturating_sub(1) {
                let (left, right) = self.particles.split_at_mut(j + 1);
                let pa = &mut left[j];
                let pb = &mut right[0];
                let dx = pb.pos[0] - pa.pos[0];
                let dy = pb.pos[1] - pa.pos[1];
                let dz = pb.pos[2] - pa.pos[2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-8);
                let correction = (dist - self.segment_length) / dist;
                let half = 0.5;
                if !pa.pinned {
                    pa.pos[0] += dx * correction * half;
                    pa.pos[1] += dy * correction * half;
                    pa.pos[2] += dz * correction * half;
                }
                if !pb.pinned {
                    pb.pos[0] -= dx * correction * half;
                    pb.pos[1] -= dy * correction * half;
                    pb.pos[2] -= dz * correction * half;
                }
            }
        }
    }
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
    pub fn chain_length(&self) -> f32 {
        self.segment_length * (self.particles.len().saturating_sub(1)) as f32
    }
    pub fn end_pos(&self) -> Option<[f32; 3]> {
        self.particles.last().map(|p| p.pos)
    }
    pub fn tip_distance_from_root(&self) -> f32 {
        if self.particles.len() < 2 {
            return 0.0;
        }
        let r = self.particles[0].pos;
        let e = self.particles[self.particles.len() - 1].pos;
        ((r[0] - e[0]).powi(2) + (r[1] - e[1]).powi(2) + (r[2] - e[2]).powi(2)).sqrt()
    }
    pub fn average_segment_length(&self) -> f32 {
        if self.particles.len() < 2 {
            return 0.0;
        }
        let n = self.particles.len() - 1;
        let total: f32 = (0..n)
            .map(|i| {
                let a = self.particles[i].pos;
                let b = self.particles[i + 1].pos;
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
            })
            .sum();
        total / n as f32
    }
}

#[allow(dead_code)]
pub fn new_particle_chain(n: usize, seg_len: f32, iters: usize) -> ParticleChain {
    ParticleChain::new(n, seg_len, iters)
}
#[allow(dead_code)]
pub fn pc_step(c: &mut ParticleChain, dt: f32, gravity: [f32; 3]) {
    c.step(dt, gravity);
}
#[allow(dead_code)]
pub fn pc_particle_count(c: &ParticleChain) -> usize {
    c.particle_count()
}
#[allow(dead_code)]
pub fn pc_chain_length(c: &ParticleChain) -> f32 {
    c.chain_length()
}
#[allow(dead_code)]
pub fn pc_end_pos(c: &ParticleChain) -> Option<[f32; 3]> {
    c.end_pos()
}
#[allow(dead_code)]
pub fn pc_tip_distance(c: &ParticleChain) -> f32 {
    c.tip_distance_from_root()
}
#[allow(dead_code)]
pub fn pc_avg_seg_len(c: &ParticleChain) -> f32 {
    c.average_segment_length()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_particle_count() {
        let c = new_particle_chain(5, 1.0, 3);
        assert_eq!(pc_particle_count(&c), 5);
    }
    #[test]
    fn test_chain_length() {
        let c = new_particle_chain(4, 1.5, 2);
        assert!((pc_chain_length(&c) - 4.5).abs() < 1e-5);
    }
    #[test]
    fn test_root_pinned() {
        let c = new_particle_chain(3, 1.0, 2);
        assert!(c.particles[0].pinned);
    }
    #[test]
    fn test_step_doesnt_panic() {
        let mut c = new_particle_chain(4, 1.0, 3);
        pc_step(&mut c, 0.016, [0.0, -9.81, 0.0]);
    }
    #[test]
    fn test_root_unchanged_after_step() {
        let mut c = new_particle_chain(3, 1.0, 2);
        let root = c.particles[0].pos;
        for _ in 0..10 {
            pc_step(&mut c, 0.01, [0.0, -9.81, 0.0]);
        }
        assert_eq!(c.particles[0].pos, root);
    }
    #[test]
    fn test_end_pos_exists() {
        let c = new_particle_chain(3, 1.0, 2);
        assert!(pc_end_pos(&c).is_some());
    }
    #[test]
    fn test_tip_distance_less_than_chain_length() {
        let mut c = new_particle_chain(4, 1.0, 5);
        for _ in 0..30 {
            pc_step(&mut c, 0.01, [0.0, -9.81, 0.0]);
        }
        let td = pc_tip_distance(&c);
        let cl = pc_chain_length(&c);
        assert!(td <= cl + 0.5);
    }
    #[test]
    fn test_avg_seg_len_near_rest() {
        let c = new_particle_chain(3, 2.0, 5);
        let avg = pc_avg_seg_len(&c);
        assert!((avg - 2.0).abs() < 1e-5);
    }
    #[test]
    fn test_single_particle() {
        let c = new_particle_chain(1, 1.0, 2);
        assert_eq!(pc_particle_count(&c), 1);
        assert_eq!(pc_chain_length(&c), 0.0);
    }
    #[test]
    fn test_gravity_pulls_chain_down() {
        let mut c = new_particle_chain(2, 1.0, 3);
        let initial_y = c.particles[1].pos[1];
        for _ in 0..20 {
            pc_step(&mut c, 0.016, [0.0, -9.81, 0.0]);
        }
        assert!(c.particles[1].pos[1] < initial_y);
    }
}
