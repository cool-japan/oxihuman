// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 2D radial explosion impulse stub.

#[derive(Debug, Clone)]
pub struct Explosion2d {
    pub center: [f32; 2],
    pub radius: f32,
    pub peak_force: f32,
    pub falloff: f32,
}

impl Explosion2d {
    pub fn new(cx: f32, cy: f32, radius: f32, peak_force: f32) -> Self {
        Explosion2d {
            center: [cx, cy],
            radius,
            peak_force,
            falloff: 2.0,
        }
    }

    pub fn force_at(&self, pos: [f32; 2]) -> [f32; 2] {
        explosion_force(self.center, self.radius, self.peak_force, self.falloff, pos)
    }

    pub fn affects(&self, pos: [f32; 2]) -> bool {
        let dx = pos[0] - self.center[0];
        let dy = pos[1] - self.center[1];
        dx * dx + dy * dy <= self.radius * self.radius
    }
}

pub fn explosion_force(
    center: [f32; 2],
    radius: f32,
    peak: f32,
    falloff: f32,
    pos: [f32; 2],
) -> [f32; 2] {
    let dx = pos[0] - center[0];
    let dy = pos[1] - center[1];
    let dist = (dx * dx + dy * dy).sqrt();
    if dist > radius || dist < f32::EPSILON {
        return [0.0; 2];
    }
    let t = 1.0 - dist / radius;
    let mag = peak * t.powf(falloff);
    let nx = dx / dist;
    let ny = dy / dist;
    [mag * nx, mag * ny]
}

pub fn apply_explosion_to_bodies(
    explosion: &Explosion2d,
    positions: &[[f32; 2]],
    velocities: &mut [[f32; 2]],
    masses: &[f32],
    dt: f32,
) {
    for ((pos, vel), &mass) in positions
        .iter()
        .zip(velocities.iter_mut())
        .zip(masses.iter())
    {
        if mass < f32::EPSILON {
            continue;
        }
        let f = explosion.force_at(*pos);
        vel[0] += f[0] / mass * dt;
        vel[1] += f[1] / mass * dt;
    }
}

pub fn explosion_energy(explosion: &Explosion2d) -> f32 {
    std::f32::consts::PI * explosion.radius * explosion.radius * explosion.peak_force
}

pub fn shockwave_radius(explosion: &Explosion2d, time: f32, wave_speed: f32) -> f32 {
    (wave_speed * time).min(explosion.radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_center_gets_no_force() {
        let exp = Explosion2d::new(0.0, 0.0, 5.0, 100.0);
        let f = exp.force_at([0.0, 0.0]);
        assert!(f[0].abs() < 1e-5 && f[1].abs() < 1e-5, /* zero at center */);
    }

    #[test]
    fn test_outside_radius_no_force() {
        let exp = Explosion2d::new(0.0, 0.0, 5.0, 100.0);
        let f = exp.force_at([10.0, 0.0]);
        assert!(f[0].abs() < 1e-5 /* outside radius = no force */,);
    }

    #[test]
    fn test_force_within_radius() {
        let exp = Explosion2d::new(0.0, 0.0, 5.0, 100.0);
        let f = exp.force_at([2.0, 0.0]);
        assert!(f[0] > 0.0 /* force in +x direction */,);
    }

    #[test]
    fn test_affects_inside() {
        let exp = Explosion2d::new(0.0, 0.0, 5.0, 100.0);
        assert!(exp.affects([3.0, 0.0]) /* inside radius */,);
    }

    #[test]
    fn test_affects_outside() {
        let exp = Explosion2d::new(0.0, 0.0, 5.0, 100.0);
        assert!(!exp.affects([6.0, 0.0]) /* outside radius */,);
    }

    #[test]
    fn test_apply_explosion_to_bodies() {
        let exp = Explosion2d::new(0.0, 0.0, 5.0, 100.0);
        let positions = vec![[2.0f32, 0.0]];
        let mut velocities = vec![[0.0f32; 2]];
        let masses = vec![1.0f32];
        apply_explosion_to_bodies(&exp, &positions, &mut velocities, &masses, 1.0);
        assert!(velocities[0][0] > 0.0 /* body pushed outward */,);
    }

    #[test]
    fn test_explosion_energy_positive() {
        let exp = Explosion2d::new(0.0, 0.0, 5.0, 100.0);
        assert!(explosion_energy(&exp) > 0.0 /* positive energy */,);
    }

    #[test]
    fn test_shockwave_radius() {
        let exp = Explosion2d::new(0.0, 0.0, 10.0, 100.0);
        let r = shockwave_radius(&exp, 1.0, 5.0);
        assert!((r - 5.0).abs() < 1e-5 /* wave radius = speed * time */,);
    }

    #[test]
    fn test_shockwave_clamped() {
        let exp = Explosion2d::new(0.0, 0.0, 5.0, 100.0);
        let r = shockwave_radius(&exp, 100.0, 5.0);
        assert!(r <= exp.radius + 1e-5, /* clamped at explosion radius */);
    }
}
