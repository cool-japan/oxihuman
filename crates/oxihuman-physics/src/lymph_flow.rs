// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Lymphatic capillary (Starling forces model).
pub struct LymphCapillary {
    pub hydrostatic_pressure: f32,
    pub oncotic_pressure: f32,
    pub permeability: f32,
    pub length: f32,
    pub radius: f32,
}

impl LymphCapillary {
    pub fn new() -> Self {
        LymphCapillary {
            hydrostatic_pressure: 15.0, // mmHg
            oncotic_pressure: 25.0,     // mmHg
            permeability: 1e-8,         // m/(s·Pa)
            length: 1e-3,               // 1 mm
            radius: 5e-6,               // 5 μm
        }
    }
}

impl Default for LymphCapillary {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_lymph_capillary() -> LymphCapillary {
    LymphCapillary::new()
}

/// NFP = (Pc - Pi) - (πc - πi)
pub fn lymph_net_filtration_pressure(
    c: &LymphCapillary,
    interstitial_hp: f32,
    interstitial_op: f32,
) -> f32 {
    (c.hydrostatic_pressure - interstitial_hp) - (c.oncotic_pressure - interstitial_op)
}

/// Surface area of the capillary cylinder: 2π r L
pub fn lymph_surface_area(c: &LymphCapillary) -> f32 {
    2.0 * PI * c.radius * c.length
}

/// Q = Lp * A * NFP
pub fn lymph_flow_rate(c: &LymphCapillary, nfp: f32) -> f32 {
    c.permeability * lymph_surface_area(c) * nfp
}

/// True when fluid is absorbed (NFP < 0).
pub fn lymph_is_absorbing(nfp: f32) -> bool {
    nfp < 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* new capillary has positive pressure */
        let c = new_lymph_capillary();
        assert!(c.hydrostatic_pressure > 0.0);
    }

    #[test]
    fn test_nfp_filtration() {
        /* default conditions: outward filtration */
        let c = new_lymph_capillary();
        let nfp = lymph_net_filtration_pressure(&c, 0.0, 5.0);
        /* Pc=15, Pi=0, πc=25, πi=5 => (15-0)-(25-5)=15-20=-5 */
        assert!((nfp - (-5.0)).abs() < 1e-4);
    }

    #[test]
    fn test_surface_area_positive() {
        /* surface area is positive */
        let c = new_lymph_capillary();
        assert!(lymph_surface_area(&c) > 0.0);
    }

    #[test]
    fn test_flow_rate_sign() {
        /* positive NFP gives positive flow */
        let c = new_lymph_capillary();
        let flow = lymph_flow_rate(&c, 5.0);
        assert!(flow > 0.0);
    }

    #[test]
    fn test_is_absorbing_true() {
        /* negative NFP indicates absorption */
        assert!(lymph_is_absorbing(-2.0));
    }

    #[test]
    fn test_is_absorbing_false() {
        /* positive NFP is not absorbing */
        assert!(!lymph_is_absorbing(2.0));
    }

    #[test]
    fn test_default_impl() {
        /* Default impl works */
        let c = LymphCapillary::default();
        assert!(c.radius > 0.0);
    }
}
