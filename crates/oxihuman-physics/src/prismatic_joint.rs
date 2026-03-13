// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Prismatic (sliding) joint along a single axis.
#[allow(dead_code)]
pub struct PrismaticJoint {
    pub position: f32,
    pub velocity: f32,
    pub min_pos: f32,
    pub max_pos: f32,
    pub axis: [f32; 3],
    pub damping: f32,
    pub locked: bool,
}

#[allow(dead_code)]
impl PrismaticJoint {
    pub fn new(axis: [f32; 3], min_pos: f32, max_pos: f32, damping: f32) -> Self {
        let axis = normalize3(axis);
        Self {
            position: 0.0,
            velocity: 0.0,
            min_pos,
            max_pos,
            axis,
            damping,
            locked: false,
        }
    }
    pub fn apply_force(&mut self, force_along_axis: f32, mass: f32, dt: f32) {
        if self.locked {
            return;
        }
        let acc = force_along_axis / mass.max(1e-10);
        self.velocity = self.velocity * (1.0 - self.damping * dt) + acc * dt;
        self.position = (self.position + self.velocity * dt).clamp(self.min_pos, self.max_pos);
        if self.position == self.min_pos || self.position == self.max_pos {
            self.velocity = 0.0;
        }
    }
    pub fn at_limit(&self) -> bool {
        self.position <= self.min_pos || self.position >= self.max_pos
    }
    pub fn world_offset(&self) -> [f32; 3] {
        [
            self.axis[0] * self.position,
            self.axis[1] * self.position,
            self.axis[2] * self.position,
        ]
    }
    pub fn range(&self) -> f32 {
        self.max_pos - self.min_pos
    }
    pub fn normalized_pos(&self) -> f32 {
        let r = self.range();
        if r < 1e-8 {
            0.0
        } else {
            (self.position - self.min_pos) / r
        }
    }
    pub fn kinetic_energy(&self, mass: f32) -> f32 {
        0.5 * mass * self.velocity * self.velocity
    }
    pub fn lock(&mut self) {
        self.locked = true;
    }
    pub fn unlock(&mut self) {
        self.locked = false;
    }
    pub fn reset(&mut self) {
        self.position = 0.0;
        self.velocity = 0.0;
        self.locked = false;
    }
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-10);
    [v[0] / len, v[1] / len, v[2] / len]
}

#[allow(dead_code)]
pub fn new_prismatic_joint(
    axis: [f32; 3],
    min_pos: f32,
    max_pos: f32,
    damping: f32,
) -> PrismaticJoint {
    PrismaticJoint::new(axis, min_pos, max_pos, damping)
}
#[allow(dead_code)]
pub fn pj_apply_force(j: &mut PrismaticJoint, force: f32, mass: f32, dt: f32) {
    j.apply_force(force, mass, dt);
}
#[allow(dead_code)]
pub fn pj_at_limit(j: &PrismaticJoint) -> bool {
    j.at_limit()
}
#[allow(dead_code)]
pub fn pj_world_offset(j: &PrismaticJoint) -> [f32; 3] {
    j.world_offset()
}
#[allow(dead_code)]
pub fn pj_range(j: &PrismaticJoint) -> f32 {
    j.range()
}
#[allow(dead_code)]
pub fn pj_normalized_pos(j: &PrismaticJoint) -> f32 {
    j.normalized_pos()
}
#[allow(dead_code)]
pub fn pj_kinetic_energy(j: &PrismaticJoint, mass: f32) -> f32 {
    j.kinetic_energy(mass)
}
#[allow(dead_code)]
pub fn pj_lock(j: &mut PrismaticJoint) {
    j.lock();
}
#[allow(dead_code)]
pub fn pj_unlock(j: &mut PrismaticJoint) {
    j.unlock();
}
#[allow(dead_code)]
pub fn pj_reset(j: &mut PrismaticJoint) {
    j.reset();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_apply_force_moves() {
        let mut j = new_prismatic_joint([0.0, 0.0, 1.0], -1.0, 1.0, 0.0);
        pj_apply_force(&mut j, 10.0, 1.0, 0.1);
        assert!(j.position > 0.0);
    }
    #[test]
    fn test_clamps_to_max() {
        let mut j = new_prismatic_joint([1.0, 0.0, 0.0], -1.0, 1.0, 0.0);
        for _ in 0..100 {
            pj_apply_force(&mut j, 1000.0, 1.0, 0.1);
        }
        assert!(j.position <= 1.0 + 1e-5);
    }
    #[test]
    fn test_at_limit() {
        let mut j = new_prismatic_joint([1.0, 0.0, 0.0], 0.0, 1.0, 0.0);
        j.position = 1.0;
        assert!(pj_at_limit(&j));
    }
    #[test]
    fn test_locked_doesnt_move() {
        let mut j = new_prismatic_joint([1.0, 0.0, 0.0], -1.0, 1.0, 0.0);
        pj_lock(&mut j);
        pj_apply_force(&mut j, 1000.0, 1.0, 0.1);
        assert_eq!(j.position, 0.0);
    }
    #[test]
    fn test_unlock() {
        let mut j = new_prismatic_joint([1.0, 0.0, 0.0], -1.0, 1.0, 0.0);
        pj_lock(&mut j);
        pj_unlock(&mut j);
        pj_apply_force(&mut j, 10.0, 1.0, 0.1);
        assert!(j.position > 0.0);
    }
    #[test]
    fn test_world_offset() {
        let mut j = new_prismatic_joint([0.0, 1.0, 0.0], -1.0, 1.0, 0.0);
        j.position = 0.5;
        let off = pj_world_offset(&j);
        assert!((off[1] - 0.5).abs() < 1e-5);
    }
    #[test]
    fn test_range() {
        let j = new_prismatic_joint([1.0, 0.0, 0.0], -2.0, 3.0, 0.0);
        assert!((pj_range(&j) - 5.0).abs() < 1e-5);
    }
    #[test]
    fn test_normalized_pos() {
        let mut j = new_prismatic_joint([1.0, 0.0, 0.0], 0.0, 2.0, 0.0);
        j.position = 1.0;
        assert!((pj_normalized_pos(&j) - 0.5).abs() < 1e-5);
    }
    #[test]
    fn test_kinetic_energy() {
        let mut j = new_prismatic_joint([1.0, 0.0, 0.0], -1.0, 1.0, 0.0);
        pj_apply_force(&mut j, 10.0, 1.0, 0.1);
        assert!(pj_kinetic_energy(&j, 1.0) > 0.0);
    }
    #[test]
    fn test_reset() {
        let mut j = new_prismatic_joint([1.0, 0.0, 0.0], -1.0, 1.0, 0.0);
        pj_apply_force(&mut j, 10.0, 1.0, 0.5);
        pj_reset(&mut j);
        assert_eq!(j.position, 0.0);
        assert_eq!(j.velocity, 0.0);
    }
}
