// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Topological surface state physics for magnetically doped topological insulators.
//!
//! Implements the Fu-Kane surface Dirac Hamiltonian model for a magnetically doped
//! topological insulator (TI) thin film or surface. The model captures:
//!
//! - Band inversion driven by exchange coupling to magnetic dopants
//! - Chern number topology from the Dirac mass gap
//! - Anomalous Hall conductance quantization in units of e²/h
//! - Z₂ topological invariant from Chern number parity
//!
//! ## Physical background
//!
//! On the surface of a 3D TI, the low-energy Hamiltonian is a 2D massless Dirac
//! fermion: H₀ = ħv_F (k_x σ_y − k_y σ_x). Magnetic doping breaks time-reversal
//! symmetry and opens an exchange gap Δ_m = m at the Dirac point. The full surface
//! Hamiltonian becomes:
//!
//!   H(k) = ħv_F (k_x σ_y − k_y σ_x) + m σ_z
//!
//! This is the classic 2-band Dirac model. The Chern number is C = ±1 when the
//! exchange gap dominates over the bulk hybridisation gap (band inversion condition:
//! |m| > E_g/2), and C = 0 in the trivial phase.
//!
//! The anomalous Hall conductance σ_xy = C·e²/(2h) in units where e²/h = 1 gives
//! σ_xy = 0.5·C·sign(m). The Z₂ invariant is ν = C mod 2.
//!
//! References:
//! - C.-Z. Chang et al., Science 340, 167 (2013) — QAHE in Cr-doped (Bi,Sb)₂Te₃
//! - R. Yu et al., Science 329, 61 (2010) — theoretical prediction
//! - L. Fu and C.L. Kane, PRB 76, 045302 (2007) — surface Dirac Hamiltonian

use std::f32::consts::PI;

/// Material parameters for a topological insulator surface state.
///
/// Describes a magnetically doped 3D topological insulator thin film or surface.
/// The exchange field `exchange_gap` (in eV) represents the Zeeman-like splitting
/// of the Dirac cone introduced by magnetic dopants (e.g. Cr, V, Mn) which breaks
/// time-reversal symmetry and opens an exchange gap at the Dirac point.
#[derive(Debug, Clone)]
pub struct TopoInsulatorConfig {
    /// Fermi velocity v_F of the surface Dirac cone `[m/s]`.
    pub fermi_velocity: f32,
    /// Bulk band gap E_g `[eV]`. Half of this is the critical exchange field
    /// above which band inversion occurs: |m| > E_g / 2.
    pub bulk_gap: f32,
    /// Surface state quasi-particle lifetime τ `[s]`.
    pub surface_state_lifetime: f32,
    /// Dirac point energy E_D `[eV]` (position of charge neutrality point).
    pub dirac_point_energy: f32,
    /// Exchange gap Δ_m = m `[eV]` opened at the Dirac point by magnetic doping.
    /// Positive → spin-up band below Dirac point; negative → spin-down below.
    /// When |m| > bulk_gap / 2 the system is in the topological (band-inverted) phase.
    pub exchange_gap: f32,
}

impl TopoInsulatorConfig {
    /// Create a new config with explicit parameters.
    pub fn new(
        fermi_velocity: f32,
        bulk_gap: f32,
        surface_state_lifetime: f32,
        dirac_point_energy: f32,
        exchange_gap: f32,
    ) -> Self {
        TopoInsulatorConfig {
            fermi_velocity,
            bulk_gap,
            surface_state_lifetime,
            dirac_point_energy,
            exchange_gap,
        }
    }

    /// Bi₂Te₃ approximate material parameters (non-magnetic, trivial).
    pub fn bi2te3() -> Self {
        /* Bi2Te3: v_F ≈ 4×10⁵ m/s, E_g ≈ 0.17 eV, no exchange gap */
        TopoInsulatorConfig::new(4.0e5, 0.17, 1e-12, 0.0, 0.0)
    }

    /// Bi₂Se₃ approximate material parameters (non-magnetic, trivial).
    pub fn bi2se3() -> Self {
        /* Bi2Se3: v_F ≈ 5×10⁵ m/s, E_g ≈ 0.3 eV, no exchange gap */
        TopoInsulatorConfig::new(5.0e5, 0.3, 5e-13, 0.0, 0.0)
    }

