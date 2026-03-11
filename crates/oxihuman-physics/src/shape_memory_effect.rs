// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shape memory alloy (SMA) stub.

/// Phase of the SMA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmaPhase {
    Austenite,
    Martensite,
    Mixed,
}

/// SMA material parameters.
#[derive(Debug, Clone)]
pub struct SmaConfig {
    /// Austenite start/finish temperatures `[K]`.
    pub as_temp: f32,
    pub af_temp: f32,
    /// Martensite start/finish temperatures `[K]`.
    pub ms_temp: f32,
    pub mf_temp: f32,
    /// Maximum recoverable strain.
    pub max_strain: f32,
    pub young_austenite: f32,
    pub young_martensite: f32,
}

impl SmaConfig {
    pub fn new(
        as_temp: f32,
        af_temp: f32,
        ms_temp: f32,
        mf_temp: f32,
        max_strain: f32,
        young_austenite: f32,
        young_martensite: f32,
    ) -> Self {
        SmaConfig {
            as_temp,
            af_temp,
            ms_temp,
            mf_temp,
            max_strain,
            young_austenite,
            young_martensite,
        }
    }

    pub fn nitinol() -> Self {
        SmaConfig::new(268.15, 281.15, 258.15, 233.15, 0.08, 70e9, 30e9)
    }
}

impl Default for SmaConfig {
    fn default() -> Self {
        Self::nitinol()
    }
}

/// Martensite volume fraction from temperature (simplified Koistinen-Marburger).
pub fn martensite_fraction(config: &SmaConfig, temperature: f32) -> f32 {
    if temperature >= config.ms_temp {
        0.0
    } else if temperature <= config.mf_temp {
        1.0
    } else {
        (config.ms_temp - temperature) / (config.ms_temp - config.mf_temp).max(1e-12)
    }
}

/// Austenite volume fraction.
pub fn austenite_fraction(config: &SmaConfig, temperature: f32) -> f32 {
    1.0 - martensite_fraction(config, temperature)
}

/// Current phase.
pub fn current_phase(config: &SmaConfig, temperature: f32) -> SmaPhase {
    let xi = martensite_fraction(config, temperature);
    if xi < 0.01 {
        SmaPhase::Austenite
    } else if xi > 0.99 {
        SmaPhase::Martensite
    } else {
        SmaPhase::Mixed
    }
}

/// Effective Young's modulus (rule of mixtures).
pub fn effective_modulus(config: &SmaConfig, temperature: f32) -> f32 {
    let xi = martensite_fraction(config, temperature);
    xi * config.young_martensite + (1.0 - xi) * config.young_austenite
}

/// Recoverable strain at given temperature and applied stress.
pub fn recoverable_strain(config: &SmaConfig, temperature: f32, stress: f32) -> f32 {
    let xi = martensite_fraction(config, temperature);
    let e = effective_modulus(config, temperature);
    let elastic_strain = if e > 0.0 { stress / e } else { 0.0 };
    let sme_strain = xi * config.max_strain;
    (elastic_strain + sme_strain).min(config.max_strain)
}

/// Recovery force at fixed strain `eps0` upon heating.
pub fn recovery_force(config: &SmaConfig, temperature: f32, eps0: f32, area: f32) -> f32 {
    let e = effective_modulus(config, temperature);
    let xi = martensite_fraction(config, temperature);
    let sme = xi * config.max_strain;
    let delta = (eps0 - sme).max(0.0);
    e * delta * area.max(0.0)
}

/// Is the SMA above the full-austenite temperature?
pub fn is_fully_austenite(config: &SmaConfig, temperature: f32) -> bool {
    temperature >= config.af_temp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_martensite_fraction_above_ms() {
        let c = SmaConfig::nitinol();
        assert_eq!(martensite_fraction(&c, c.ms_temp + 10.0), 0.0);
    }

    #[test]
    fn test_martensite_fraction_below_mf() {
        let c = SmaConfig::nitinol();
        assert_eq!(martensite_fraction(&c, c.mf_temp - 10.0), 1.0);
    }

    #[test]
    fn test_martensite_fraction_intermediate() {
        let c = SmaConfig::nitinol();
        let mid = (c.ms_temp + c.mf_temp) / 2.0;
        let xi = martensite_fraction(&c, mid);
        assert!((0.0..=1.0).contains(&xi));
    }

    #[test]
    fn test_austenite_plus_martensite_equals_one() {
        let c = SmaConfig::nitinol();
        let t = 250.0;
        let sum = martensite_fraction(&c, t) + austenite_fraction(&c, t);
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_phase_classification() {
        let c = SmaConfig::nitinol();
        assert_eq!(current_phase(&c, c.ms_temp + 20.0), SmaPhase::Austenite);
        assert_eq!(current_phase(&c, c.mf_temp - 20.0), SmaPhase::Martensite);
    }

    #[test]
    fn test_effective_modulus_bounds() {
        let c = SmaConfig::nitinol();
        let e_high = effective_modulus(&c, c.ms_temp + 10.0);
        let e_low = effective_modulus(&c, c.mf_temp - 10.0);
        /* austenite stiffer than martensite for Nitinol */
        assert!(e_high > e_low);
    }

    #[test]
    fn test_recoverable_strain_bounded() {
        let c = SmaConfig::nitinol();
        let eps = recoverable_strain(&c, 240.0, 100e6);
        assert!((0.0..=c.max_strain + 0.001).contains(&eps));
    }

    #[test]
    fn test_recovery_force_positive() {
        let c = SmaConfig::nitinol();
        /* heat to austenite: xi small => restoring force */
        let f = recovery_force(&c, c.af_temp + 10.0, 0.05, 1e-4);
        assert!(f >= 0.0);
    }

    #[test]
    fn test_is_fully_austenite() {
        let c = SmaConfig::nitinol();
        assert!(is_fully_austenite(&c, c.af_temp + 5.0));
        assert!(!is_fully_austenite(&c, c.af_temp - 5.0));
    }

    #[test]
    fn test_nitinol_default() {
        let c = SmaConfig::nitinol();
        assert!(c.max_strain > 0.0);
    }
}
