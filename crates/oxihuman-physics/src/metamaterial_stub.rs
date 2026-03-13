// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Acoustic/mechanical metamaterial stub.
//!
//! Models locally-resonant and Bragg-scattering metamaterials that exhibit
//! bandgaps in acoustic or elastic wave transmission.

use std::f64::consts::PI;

/// Type of metamaterial unit cell.
#[derive(Debug, Clone, PartialEq)]
pub enum MetamaterialKind {
    /// Bragg scattering (periodic structure).
    BraggScattering,
    /// Locally resonant (mass-in-mass resonators).
    LocallyResonant,
    /// Pentamode (near-fluid behaviour).
    Pentamode,
}

/// Parameters for a 1D locally-resonant metamaterial unit cell.
#[derive(Debug, Clone)]
pub struct MetamaterialParams {
    pub kind: MetamaterialKind,
    /// Lattice constant / unit cell size `[m]`.
    pub lattice_constant: f64,
    /// Host matrix density [kg/m³].
    pub matrix_density: f64,
    /// Host matrix Young's modulus `[Pa]`.
    pub matrix_modulus: f64,
    /// Inner resonator mass `[kg]`.
    pub resonator_mass: f64,
    /// Resonator spring constant [N/m].
    pub resonator_spring: f64,
}

impl Default for MetamaterialParams {
    fn default() -> Self {
        Self {
            kind: MetamaterialKind::LocallyResonant,
            lattice_constant: 0.01,
            matrix_density: 2700.0,
            matrix_modulus: 70e9,
            resonator_mass: 0.001,
            resonator_spring: 1e4,
        }
    }
}

/// Bandgap information for a metamaterial.
#[derive(Debug, Clone)]
pub struct BandgapInfo {
    pub lower_freq_hz: f64,
    pub upper_freq_hz: f64,
    pub is_complete: bool,
}

impl BandgapInfo {
    pub fn width_hz(&self) -> f64 {
        self.upper_freq_hz - self.lower_freq_hz
    }

    pub fn center_freq_hz(&self) -> f64 {
        (self.lower_freq_hz + self.upper_freq_hz) * 0.5
    }

    pub fn contains_frequency(&self, freq: f64) -> bool {
        freq >= self.lower_freq_hz && freq <= self.upper_freq_hz
    }
}

/// Compute the resonance frequency of the inner mass-spring resonator.
pub fn resonator_frequency_hz(params: &MetamaterialParams) -> f64 {
    if params.resonator_mass <= 0.0 {
        return 0.0;
    }
    (params.resonator_spring / params.resonator_mass).sqrt() / (2.0 * PI)
}

/// Estimate the locally-resonant bandgap around the resonance frequency.
pub fn locally_resonant_bandgap(params: &MetamaterialParams) -> BandgapInfo {
    let f0 = resonator_frequency_hz(params);
    /* Bandgap width estimate from coupled oscillator theory */
    let mass_ratio = params.resonator_mass
        / (params.matrix_density * params.lattice_constant.powi(3)).max(1e-30);
    let half_width = f0 * mass_ratio.sqrt() * 0.5;
    BandgapInfo {
        lower_freq_hz: (f0 - half_width).max(0.0),
        upper_freq_hz: f0 + half_width,
        is_complete: true,
    }
}

/// Compute the Bragg frequency for a 1D periodic structure.
///
/// f_Bragg = c / (2 * a), where c is wave speed and a is lattice constant.
pub fn bragg_frequency_hz(params: &MetamaterialParams) -> f64 {
    let c = wave_speed(params);
    c / (2.0 * params.lattice_constant)
}

/// Estimate the longitudinal wave speed in the host matrix.
pub fn wave_speed(params: &MetamaterialParams) -> f64 {
    (params.matrix_modulus / params.matrix_density).sqrt()
}

/// Compute the transmission loss through N unit cells at a given frequency.
///
/// Simplified model: TL increases with number of cells in the bandgap.
pub fn transmission_loss_db(freq_hz: f64, n_cells: usize, params: &MetamaterialParams) -> f64 {
    let gap = locally_resonant_bandgap(params);
    if gap.contains_frequency(freq_hz) {
        n_cells as f64 * 10.0 /* 10 dB per cell (stub estimate) */
    } else {
        0.0
    }
}

/// Check whether a frequency falls in any known bandgap.
pub fn is_in_bandgap(freq_hz: f64, params: &MetamaterialParams) -> bool {
    locally_resonant_bandgap(params).contains_frequency(freq_hz)
}

/// Compute the effective mass density at a given frequency (dynamic mass).
///
/// ρ_eff = ρ_host * [1 - f²_res / (f²_res - f²)]
pub fn effective_mass_density(freq_hz: f64, params: &MetamaterialParams) -> f64 {
    let f0 = resonator_frequency_hz(params);
    let f02 = f0 * f0;
    let f2 = freq_hz * freq_hz;
    if (f02 - f2).abs() < 1e-30 {
        return f64::INFINITY;
    }
    params.matrix_density * (1.0 - f02 / (f02 - f2))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> MetamaterialParams {
        MetamaterialParams::default()
    }

    #[test]
    fn test_resonator_frequency_positive() {
        assert!(resonator_frequency_hz(&default_params()) > 0.0);
    }

    #[test]
    fn test_wave_speed_positive() {
        assert!(wave_speed(&default_params()) > 0.0);
    }

    #[test]
    fn test_bragg_frequency_positive() {
        assert!(bragg_frequency_hz(&default_params()) > 0.0);
    }

    #[test]
    fn test_bandgap_contains_resonance() {
        let p = default_params();
        let f0 = resonator_frequency_hz(&p);
        let gap = locally_resonant_bandgap(&p);
        assert!(gap.contains_frequency(f0));
    }

    #[test]
    fn test_bandgap_width_positive() {
        let gap = locally_resonant_bandgap(&default_params());
        assert!(gap.width_hz() > 0.0);
    }

    #[test]
    fn test_transmission_loss_in_gap() {
        let p = default_params();
        let f0 = resonator_frequency_hz(&p);
        let tl = transmission_loss_db(f0, 5, &p);
        assert!(tl > 0.0);
    }

    #[test]
    fn test_transmission_loss_outside_gap() {
        let p = default_params();
        let tl = transmission_loss_db(1e6, 5, &p); /* Far outside gap */
        assert_eq!(tl, 0.0);
    }

    #[test]
    fn test_is_in_bandgap() {
        let p = default_params();
        let f0 = resonator_frequency_hz(&p);
        assert!(is_in_bandgap(f0, &p));
    }

    #[test]
    fn test_effective_mass_density_finite_off_resonance() {
        let p = default_params();
        let f0 = resonator_frequency_hz(&p);
        let rho_eff = effective_mass_density(f0 * 2.0, &p);
        assert!(rho_eff.is_finite());
    }
}
