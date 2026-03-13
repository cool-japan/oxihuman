// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Phase-field fracture model stub — uses a scalar damage field d ∈ `[0,1]`
//! to represent crack evolution without explicit crack tracking.

/// Phase-field fracture model on a 1-D bar.
pub struct PhaseFieldFracture {
    pub n: usize,         /* number of nodes */
    pub d: Vec<f64>,      /* damage field (0 = intact, 1 = fully broken) */
    pub u: Vec<f64>,      /* displacement field */
    pub dx: f64,          /* node spacing */
    pub gc: f64,          /* critical energy release rate */
    pub l0: f64,          /* regularisation length */
    pub elastic_mod: f64, /* elastic modulus */
}

impl PhaseFieldFracture {
    /// Create a new phase-field model.
    pub fn new(n: usize, length: f64, gc: f64, l0: f64, elastic_mod: f64) -> Self {
        let dx = length / (n as f64 - 1.0).max(1.0);
        Self {
            n,
            d: vec![0.0; n],
            u: vec![0.0; n],
            dx,
            gc,
            l0,
            elastic_mod,
        }
    }

    /// Set displacement boundary condition.
    pub fn set_displacement(&mut self, idx: usize, val: f64) {
        if idx < self.n {
            self.u[idx] = val;
        }
    }

    /// Compute the strain energy density at node `i`.
    pub fn strain_energy_density(&self, i: usize) -> f64 {
        if i == 0 || i + 1 >= self.n {
            return 0.0;
        }
        let eps = (self.u[i + 1] - self.u[i]) / self.dx;
        0.5 * self.elastic_mod * eps * eps
    }

    /// Update damage field using a simple explicit driving-force rule.
    #[allow(clippy::needless_range_loop)]
    pub fn update_damage(&mut self) {
        let gc = self.gc;
        let l0 = self.l0;
        let e_mod = self.elastic_mod;
        let dx = self.dx;
        let n = self.n;
        let mut d_new = self.d.clone();
        for i in 1..(n - 1) {
            let psi = self.strain_energy_density(i);
            /* driving force H = 2*psi / gc */
            let h = 2.0 * psi / gc;
            /* Laplacian of d */
            let lap = (self.d[i + 1] - 2.0 * self.d[i] + self.d[i - 1]) / (dx * dx);
            /* phase-field evolution: (2*psi/gc + 1/l0^2)*d - l0^2*lap_d = 2*psi/gc */
            let denom = h + 1.0 / (l0 * l0);
            let rhs = h + l0 * l0 * e_mod * lap;
            d_new[i] = (rhs / denom).clamp(self.d[i], 1.0); /* irreversibility */
        }
        self.d = d_new;
    }

    /// Maximum damage in the field.
    pub fn max_damage(&self) -> f64 {
        self.d.iter().cloned().fold(0.0f64, f64::max)
    }

    /// Fraction of nodes with damage above threshold.
    pub fn damage_fraction(&self, threshold: f64) -> f64 {
        let count = self.d.iter().filter(|&&v| v >= threshold).count();
        count as f64 / self.n as f64
    }

    /// Total elastic energy (degraded by (1-d)^2).
    pub fn total_elastic_energy(&self) -> f64 {
        let mut energy = 0.0;
        for i in 0..self.n.saturating_sub(1) {
            let eps = (self.u[i + 1] - self.u[i]) / self.dx;
            let deg = (1.0 - self.d[i]).powi(2);
            energy += 0.5 * self.elastic_mod * eps * eps * deg * self.dx;
        }
        energy
    }
}

/// Create a new phase-field fracture model.
pub fn new_phase_field_fracture(
    n: usize,
    length: f64,
    gc: f64,
    l0: f64,
    elastic_mod: f64,
) -> PhaseFieldFracture {
    PhaseFieldFracture::new(n, length, gc, l0, elastic_mod)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_model() -> PhaseFieldFracture {
        PhaseFieldFracture::new(5, 1.0, 1.0, 0.1, 1.0)
    }

    #[test]
    fn test_initial_damage_zero() {
        let m = simple_model();
        assert!(m.d.iter().all(|&d| d == 0.0)); /* no initial damage */
    }

    #[test]
    fn test_set_displacement() {
        let mut m = simple_model();
        m.set_displacement(4, 0.5);
        assert!((m.u[4] - 0.5).abs() < 1e-10); /* displacement set */
    }

    #[test]
    fn test_strain_energy_nonzero() {
        let mut m = simple_model();
        m.set_displacement(4, 0.1);
        let psi = m.strain_energy_density(3);
        assert!(psi > 0.0); /* energy in strained region */
    }

    #[test]
    fn test_update_damage_nonnegative() {
        let mut m = simple_model();
        m.set_displacement(4, 1.0);
        m.update_damage();
        assert!(m.d.iter().all(|&d| d >= 0.0)); /* damage non-negative */
    }

    #[test]
    fn test_damage_bounded() {
        let mut m = simple_model();
        m.set_displacement(4, 100.0);
        for _ in 0..20 {
            m.update_damage();
        }
        assert!(m.max_damage() <= 1.0); /* damage does not exceed 1 */
    }

    #[test]
    fn test_max_damage_initially_zero() {
        let m = simple_model();
        assert_eq!(m.max_damage(), 0.0); /* no damage initially */
    }

    #[test]
    fn test_damage_fraction() {
        let m = simple_model();
        assert_eq!(m.damage_fraction(0.0), 1.0); /* all nodes at or above 0 */
    }

    #[test]
    fn test_total_elastic_energy_initially_zero() {
        let m = simple_model();
        assert_eq!(m.total_elastic_energy(), 0.0); /* no displacement, no energy */
    }

    #[test]
    fn test_new_helper() {
        let m = new_phase_field_fracture(10, 2.0, 1.0, 0.1, 1.0);
        assert_eq!(m.n, 10); /* helper creates model with correct n */
    }
}
