// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Inextensible cable constraint (no extension, free compression).

/// A cable constraint between two point masses.
#[derive(Debug, Clone)]
pub struct CableConstraint {
    pub max_length: f64,
    pub stiffness: f64,
}

impl CableConstraint {
    /// Create a new cable constraint.
    pub fn new(max_length: f64, stiffness: f64) -> Self {
        CableConstraint { max_length, stiffness }
    }

    /// Compute the corrective impulse for positions `pa`, `pb` and masses `ma`, `mb`.
    /// Returns the position corrections `(delta_a, delta_b)`.
    pub fn solve(
        &self,
        pa: [f64; 3],
        pb: [f64; 3],
        inv_mass_a: f64,
        inv_mass_b: f64,
    ) -> ([f64; 3], [f64; 3]) {
        let diff = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let dist = (diff[0] * diff[0] + diff[1] * diff[1] + diff[2] * diff[2]).sqrt();
        if dist <= self.max_length || dist < 1e-12 {
            return ([0.0; 3], [0.0; 3]);
        }
        let violation = dist - self.max_length;
        let total_inv = inv_mass_a + inv_mass_b;
        if total_inv < 1e-30 {
            return ([0.0; 3], [0.0; 3]);
        }
        let corr = violation * self.stiffness / total_inv;
        let n = [diff[0] / dist, diff[1] / dist, diff[2] / dist];
        let da = [n[0] * corr * inv_mass_a, n[1] * corr * inv_mass_a, n[2] * corr * inv_mass_a];
        let db = [-n[0] * corr * inv_mass_b, -n[1] * corr * inv_mass_b, -n[2] * corr * inv_mass_b];
        (da, db)
    }

    /// True if the cable is taut (distance >= max_length).
    pub fn is_taut(&self, pa: [f64; 3], pb: [f64; 3]) -> bool {
        let d2 = (0..3).map(|k| (pb[k] - pa[k]).powi(2)).sum::<f64>();
        d2.sqrt() >= self.max_length
    }
}

/// Create a new cable constraint.
pub fn new_cable_constraint(max_length: f64, stiffness: f64) -> CableConstraint {
    CableConstraint::new(max_length, stiffness)
}

/// Solve the constraint.
pub fn cc_solve(
    c: &CableConstraint,
    pa: [f64; 3],
    pb: [f64; 3],
    inv_ma: f64,
    inv_mb: f64,
) -> ([f64; 3], [f64; 3]) {
    c.solve(pa, pb, inv_ma, inv_mb)
}

/// Check if taut.
pub fn cc_is_taut(c: &CableConstraint, pa: [f64; 3], pb: [f64; 3]) -> bool {
    c.is_taut(pa, pb)
}

/// Max length.
pub fn cc_max_length(c: &CableConstraint) -> f64 {
    c.max_length
}

/// Stiffness.
pub fn cc_stiffness(c: &CableConstraint) -> f64 {
    c.stiffness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_correction_when_slack() {
        let c = new_cable_constraint(5.0, 1.0);
        let (da, db) = cc_solve(&c, [0.0; 3], [3.0, 0.0, 0.0], 1.0, 1.0);
        assert!(da.iter().all(|&x| x == 0.0) /* slack cable — no correction */);
        assert!(db.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_correction_when_taut() {
        let c = new_cable_constraint(2.0, 1.0);
        let (da, db) = cc_solve(&c, [0.0; 3], [4.0, 0.0, 0.0], 1.0, 1.0);
        assert!(da[0] > 0.0 /* a moves toward b */);
        assert!(db[0] < 0.0 /* b moves toward a */);
    }

    #[test]
    fn test_is_taut() {
        let c = new_cable_constraint(1.0, 1.0);
        assert!(cc_is_taut(&c, [0.0; 3], [2.0, 0.0, 0.0]) /* taut */);
    }

    #[test]
    fn test_is_not_taut() {
        let c = new_cable_constraint(5.0, 1.0);
        assert!(!cc_is_taut(&c, [0.0; 3], [1.0, 0.0, 0.0]) /* slack */);
    }

    #[test]
    fn test_max_length() {
        let c = new_cable_constraint(3.5, 0.9);
        assert!((cc_max_length(&c) - 3.5).abs() < 1e-12 /* length stored */);
    }

    #[test]
    fn test_stiffness() {
        let c = new_cable_constraint(1.0, 0.75);
        assert!((cc_stiffness(&c) - 0.75).abs() < 1e-12 /* stiffness stored */);
    }

    #[test]
    fn test_equal_masses_symmetric() {
        let c = new_cable_constraint(1.0, 1.0);
        let (da, db) = cc_solve(&c, [0.0; 3], [3.0, 0.0, 0.0], 1.0, 1.0);
        assert!((da[0] + db[0]).abs() < 1e-10 /* equal and opposite */);
    }

    #[test]
    fn test_pinned_b_absorbs_all() {
        let c = new_cable_constraint(1.0, 1.0);
        /* inv_mass_b = 0 means b is pinned */
        let (da, _db) = cc_solve(&c, [0.0; 3], [3.0, 0.0, 0.0], 1.0, 0.0);
        assert!(da[0] > 0.0 /* a takes all correction */);
    }

    #[test]
    fn test_zero_distance_no_correction() {
        let c = new_cable_constraint(0.5, 1.0);
        let (da, db) = cc_solve(&c, [0.0; 3], [0.0; 3], 1.0, 1.0);
        assert!(da.iter().all(|&x| x == 0.0) /* degenerate case */);
        assert!(db.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_3d_taut() {
        let c = new_cable_constraint(1.0, 1.0);
        let pa = [0.0, 0.0, 0.0];
        let pb = [1.0, 1.0, 1.0]; /* distance = sqrt(3) > 1 */
        assert!(cc_is_taut(&c, pa, pb) /* taut in 3D */);
    }
}
