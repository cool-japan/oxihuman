// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A vortex field that induces rotational velocity on particles around an axis.

use std::f32::consts::PI;

/// A single vortex defined by a center, axis, strength, and falloff radius.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Vortex {
    pub center: [f32; 3],
    /// Unit axis of rotation.
    pub axis: [f32; 3],
    /// Angular velocity strength (rad/s at unit radius).
    pub strength: f32,
    /// Radius beyond which the vortex has no effect.
    pub radius: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
impl Vortex {
    pub fn new(center: [f32; 3], axis: [f32; 3], strength: f32, radius: f32) -> Self {
        let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2])
            .sqrt()
            .max(1e-6);
        let axis = [axis[0] / len, axis[1] / len, axis[2] / len];
        Self {
            center,
            axis,
            strength,
            radius: radius.max(1e-6),
            enabled: true,
        }
    }

    /// Compute the velocity induced at `pos`.
    pub fn velocity_at(&self, pos: [f32; 3]) -> [f32; 3] {
        if !self.enabled {
            return [0.0; 3];
        }
        let r = [
            pos[0] - self.center[0],
            pos[1] - self.center[1],
            pos[2] - self.center[2],
        ];
        // Project r onto the plane perpendicular to axis.
        let dot = r[0] * self.axis[0] + r[1] * self.axis[1] + r[2] * self.axis[2];
        let r_perp = [
            r[0] - dot * self.axis[0],
            r[1] - dot * self.axis[1],
            r[2] - dot * self.axis[2],
        ];
        let dist = (r_perp[0] * r_perp[0] + r_perp[1] * r_perp[1] + r_perp[2] * r_perp[2]).sqrt();
        if dist < 1e-6 || dist > self.radius {
            return [0.0; 3];
        }
        // Falloff: linear decay from 1 at center to 0 at radius.
        let factor = (1.0 - dist / self.radius).max(0.0) * self.strength;
        // Tangential direction = axis × r_perp / dist.
        let tang = [
            (self.axis[1] * r_perp[2] - self.axis[2] * r_perp[1]) / dist,
            (self.axis[2] * r_perp[0] - self.axis[0] * r_perp[2]) / dist,
            (self.axis[0] * r_perp[1] - self.axis[1] * r_perp[0]) / dist,
        ];
        [tang[0] * factor, tang[1] * factor, tang[2] * factor]
    }
}

/// A collection of vortices forming a composite field.
#[allow(dead_code)]
pub struct VortexField {
    vortices: Vec<Vortex>,
    pub time: f32,
}

#[allow(dead_code)]
impl VortexField {
    pub fn new() -> Self {
        Self {
            vortices: Vec::new(),
            time: 0.0,
        }
    }

    pub fn add_vortex(&mut self, v: Vortex) -> usize {
        let id = self.vortices.len();
        self.vortices.push(v);
        id
    }

    pub fn vortex_count(&self) -> usize {
        self.vortices.len()
    }

    pub fn set_enabled(&mut self, id: usize, enabled: bool) {
        if let Some(v) = self.vortices.get_mut(id) {
            v.enabled = enabled;
        }
    }

    /// Compute the total velocity at `pos` from all vortices.
    pub fn sample(&self, pos: [f32; 3]) -> [f32; 3] {
        let mut vel = [0.0_f32; 3];
        for v in &self.vortices {
            let dv = v.velocity_at(pos);
            vel[0] += dv[0];
            vel[1] += dv[1];
            vel[2] += dv[2];
        }
        vel
    }

    /// Apply vortex velocities to a set of positions and velocities over dt.
    pub fn apply(&self, positions: &[[f32; 3]], velocities: &mut [[f32; 3]], dt: f32) {
        for (pos, vel) in positions.iter().zip(velocities.iter_mut()) {
            let dv = self.sample(*pos);
            vel[0] += dv[0] * dt;
            vel[1] += dv[1] * dt;
            vel[2] += dv[2] * dt;
        }
    }

    pub fn advance_time(&mut self, dt: f32) {
        self.time += dt;
    }

    pub fn clear(&mut self) {
        self.vortices.clear();
        self.time = 0.0;
    }

    pub fn is_empty(&self) -> bool {
        self.vortices.is_empty()
    }
}

impl Default for VortexField {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_vortex_field() -> VortexField {
    VortexField::new()
}

/// Circulation Γ = 2π * r² * ω for a vortex at radius r.
pub fn circulation(strength: f32, radius: f32) -> f32 {
    2.0 * PI * radius * radius * strength
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn velocity_nonzero_within_radius() {
        let v = Vortex::new([0.0; 3], [0.0, 1.0, 0.0], 5.0, 10.0);
        let vel = v.velocity_at([1.0, 0.0, 0.0]);
        assert!(vel.iter().any(|&x| x.abs() > 1e-5));
    }

    #[test]
    fn velocity_zero_outside_radius() {
        let v = Vortex::new([0.0; 3], [0.0, 1.0, 0.0], 5.0, 1.0);
        let vel = v.velocity_at([5.0, 0.0, 0.0]);
        assert!(vel.iter().all(|&x| x.abs() < 1e-5));
    }

    #[test]
    fn disabled_vortex_zero_velocity() {
        let mut v = Vortex::new([0.0; 3], [0.0, 1.0, 0.0], 5.0, 10.0);
        v.enabled = false;
        let vel = v.velocity_at([1.0, 0.0, 0.0]);
        assert!(vel.iter().all(|&x| x.abs() < 1e-5));
    }

    #[test]
    fn field_accumulates_vortices() {
        let mut f = new_vortex_field();
        f.add_vortex(Vortex::new([0.0; 3], [0.0, 1.0, 0.0], 2.0, 10.0));
        f.add_vortex(Vortex::new([0.0; 3], [0.0, 1.0, 0.0], 3.0, 10.0));
        assert_eq!(f.vortex_count(), 2);
    }

    #[test]
    fn sample_sums_vortices() {
        let mut f = new_vortex_field();
        f.add_vortex(Vortex::new([0.0; 3], [0.0, 1.0, 0.0], 1.0, 10.0));
        let vel = f.sample([1.0, 0.0, 0.0]);
        assert!(vel.iter().any(|&x| x.abs() > 1e-5));
    }

    #[test]
    fn apply_updates_velocities() {
        let mut f = new_vortex_field();
        f.add_vortex(Vortex::new([0.0; 3], [0.0, 1.0, 0.0], 5.0, 10.0));
        let positions = vec![[1.0, 0.0, 0.0]];
        let mut velocities = vec![[0.0; 3]];
        f.apply(&positions, &mut velocities, 1.0);
        assert!(velocities[0].iter().any(|&x| x.abs() > 1e-5));
    }

    #[test]
    fn set_enabled_disables() {
        let mut f = new_vortex_field();
        let id = f.add_vortex(Vortex::new([0.0; 3], [0.0, 1.0, 0.0], 5.0, 10.0));
        f.set_enabled(id, false);
        let vel = f.sample([1.0, 0.0, 0.0]);
        assert!(vel.iter().all(|&x| x.abs() < 1e-5));
    }

    #[test]
    fn circulation_positive() {
        assert!(circulation(1.0, 2.0) > 0.0);
    }

    #[test]
    fn time_advances() {
        let mut f = new_vortex_field();
        f.advance_time(0.1);
        assert!((f.time - 0.1).abs() < 1e-6);
    }

    #[test]
    fn clear_empties() {
        let mut f = new_vortex_field();
        f.add_vortex(Vortex::new([0.0; 3], [0.0, 1.0, 0.0], 1.0, 5.0));
        f.clear();
        assert!(f.is_empty());
    }
}
