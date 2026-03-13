// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct CrowdAgent {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub goal: [f32; 2],
    pub radius: f32,
    pub max_speed: f32,
}

pub fn new_crowd_agent(pos: [f32; 2], goal: [f32; 2]) -> CrowdAgent {
    CrowdAgent {
        pos,
        vel: [0.0, 0.0],
        goal,
        radius: 0.3,
        max_speed: 1.4,
    }
}

pub fn agent_driving_force(a: &CrowdAgent, desired_speed: f32) -> [f32; 2] {
    let dx = a.goal[0] - a.pos[0];
    let dy = a.goal[1] - a.pos[1];
    let dist = (dx * dx + dy * dy).sqrt().max(1e-8);
    let ux = dx / dist;
    let uy = dy / dist;
    /* desired velocity - current velocity = driving force */
    let dvx = ux * desired_speed - a.vel[0];
    let dvy = uy * desired_speed - a.vel[1];
    [dvx, dvy]
}

pub fn agent_repulsion(a: &CrowdAgent, b: &CrowdAgent) -> [f32; 2] {
    let dx = a.pos[0] - b.pos[0];
    let dy = a.pos[1] - b.pos[1];
    let dist = (dx * dx + dy * dy).sqrt().max(1e-8);
    /* exponential decay repulsion */
    let strength = (-dist / (a.radius + b.radius)).exp();
    let nx = dx / dist;
    let ny = dy / dist;
    [strength * nx, strength * ny]
}

pub fn agent_step(a: &mut CrowdAgent, force: [f32; 2], dt: f32) {
    a.vel[0] += force[0] * dt;
    a.vel[1] += force[1] * dt;
    /* clamp to max_speed */
    let speed = (a.vel[0] * a.vel[0] + a.vel[1] * a.vel[1]).sqrt();
    if speed > a.max_speed {
        let scale = a.max_speed / speed;
        a.vel[0] *= scale;
        a.vel[1] *= scale;
    }
    a.pos[0] += a.vel[0] * dt;
    a.pos[1] += a.vel[1] * dt;
}

pub fn agent_has_reached_goal(a: &CrowdAgent, tol: f32) -> bool {
    let dx = a.goal[0] - a.pos[0];
    let dy = a.goal[1] - a.pos[1];
    (dx * dx + dy * dy).sqrt() < tol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_crowd_agent() {
        /* create agent at pos with goal */
        let a = new_crowd_agent([0.0, 0.0], [10.0, 0.0]);
        assert_eq!(a.pos, [0.0, 0.0]);
        assert_eq!(a.goal, [10.0, 0.0]);
    }

    #[test]
    fn test_driving_force_direction() {
        /* driving force points toward goal */
        let a = new_crowd_agent([0.0, 0.0], [1.0, 0.0]);
        let f = agent_driving_force(&a, 1.0);
        assert!(f[0] > 0.0);
    }

    #[test]
    fn test_agent_repulsion() {
        /* repulsion pushes agents apart */
        let a = new_crowd_agent([0.0, 0.0], [5.0, 0.0]);
        let b = new_crowd_agent([0.4, 0.0], [5.0, 0.0]);
        let f = agent_repulsion(&a, &b);
        /* force on a points left (negative x) away from b */
        assert!(f[0] < 0.0);
    }

    #[test]
    fn test_agent_step_moves() {
        /* step integrates motion */
        let mut a = new_crowd_agent([0.0, 0.0], [5.0, 0.0]);
        agent_step(&mut a, [1.0, 0.0], 0.1);
        assert!(a.pos[0] > 0.0);
    }

    #[test]
    fn test_agent_not_at_goal() {
        /* agent far from goal has not reached it */
        let a = new_crowd_agent([0.0, 0.0], [10.0, 0.0]);
        assert!(!agent_has_reached_goal(&a, 0.1));
    }

    #[test]
    fn test_agent_at_goal() {
        /* agent near goal returns true */
        let a = new_crowd_agent([10.0, 0.0], [10.0, 0.0]);
        assert!(agent_has_reached_goal(&a, 0.1));
    }

    #[test]
    fn test_agent_speed_clamped() {
        /* speed clamped to max_speed */
        let mut a = new_crowd_agent([0.0, 0.0], [100.0, 0.0]);
        for _ in 0..100 {
            let f = agent_driving_force(&a, a.max_speed);
            agent_step(&mut a, f, 1.0);
        }
        let speed = (a.vel[0] * a.vel[0] + a.vel[1] * a.vel[1]).sqrt();
        assert!(speed <= a.max_speed + 1e-5);
    }
}
