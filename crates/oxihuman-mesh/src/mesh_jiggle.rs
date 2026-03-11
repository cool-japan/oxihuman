// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct JiggleVertex {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub target: [f32; 3],
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

pub fn new_jiggle_vertex(pos: [f32; 3], stiffness: f32, damping: f32) -> JiggleVertex {
    JiggleVertex {
        pos,
        vel: [0.0; 3],
        target: pos,
        stiffness,
        damping,
        mass: 1.0,
    }
}

/// Spring-mass step toward target.
pub fn jiggle_step(j: &mut JiggleVertex, dt: f32) {
    let inv_mass = if j.mass > 1e-8 { 1.0 / j.mass } else { 0.0 };
    for i in 0..3 {
        let spring_force = j.stiffness * (j.target[i] - j.pos[i]);
        let damp_force = -j.damping * j.vel[i];
        let acc = (spring_force + damp_force) * inv_mass;
        j.vel[i] += acc * dt;
        j.pos[i] += j.vel[i] * dt;
    }
}

pub fn jiggle_set_target(j: &mut JiggleVertex, target: [f32; 3]) {
    j.target = target;
}

pub fn jiggle_offset(j: &JiggleVertex) -> f32 {
    let d = [
        j.pos[0] - j.target[0],
        j.pos[1] - j.target[1],
        j.pos[2] - j.target[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

pub fn jiggle_is_settled(j: &JiggleVertex, tol: f32) -> bool {
    jiggle_offset(j) < tol
        && (j.vel[0] * j.vel[0] + j.vel[1] * j.vel[1] + j.vel[2] * j.vel[2]).sqrt() < tol
}

pub fn jiggle_impulse(j: &mut JiggleVertex, dv: [f32; 3]) {
    j.vel[0] += dv[0];
    j.vel[1] += dv[1];
    j.vel[2] += dv[2];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_jiggle_vertex() {
        /* target equals pos initially */
        let j = new_jiggle_vertex([1.0, 0.0, 0.0], 10.0, 1.0);
        assert!((jiggle_offset(&j)).abs() < 1e-6);
    }

    #[test]
    fn test_jiggle_step_moves_toward_target() {
        /* step moves pos toward new target */
        let mut j = new_jiggle_vertex([0.0, 0.0, 0.0], 10.0, 0.1);
        jiggle_set_target(&mut j, [1.0, 0.0, 0.0]);
        jiggle_step(&mut j, 0.016);
        assert!(j.pos[0] > 0.0);
    }

    #[test]
    fn test_jiggle_set_target() {
        /* set target changes offset */
        let mut j = new_jiggle_vertex([0.0, 0.0, 0.0], 1.0, 0.5);
        jiggle_set_target(&mut j, [1.0, 0.0, 0.0]);
        assert!((jiggle_offset(&j) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_jiggle_offset_settled() {
        /* at target => settled */
        let j = new_jiggle_vertex([0.0, 0.0, 0.0], 1.0, 1.0);
        assert!(jiggle_is_settled(&j, 1e-3));
    }

    #[test]
    fn test_jiggle_impulse() {
        /* impulse adds velocity */
        let mut j = new_jiggle_vertex([0.0, 0.0, 0.0], 1.0, 0.5);
        jiggle_impulse(&mut j, [0.0, 1.0, 0.0]);
        assert!((j.vel[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_jiggle_not_settled_after_impulse() {
        /* impulse -> not settled */
        let mut j = new_jiggle_vertex([0.0, 0.0, 0.0], 1.0, 0.0);
        jiggle_impulse(&mut j, [0.0, 5.0, 0.0]);
        jiggle_step(&mut j, 0.016);
        assert!(!jiggle_is_settled(&j, 1e-3));
    }

    #[test]
    fn test_jiggle_converges() {
        /* many steps => converges to target */
        let mut j = new_jiggle_vertex([0.0, 0.0, 0.0], 50.0, 20.0);
        jiggle_set_target(&mut j, [1.0, 0.0, 0.0]);
        for _ in 0..500 {
            jiggle_step(&mut j, 0.01);
        }
        assert!(jiggle_offset(&j) < 0.1);
    }
}