    /// Cr-doped (Bi,Sb)₂Te₃ in the quantum anomalous Hall insulator phase.
    ///
    /// Approximate parameters for the QAHE system realised by Chang et al. (2013).
    /// The exchange gap (≈ 50 meV) exceeds half the bulk gap (≈ 30 meV) so the
    /// system is band-inverted with Chern number C = 1.
    ///
    /// Note: the bulk gap of the magnetically doped film is reduced relative to
    /// the pristine TI due to inter-surface hybridisation (thin film). We use
    /// bulk_gap = 0.05 eV and exchange_gap = 0.06 eV so that exchange_gap > bulk_gap/2.
    pub fn cr_bsbite_qahe() -> Self {
        /* Cr:(Bi,Sb)2Te3 — exchange-driven QAHE phase.
        bulk_gap = 0.05 eV, exchange_gap = 0.06 eV > 0.025 eV = bulk_gap/2 */
        TopoInsulatorConfig::new(3.5e5, 0.05, 2e-13, 0.0, 0.06)
    }

    /// Determine whether the exchange field drives band inversion (topological phase).
    ///
    /// Band inversion occurs when the exchange gap exceeds half the bulk gap:
    ///   |m| > E_g / 2
    /// This is the condition for the Dirac mass to change sign relative to the
    /// bulk hybridisation gap, producing a non-trivial Chern number.
    #[inline]
    pub fn is_band_inverted(&self) -> bool {
        self.exchange_gap.abs() > self.bulk_gap * 0.5
    }
}

impl Default for TopoInsulatorConfig {
    fn default() -> Self {
        Self::bi2se3()
    }
}

/// Dirac cone energy dispersion: E(k) = ħv_F |k|.
///
/// Returns the band energy in eV for a given momentum magnitude k [m⁻¹].
/// Uses ħ = 6.582×10⁻¹⁶ eV·s.
pub fn surface_dispersion(config: &TopoInsulatorConfig, k_magnitude: f32) -> f32 {
    /* ħ in eV·s */
    let hbar_ev = 6.582e-16f32;
    hbar_ev * config.fermi_velocity * k_magnitude.abs()
}

/// Spin-momentum locking angle (helical spin texture): φ_spin = φ_k + π/2.
///
/// The spin of the surface Dirac fermion is locked perpendicular to its momentum
/// (helical texture). For momentum direction φ_k the spin direction is φ_k + π/2.
pub fn spin_angle(k_angle: f32) -> f32 {
    k_angle + PI / 2.0
}

/// Topological Z₂ invariant ν ∈ {0, 1} for the magnetically doped TI surface.
///
/// Computed from the Chern number: ν = C mod 2.
///
/// - ν = 1: topologically non-trivial (band-inverted, exchange-dominated phase)
/// - ν = 0: topologically trivial (bulk-gap-dominated phase)
///
/// ## Physics
///
/// In the single-cone Fu-Kane model the Chern number is |C| = 1 when band inversion
/// occurs. Since Z₂ ≡ C mod 2, a Chern-1 phase has ν = 1. Systems with even Chern
/// number (including C = 0) map to ν = 0.
pub fn z2_invariant(config: &TopoInsulatorConfig) -> i32 {
    let c = chern_number(config);
    /* Z2 = C mod 2 — for this single-cone model: 1 iff topologically non-trivial */
    c.unsigned_abs() as i32 % 2
}

/// Surface state density of states at energy E.
///
/// For a 2D massless Dirac fermion: DOS(E) = |E − E_D| / (π ħ² v_F²).
/// Units: states per eV per m².
pub fn surface_dos(config: &TopoInsulatorConfig, energy: f32) -> f32 {
    let hbar_ev = 6.582e-16f32;
    let vf = config.fermi_velocity;
    let e = (energy - config.dirac_point_energy).abs();
    e / (PI * hbar_ev * hbar_ev * vf * vf)
}

/// Mean free path of surface Dirac electrons: λ = v_F · τ.
pub fn mean_free_path(config: &TopoInsulatorConfig) -> f32 {
    config.fermi_velocity * config.surface_state_lifetime
}

/// Fermi wavevector at given chemical potential (measured from Dirac point).
///
/// From the linear dispersion E = ħv_F k_F:
///   k_F = μ / (ħ v_F)
pub fn fermi_wavevector(config: &TopoInsulatorConfig, chemical_potential: f32) -> f32 {
    let hbar_ev = 6.582e-16f32;
    if config.fermi_velocity <= 0.0 || hbar_ev <= 0.0 {
        return 0.0;
    }
    chemical_potential.abs() / (hbar_ev * config.fermi_velocity)
}

/// Check if energy is within the bulk gap (surface state relevant).
pub fn is_in_bulk_gap(config: &TopoInsulatorConfig, energy: f32) -> bool {
    let half_gap = config.bulk_gap / 2.0;
    let e_rel = energy - config.dirac_point_energy;
    e_rel.abs() < half_gap
}

