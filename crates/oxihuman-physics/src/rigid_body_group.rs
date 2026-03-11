// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Group of rigid bodies with collective transform.

/// A body member of the group.
#[derive(Debug, Clone)]
pub struct GroupMember {
    pub id: u64,
    pub local_offset: [f32; 3],
    pub mass: f32,
}

/// A rigid body group sharing a common center of mass.
pub struct RigidBodyGroup {
    pub members: Vec<GroupMember>,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub enabled: bool,
    pub group_id: u64,
}

fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
impl RigidBodyGroup {
    pub fn new(group_id: u64) -> Self {
        RigidBodyGroup {
            members: Vec::new(),
            position: [0.0; 3],
            velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            enabled: true,
            group_id,
        }
    }

    pub fn add_member(&mut self, id: u64, local_offset: [f32; 3], mass: f32) {
        self.members.push(GroupMember {
            id,
            local_offset,
            mass: mass.max(1e-9),
        });
    }

    pub fn remove_member(&mut self, id: u64) -> bool {
        let before = self.members.len();
        self.members.retain(|m| m.id != id);
        self.members.len() < before
    }

    pub fn total_mass(&self) -> f32 {
        self.members.iter().map(|m| m.mass).sum()
    }

    pub fn center_of_mass(&self) -> [f32; 3] {
        let tm = self.total_mass();
        if tm < 1e-9 {
            return self.position;
        }
        let mut com = [0.0f32; 3];
        for m in &self.members {
            let world = vec3_add(self.position, m.local_offset);
            let weighted = vec3_scale(world, m.mass / tm);
            com[0] += weighted[0];
            com[1] += weighted[1];
            com[2] += weighted[2];
        }
        com
    }

    pub fn step(&mut self, dt: f32) {
        if !self.enabled {
            return;
        }
        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;
        self.position[2] += self.velocity[2] * dt;
    }

    pub fn apply_impulse(&mut self, impulse: [f32; 3]) {
        let tm = self.total_mass();
        if tm < 1e-9 {
            return;
        }
        let dv = vec3_scale(impulse, 1.0 / tm);
        self.velocity[0] += dv[0];
        self.velocity[1] += dv[1];
        self.velocity[2] += dv[2];
    }

    pub fn member_world_pos(&self, id: u64) -> Option<[f32; 3]> {
        self.members
            .iter()
            .find(|m| m.id == id)
            .map(|m| vec3_add(self.position, m.local_offset))
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn kinetic_energy(&self) -> f32 {
        let tm = self.total_mass();
        let v2 = self.velocity[0].powi(2) + self.velocity[1].powi(2) + self.velocity[2].powi(2);
        0.5 * tm * v2
    }

    pub fn stop(&mut self) {
        self.velocity = [0.0; 3];
        self.angular_velocity = [0.0; 3];
    }
}

pub fn new_rigid_body_group(group_id: u64) -> RigidBodyGroup {
    RigidBodyGroup::new(group_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_total_mass() {
        let mut g = new_rigid_body_group(1);
        g.add_member(1, [0.0; 3], 5.0);
        g.add_member(2, [1.0, 0.0, 0.0], 3.0);
        assert!((g.total_mass() - 8.0).abs() < 1e-6);
    }

    #[test]
    fn remove_member() {
        let mut g = new_rigid_body_group(1);
        g.add_member(1, [0.0; 3], 1.0);
        assert!(g.remove_member(1));
        assert_eq!(g.member_count(), 0);
    }

    #[test]
    fn step_updates_position() {
        let mut g = new_rigid_body_group(1);
        g.velocity = [1.0, 0.0, 0.0];
        g.step(2.0);
        assert!((g.position[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn apply_impulse() {
        let mut g = new_rigid_body_group(1);
        g.add_member(1, [0.0; 3], 2.0);
        g.apply_impulse([4.0, 0.0, 0.0]);
        assert!((g.velocity[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn member_world_pos() {
        let mut g = new_rigid_body_group(1);
        g.position = [1.0, 0.0, 0.0];
        g.add_member(1, [2.0, 0.0, 0.0], 1.0);
        let pos = g.member_world_pos(1).unwrap();
        assert!((pos[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn kinetic_energy() {
        let mut g = new_rigid_body_group(1);
        g.add_member(1, [0.0; 3], 2.0);
        g.velocity = [3.0, 4.0, 0.0];
        assert!((g.kinetic_energy() - 25.0).abs() < 1e-5);
    }

    #[test]
    fn disabled_no_step() {
        let mut g = new_rigid_body_group(1);
        g.velocity = [1.0, 0.0, 0.0];
        g.enabled = false;
        g.step(1.0);
        assert_eq!(g.position[0], 0.0);
    }

    #[test]
    fn stop_zeroes_velocity() {
        let mut g = new_rigid_body_group(1);
        g.velocity = [5.0, 5.0, 5.0];
        g.stop();
        let v2 = g.velocity[0].powi(2) + g.velocity[1].powi(2) + g.velocity[2].powi(2);
        assert!(v2 < 1e-10);
    }

    #[test]
    fn center_of_mass_empty() {
        let g = new_rigid_body_group(1);
        let com = g.center_of_mass();
        assert_eq!(com, [0.0; 3]);
    }
}
