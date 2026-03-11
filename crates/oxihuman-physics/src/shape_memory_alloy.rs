// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shape Memory Alloy (SMA) spring actuator stub.

/// SMA phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmaPhase {
    Martensite,
    Austenite,
    Mixed,
}

/// An SMA spring actuator.
#[derive(Debug, Clone)]
pub struct SmaSpring {
    /// Current temperature (K).
    pub temperature: f32,
    /// Austenite start temperature (K).
    pub as_temp: f32,
    /// Austenite finish temperature (K).
    pub af_temp: f32,
    /// Martensite start temperature (K).
    pub ms_temp: f32,
    /// Martensite finish temperature (K).
    pub mf_temp: f32,
    /// Maximum recoverable strain.
    pub max_strain: f32,
    /// Current strain.
    pub strain: f32,
    /// Modulus in austenite (Pa).
    pub e_austenite: f32,
    /// Modulus in martensite (Pa).
    pub e_martensite: f32,
}

impl SmaSpring {
    pub fn new() -> Self {
        SmaSpring {
            temperature: 293.0, /* 20°C */
            as_temp: 333.0,     /* 60°C */
            af_temp: 353.0,     /* 80°C */
            ms_temp: 323.0,     /* 50°C */
            mf_temp: 303.0,     /* 30°C */
            max_strain: 0.08,
            strain: 0.08, /* start in martensite */
            e_austenite: 70e9,
            e_martensite: 30e9,
        }
    }
}

impl Default for SmaSpring {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new SMA spring actuator.
pub fn new_sma_spring() -> SmaSpring {
    SmaSpring::new()
}

/// Return the current phase based on temperature.
pub fn sma_phase(s: &SmaSpring) -> SmaPhase {
    if s.temperature >= s.af_temp {
        SmaPhase::Austenite
    } else if s.temperature <= s.mf_temp {
        SmaPhase::Martensite
    } else {
        SmaPhase::Mixed
    }
}

/// Return the martensite fraction (0 = fully austenite, 1 = fully martensite).
pub fn sma_martensite_fraction(s: &SmaSpring) -> f32 {
    if s.temperature >= s.af_temp {
        0.0
    } else if s.temperature <= s.mf_temp {
        1.0
    } else if s.temperature < s.as_temp {
        /* between mf and as: slowly cooling */
        let range = s.ms_temp - s.mf_temp;
        if range < 1e-3 {
            return 1.0;
        }
        ((s.ms_temp - s.temperature) / range).clamp(0.0, 1.0)
    } else {
        /* between as and af: heating */
        let range = s.af_temp - s.as_temp;
        if range < 1e-3 {
            return 0.0;
        }
        ((s.af_temp - s.temperature) / range).clamp(0.0, 1.0)
    }
}

/// Effective modulus based on martensite fraction.
pub fn sma_effective_modulus(s: &SmaSpring) -> f32 {
    let xi = sma_martensite_fraction(s);
    xi * s.e_martensite + (1.0 - xi) * s.e_austenite
}

/// Update strain based on temperature (shape recovery on heating).
pub fn sma_update_strain(s: &mut SmaSpring) {
    let xi = sma_martensite_fraction(s);
    s.strain = s.max_strain * xi;
}

/// Set temperature and update phase/strain.
pub fn sma_set_temperature(s: &mut SmaSpring, temp: f32) {
    s.temperature = temp.max(0.0);
    sma_update_strain(s);
}

/// Return the recovery force (N) for cross-sectional area `area` (m²).
pub fn sma_recovery_force(s: &SmaSpring, area: f32) -> f32 {
    sma_effective_modulus(s) * s.strain * area
}

/// Return `true` if the SMA is fully actuated (austenite phase).
pub fn sma_is_actuated(s: &SmaSpring) -> bool {
    sma_phase(s) == SmaPhase::Austenite
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_martensite_at_cold() {
        let s = new_sma_spring(); /* 20°C < mf=30°C */
        assert_eq!(sma_phase(&s), SmaPhase::Martensite);
    }

    #[test]
    fn test_austenite_when_hot() {
        let mut s = new_sma_spring();
        sma_set_temperature(&mut s, 400.0); /* above af=80°C */
        assert_eq!(sma_phase(&s), SmaPhase::Austenite);
    }

    #[test]
    fn test_strain_zero_when_hot() {
        let mut s = new_sma_spring();
        sma_set_temperature(&mut s, 400.0);
        assert!(s.strain < 1e-5);
    }

    #[test]
    fn test_strain_max_when_cold() {
        let s = new_sma_spring();
        assert!((s.strain - s.max_strain).abs() < 1e-5);
    }

    #[test]
    fn test_recovery_force_positive_when_strained() {
        let s = new_sma_spring();
        let f = sma_recovery_force(&s, 1e-4);
        assert!(f > 0.0);
    }

    #[test]
    fn test_martensite_fraction_zero_when_hot() {
        let mut s = new_sma_spring();
        sma_set_temperature(&mut s, 360.0);
        assert!(sma_martensite_fraction(&s) < 1e-5);
    }

    #[test]
    fn test_martensite_fraction_one_when_cold() {
        let s = new_sma_spring();
        assert!((sma_martensite_fraction(&s) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_effective_modulus_between_limits() {
        let s = new_sma_spring();
        let e = sma_effective_modulus(&s);
        assert!(e >= s.e_austenite.min(s.e_martensite));
        assert!(e <= s.e_austenite.max(s.e_martensite));
    }

    #[test]
    fn test_not_actuated_when_cold() {
        let s = new_sma_spring();
        assert!(!sma_is_actuated(&s));
    }

    #[test]
    fn test_actuated_when_hot() {
        let mut s = new_sma_spring();
        sma_set_temperature(&mut s, 400.0);
        assert!(sma_is_actuated(&s));
    }
}
