// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Glomerular filtration unit.
pub struct Glomerulus {
    pub filtration_surface_area: f32,
    pub permeability: f32,
    pub hydrostatic_pressure_pa: f32,
    pub oncotic_pressure_pa: f32,
    pub bowman_pressure_pa: f32,
}

impl Glomerulus {
    pub fn new() -> Self {
        // Typical values:
        // Pgc = 55 mmHg * 133.3 Pa/mmHg ≈ 7333 Pa
        // πgc = 30 mmHg ≈ 3999 Pa
        // Pbs = 15 mmHg ≈ 2000 Pa
        Glomerulus {
            filtration_surface_area: 0.8,             // m² (total)
            permeability: 12.5e-3 / (3600.0 * 133.3), // Kf in SI
            hydrostatic_pressure_pa: 7333.0,
            oncotic_pressure_pa: 3999.0,
            bowman_pressure_pa: 2000.0,
        }
    }
}

impl Default for Glomerulus {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_glomerulus() -> Glomerulus {
    Glomerulus::new()
}

/// NFP = P_hydro - P_oncotic - P_bowman
pub fn gfr_net_filtration_pressure(g: &Glomerulus) -> f32 {
    g.hydrostatic_pressure_pa - g.oncotic_pressure_pa - g.bowman_pressure_pa
}

/// GFR = Kf * NFP  (m³/s here, multiply by 1e6 for mL/s)
pub fn gfr_filtration_rate(g: &Glomerulus) -> f32 {
    g.permeability * g.filtration_surface_area * gfr_net_filtration_pressure(g)
}

pub fn gfr_is_filtering(g: &Glomerulus) -> bool {
    gfr_net_filtration_pressure(g) > 0.0
}

pub fn gfr_update_pressure(g: &mut Glomerulus, hydrostatic: f32) {
    g.hydrostatic_pressure_pa = hydrostatic;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* new glomerulus has positive pressures */
        let g = new_glomerulus();
        assert!(g.hydrostatic_pressure_pa > 0.0);
    }

    #[test]
    fn test_nfp_positive() {
        /* NFP is positive in normal conditions */
        let g = new_glomerulus();
        assert!(gfr_net_filtration_pressure(&g) > 0.0);
    }

    #[test]
    fn test_filtration_rate_positive() {
        /* filtration rate positive when NFP > 0 */
        let g = new_glomerulus();
        assert!(gfr_filtration_rate(&g) > 0.0);
    }

    #[test]
    fn test_is_filtering_true() {
        /* normal conditions produce filtration */
        let g = new_glomerulus();
        assert!(gfr_is_filtering(&g));
    }

    #[test]
    fn test_is_filtering_false() {
        /* reduced hydrostatic stops filtration */
        let mut g = new_glomerulus();
        gfr_update_pressure(&mut g, 1000.0);
        assert!(!gfr_is_filtering(&g));
    }

    #[test]
    fn test_update_pressure() {
        /* update_pressure changes the pressure */
        let mut g = new_glomerulus();
        gfr_update_pressure(&mut g, 8000.0);
        assert!((g.hydrostatic_pressure_pa - 8000.0).abs() < 1e-3);
    }

    #[test]
    fn test_default() {
        /* Default impl works */
        let g = Glomerulus::default();
        assert!(g.filtration_surface_area > 0.0);
    }
}
