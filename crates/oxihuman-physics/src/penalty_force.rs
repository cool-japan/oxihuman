// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Penalty force method — enforces constraints via spring-like penalty forces.

/// A penalty force element connecting two points.
#[derive(Debug, Clone)]
pub struct PenaltyElement {
    pub point_a: [f64; 3],
    pub point_b: [f64; 3],
    pub stiffness: f64,
    pub damping: f64,
    pub rest_length: f64,
}

impl PenaltyElement {
    /// Create a new penalty spring element.
    pub fn new(
        point_a: [f64; 3],
        point_b: [f64; 3],
        stiffness: f64,
        damping: f64,
        rest_length: f64,
    ) -> Self {
        PenaltyElement {
            point_a,
            point_b,
            stiffness: stiffness.max(0.0),
            damping: damping.max(0.0),
            rest_length: rest_length.max(0.0),
        }
    }

    /// Current distance between the two points.
    pub fn current_distance(&self) -> f64 {
        let dx = self.point_a[0] - self.point_b[0];
        let dy = self.point_a[1] - self.point_b[1];
        let dz = self.point_a[2] - self.point_b[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Penetration depth (positive if overlapping: dist < rest_length for contact springs).
    pub fn penetration(&self) -> f64 {
        (self.rest_length - self.current_distance()).max(0.0)
    }

    /// Compute penalty force magnitude: k * penetration.
    pub fn force_magnitude(&self) -> f64 {
        self.stiffness * self.penetration()
    }

    /// Compute penalty force direction (from B to A, normalized).
    pub fn force_direction(&self) -> [f64; 3] {
        let dx = self.point_a[0] - self.point_b[0];
        let dy = self.point_a[1] - self.point_b[1];
        let dz = self.point_a[2] - self.point_b[2];
        let len = (dx * dx + dy * dy + dz * dz).sqrt();
        if len < 1e-12 {
            return [0.0, 1.0, 0.0];
        }
        [dx / len, dy / len, dz / len]
    }

    /// Compute full force vector at point A.
    pub fn force_on_a(&self, vel_a: [f64; 3], vel_b: [f64; 3]) -> [f64; 3] {
        let dir = self.force_direction();
        let fmag = self.force_magnitude();
        let dv = [
            vel_a[0] - vel_b[0],
            vel_a[1] - vel_b[1],
            vel_a[2] - vel_b[2],
        ];
        let vrel = dv[0] * dir[0] + dv[1] * dir[1] + dv[2] * dir[2];
        let total = fmag - self.damping * vrel;
        [dir[0] * total, dir[1] * total, dir[2] * total]
    }
}

/// Compute total penalty energy for a set of elements.
pub fn total_penalty_energy(elements: &[PenaltyElement]) -> f64 {
    elements
        .iter()
        .map(|e| {
            let p = e.penetration();
            0.5 * e.stiffness * p * p
        })
        .sum()
}

/// Apply penalty forces to velocity arrays.
pub fn apply_penalty_forces(
    elements: &[PenaltyElement],
    vel_a: &mut [[f64; 3]],
    vel_b: &mut [[f64; 3]],
    mass_a: &[f64],
    mass_b: &[f64],
    dt: f64,
) {
    for (idx, e) in elements.iter().enumerate() {
        if idx >= vel_a.len() || idx >= vel_b.len() {
            break;
        }
        let f = e.force_on_a(vel_a[idx], vel_b[idx]);
        let im_a = if mass_a[idx] > 1e-12 {
            1.0 / mass_a[idx]
        } else {
            0.0
        };
        let im_b = if mass_b[idx] > 1e-12 {
            1.0 / mass_b[idx]
        } else {
            0.0
        };
        for k in 0..3 {
            vel_a[idx][k] += im_a * f[k] * dt;
            vel_b[idx][k] -= im_b * f[k] * dt;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_distance() {
        let e = PenaltyElement::new([0.0, 0.0, 0.0], [3.0, 4.0, 0.0], 1.0, 0.0, 0.0);
        assert!((e.current_distance() - 5.0).abs() < 1e-10 /* 3-4-5 triangle */);
    }

    #[test]
    fn test_no_penetration() {
        let e = PenaltyElement::new([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], 1.0, 0.0, 1.0);
        assert_eq!(
            e.penetration(),
            0.0 /* no overlap when dist > rest_length */
        );
    }

    #[test]
    fn test_penetration_positive() {
        let e = PenaltyElement::new([0.0, 0.0, 0.0], [0.5, 0.0, 0.0], 1.0, 0.0, 1.0);
        assert!(e.penetration() > 0.0 /* overlapping */);
    }

    #[test]
    fn test_force_magnitude_zero_no_penetration() {
        let e = PenaltyElement::new([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], 100.0, 0.0, 1.0);
        assert_eq!(e.force_magnitude(), 0.0 /* no penetration, no force */);
    }

    #[test]
    fn test_force_magnitude_nonzero() {
        let e = PenaltyElement::new([0.0, 0.0, 0.0], [0.5, 0.0, 0.0], 10.0, 0.0, 1.0);
        assert!(e.force_magnitude() > 0.0 /* penetration produces force */);
    }

    #[test]
    fn test_force_direction_unit() {
        let e = PenaltyElement::new([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0, 0.0, 0.0);
        let d = e.force_direction();
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-10 /* unit length */);
    }

    #[test]
    fn test_total_penalty_energy_zero() {
        let e = PenaltyElement::new([0.0; 3], [2.0, 0.0, 0.0], 1.0, 0.0, 1.0);
        let energy = total_penalty_energy(&[e]);
        assert_eq!(energy, 0.0 /* no overlap, no energy */);
    }

    #[test]
    fn test_total_penalty_energy_nonzero() {
        let e = PenaltyElement::new([0.0; 3], [0.5, 0.0, 0.0], 10.0, 0.0, 1.0);
        let energy = total_penalty_energy(&[e]);
        assert!(energy > 0.0 /* overlap gives positive energy */);
    }

    #[test]
    fn test_stiffness_clamped() {
        let e = PenaltyElement::new([0.0; 3], [0.0; 3], -1.0, -1.0, 0.0);
        assert!(e.stiffness >= 0.0 /* clamped to zero */);
        assert!(e.damping >= 0.0 /* clamped to zero */);
    }
}
