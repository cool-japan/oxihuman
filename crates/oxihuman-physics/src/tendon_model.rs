// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Tendon compliance (series elastic element) model.

/// A tendon with nonlinear compliance.
#[derive(Debug, Clone)]
pub struct Tendon {
    /// Slack length (unstretched, m).
    pub slack_len: f32,
    /// Current length (m).
    pub current_len: f32,
    /// Stiffness coefficient (N/m).
    pub stiffness: f32,
    /// Damping coefficient.
    pub damping: f32,
}

impl Tendon {
    pub fn new(slack_len: f32, stiffness: f32) -> Self {
        Tendon {
            slack_len,
            current_len: slack_len,
            stiffness,
            damping: 0.01,
        }
    }
}

/// Create a new tendon.
pub fn new_tendon(slack_len: f32, stiffness: f32) -> Tendon {
    Tendon::new(slack_len, stiffness)
}

/// Return the tendon strain (stretch / slack_len).
pub fn tendon_strain(t: &Tendon) -> f32 {
    ((t.current_len - t.slack_len) / t.slack_len.max(1e-10)).max(0.0)
}

/// Return tendon force (nonlinear: quadratic toe region then linear).
pub fn tendon_force(t: &Tendon) -> f32 {
    let strain = tendon_strain(t);
    if strain <= 0.0 {
        0.0
    } else if strain < 0.02 {
        /* toe region: quadratic */
        3750.0 * strain * strain * t.stiffness
    } else {
        /* linear region */
        t.stiffness * (strain - 0.01)
    }
}

/// Set the current tendon length.
pub fn tendon_set_length(t: &mut Tendon, len: f32) {
    t.current_len = len.max(0.0);
}

/// Return `true` if the tendon is under tension (stretched beyond slack).
pub fn tendon_is_taut(t: &Tendon) -> bool {
    t.current_len > t.slack_len + 1e-6
}

/// Elongation (stretch beyond slack length, clamped ≥ 0).
pub fn tendon_elongation(t: &Tendon) -> f32 {
    (t.current_len - t.slack_len).max(0.0)
}

/// Stiffness at current strain (tangent stiffness).
pub fn tendon_tangent_stiffness(t: &Tendon) -> f32 {
    let strain = tendon_strain(t);
    if strain <= 0.0 {
        0.0
    } else if strain < 0.02 {
        7500.0 * strain * t.stiffness
    } else {
        t.stiffness
    }
}

/// Scale tendon stiffness by `factor`.
pub fn tendon_scale_stiffness(t: &mut Tendon, factor: f32) {
    t.stiffness *= factor.max(0.0);
}

/// Compute energy stored in the tendon (integral of force over elongation).
pub fn tendon_stored_energy(t: &Tendon) -> f32 {
    let el = tendon_elongation(t);
    0.5 * tendon_force(t) * el
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tendon_at_slack() {
        let t = new_tendon(0.10, 1000.0);
        assert!((t.slack_len - 0.10).abs() < 1e-6);
        assert!(!tendon_is_taut(&t));
    }

    #[test]
    fn test_force_zero_when_slack() {
        let t = new_tendon(0.10, 1000.0);
        assert!(tendon_force(&t).abs() < 1e-5);
    }

    #[test]
    fn test_force_positive_when_taut() {
        let mut t = new_tendon(0.10, 1000.0);
        tendon_set_length(&mut t, 0.12);
        assert!(tendon_force(&t) > 0.0);
    }

    #[test]
    fn test_taut_when_stretched() {
        let mut t = new_tendon(0.10, 1000.0);
        tendon_set_length(&mut t, 0.15);
        assert!(tendon_is_taut(&t));
    }

    #[test]
    fn test_elongation_zero_at_slack() {
        let t = new_tendon(0.10, 1000.0);
        assert!((tendon_elongation(&t) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_strain_increases_with_length() {
        let mut t = new_tendon(0.10, 1000.0);
        tendon_set_length(&mut t, 0.12);
        let s1 = tendon_strain(&t);
        tendon_set_length(&mut t, 0.15);
        let s2 = tendon_strain(&t);
        assert!(s2 > s1);
    }

    #[test]
    fn test_scale_stiffness() {
        let mut t = new_tendon(0.10, 1000.0);
        tendon_scale_stiffness(&mut t, 2.0);
        assert!((t.stiffness - 2000.0).abs() < 1e-3);
    }

    #[test]
    fn test_stored_energy_positive_when_taut() {
        let mut t = new_tendon(0.10, 1000.0);
        tendon_set_length(&mut t, 0.15);
        assert!(tendon_stored_energy(&t) > 0.0);
    }

    #[test]
    fn test_tangent_stiffness_nonzero_when_taut() {
        let mut t = new_tendon(0.10, 1000.0);
        tendon_set_length(&mut t, 0.13);
        assert!(tendon_tangent_stiffness(&t) > 0.0);
    }
}