/// Quantum anomalous Hall conductance σ_xy in units of e²/h (deprecated alias).
///
/// For the Fu-Kane magnetically gapped TI surface:
///   σ_xy = (C / 2) · sign(m)   [in units of e²/h]
///
/// where C is the Chern number and m is the exchange gap. When C = 0, σ_xy = 0.
/// When C = 1 and m > 0, σ_xy = +0.5 (half-quantised Hall plateau at QAH transition).
///
/// The half-quantisation 0.5 e²/h arises because each TI surface contributes half
/// a quantum of Hall conductance; the full integer quantisation e²/h is recovered
/// when both top and bottom surfaces are in the same topological phase.
pub fn anomalous_hall_conductance_stub(config: &TopoInsulatorConfig) -> f32 {
    let c = chern_number(config);
    if c == 0 {
        return 0.0;
    }
    /* σ_xy = 0.5 * C  in units of e²/h.
    The Chern number already encodes the sign of the exchange field:
      C = +1 for positive exchange (spin-↑ band inverted) → σ_xy = +0.5 e²/h
      C = −1 for negative exchange (time-reversed QAH) → σ_xy = −0.5 e²/h
    Each TI surface contributes half a quantum; the full e²/h quantisation
    arises when top and bottom surfaces are summed coherently. */
    0.5 * (c as f32)
}

/// Chern number C of the magnetically doped TI surface.
///
/// Computed from the 2-band Fu-Kane Dirac model on the TI surface:
///
///   H(k) = ħv_F (k_x σ_y − k_y σ_x) + m σ_z
///
/// The Berry curvature integrated over the Brillouin zone gives:
///   C = sign(m) / 2 · (1 − sign(m · (m − E_g/2)))
///
/// For the practical band-inversion criterion |m| > E_g/2:
///   C = sign(m)  if band-inverted
///   C = 0        otherwise
///
/// Here sign(m) reflects whether the magnetic exchange favours spin-up or spin-down
/// below the Dirac point. In the simplest model with a single TI surface Dirac cone,
/// |C| = 1 in the topological phase.
pub fn chern_number(config: &TopoInsulatorConfig) -> i32 {
    if config.is_band_inverted() {
        /* Chern number sign tracks the sign of the exchange field */
        if config.exchange_gap >= 0.0 {
            1
        } else {
            -1
        }
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_dispersion_zero_k() {
        let c = TopoInsulatorConfig::default();
        assert_eq!(surface_dispersion(&c, 0.0), 0.0);
    }

    #[test]
    fn test_surface_dispersion_positive_k() {
        let c = TopoInsulatorConfig::default();
        assert!(surface_dispersion(&c, 1e9) > 0.0);
    }

    #[test]
    fn test_spin_angle_offset() {
        let k_angle = 0.0f32;
        assert!((spin_angle(k_angle) - PI / 2.0).abs() < 1e-6);
    }

    /// Updated: bi2se3 has no exchange gap so z2 should be 0 (trivial).
    #[test]
    fn test_z2_invariant_trivial_for_unperturbed_bi2se3() {
        let c = TopoInsulatorConfig::default(); /* bi2se3: exchange_gap = 0 */
        assert_eq!(z2_invariant(&c), 0);
    }

    /// NEW: z2 = 1 when exchange gap exceeds half the bulk gap (band inverted).
    #[test]
    fn test_non_trivial_z2_when_inverted() {
        /* bulk_gap = 0.3 eV → threshold = 0.15 eV; exchange_gap = 0.2 eV > threshold */
        let c = TopoInsulatorConfig::new(5.0e5, 0.3, 5e-13, 0.0, 0.2);
        assert!(c.is_band_inverted(), "should be band-inverted");
        assert_eq!(z2_invariant(&c), 1, "non-trivial Z2 expected");
    }

    /// NEW: z2 = 0 when exchange gap is zero (trivial phase).
    #[test]
    fn test_trivial_z2_when_not_inverted() {
        /* exchange_gap = 0 → not band-inverted */
        let c = TopoInsulatorConfig::new(5.0e5, 0.3, 5e-13, 0.0, 0.0);
        assert!(!c.is_band_inverted(), "should NOT be band-inverted");
        assert_eq!(z2_invariant(&c), 0, "trivial Z2 expected");
    }

    /// NEW: positive exchange gap → positive anomalous Hall conductance.
    #[test]
    fn test_hall_conductance_sign() {
        /* Inverted phase with positive exchange field */
        let c = TopoInsulatorConfig::new(5.0e5, 0.3, 5e-13, 0.0, 0.2);
        let sigma = anomalous_hall_conductance_stub(&c);
        assert!(
            sigma > 0.0,
            "positive exchange gap should give positive σ_xy"
        );
        assert!((sigma - 0.5).abs() < 1e-6, "σ_xy should be 0.5 e²/h");
    }

    /// Hall conductance is zero in trivial phase.
    #[test]
    fn test_hall_conductance_zero_trivial() {
        let c = TopoInsulatorConfig::new(5.0e5, 0.3, 5e-13, 0.0, 0.0);
        assert_eq!(anomalous_hall_conductance_stub(&c), 0.0);
    }

    /// Hall conductance is negative for negative exchange gap (band-inverted).
    #[test]
    fn test_hall_conductance_negative_exchange() {
        let c = TopoInsulatorConfig::new(5.0e5, 0.3, 5e-13, 0.0, -0.2);
        let sigma = anomalous_hall_conductance_stub(&c);
        assert!(
            sigma < 0.0,
            "negative exchange gap should give negative σ_xy"
        );
        assert!((sigma + 0.5).abs() < 1e-6, "σ_xy should be -0.5 e²/h");
    }

    /// Chern number is 1 for inverted phase with positive exchange.
    #[test]
    fn test_chern_number_inverted() {
        let c = TopoInsulatorConfig::new(5.0e5, 0.3, 5e-13, 0.0, 0.2);
        assert_eq!(chern_number(&c), 1);
    }

    /// Chern number is 0 for trivial phase.
    #[test]
    fn test_chern_number_trivial() {
        let c = TopoInsulatorConfig::default();
        assert_eq!(chern_number(&c), 0);
    }

    /// Chern number is -1 for negative exchange (time-reversed topological phase).
    #[test]
    fn test_chern_number_negative_exchange() {
        let c = TopoInsulatorConfig::new(5.0e5, 0.3, 5e-13, 0.0, -0.2);
        assert_eq!(chern_number(&c), -1);
    }

    /// QAHE preset is in the non-trivial phase.
    #[test]
    fn test_qahe_preset_is_inverted() {
        let c = TopoInsulatorConfig::cr_bsbite_qahe();
        assert!(c.is_band_inverted());
        assert_eq!(chern_number(&c), 1);
    }

    #[test]
    fn test_surface_dos_positive() {
        let c = TopoInsulatorConfig::default();
        assert!(surface_dos(&c, 0.1) > 0.0);
    }

    #[test]
    fn test_surface_dos_at_dirac_point_zero() {
        let c = TopoInsulatorConfig::default();
        /* At Dirac point, DOS = 0 for linear dispersion */
        let dos = surface_dos(&c, c.dirac_point_energy);
        assert_eq!(dos, 0.0);
    }

    #[test]
    fn test_mean_free_path_positive() {
        let c = TopoInsulatorConfig::default();
        assert!(mean_free_path(&c) > 0.0);
    }

    #[test]
    fn test_fermi_wavevector_positive() {
        let c = TopoInsulatorConfig::default();
        assert!(fermi_wavevector(&c, 0.1) > 0.0);
    }

    #[test]
    fn test_is_in_bulk_gap() {
        let c = TopoInsulatorConfig::bi2se3();
        /* energy within gap */
        assert!(is_in_bulk_gap(&c, 0.05));
        /* energy outside gap */
        assert!(!is_in_bulk_gap(&c, 0.5));
    }

    /// Updated: the anomalous_hall_conductance_stub now returns physics-correct value.
    /// For QAHE preset (inverted, positive exchange), σ_xy = 0.5.
    #[test]
    fn test_anomalous_hall_conductance() {
        let c = TopoInsulatorConfig::cr_bsbite_qahe();
        assert!((anomalous_hall_conductance_stub(&c) - 0.5).abs() < 0.001);
    }

    /// Boundary case: exchange_gap exactly at inversion threshold is trivial.
    #[test]
    fn test_z2_at_inversion_boundary() {
        /* exchange_gap = bulk_gap * 0.5 exactly → NOT inverted (strict inequality) */
        let c = TopoInsulatorConfig::new(5.0e5, 0.3, 5e-13, 0.0, 0.15);
        /* 0.15 > 0.15 is false, so trivial */
        assert!(!c.is_band_inverted());
        assert_eq!(z2_invariant(&c), 0);
    }

    /// Just above threshold: topological.
    #[test]
    fn test_z2_just_above_inversion_threshold() {
        let c = TopoInsulatorConfig::new(5.0e5, 0.3, 5e-13, 0.0, 0.1501);
        assert!(c.is_band_inverted());
        assert_eq!(z2_invariant(&c), 1);
    }
}
