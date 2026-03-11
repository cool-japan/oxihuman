// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

const R_GAS: f32 = 8.314; // J/(mol·K)

pub struct OsmoticSolution {
    pub concentration_mol_per_l: f32,
    pub temperature_k: f32,
    pub vant_hoff_factor: f32,
}

impl OsmoticSolution {
    pub fn new(conc: f32, temp_k: f32) -> Self {
        OsmoticSolution {
            concentration_mol_per_l: conc,
            temperature_k: temp_k,
            vant_hoff_factor: 1.0,
        }
    }
}

pub fn new_osmotic_solution(conc: f32, temp_k: f32) -> OsmoticSolution {
    OsmoticSolution::new(conc, temp_k)
}

/// π = i C R T  (concentration in mol/L → mol/m³ by *1000)
pub fn osmotic_pressure_pa(s: &OsmoticSolution) -> f32 {
    s.vant_hoff_factor * s.concentration_mol_per_l * 1000.0 * R_GAS * s.temperature_k
}

pub fn osmotic_is_hypertonic(a: &OsmoticSolution, b: &OsmoticSolution) -> bool {
    osmotic_pressure_pa(a) > osmotic_pressure_pa(b)
}

/// Positive value indicates net flow from `low` to `high` (into `high`).
pub fn osmotic_flow_direction(high: &OsmoticSolution, low: &OsmoticSolution) -> f32 {
    osmotic_pressure_pa(high) - osmotic_pressure_pa(low)
}

/// Equilibrium concentration after mixing two compartments.
pub fn osmotic_equilibrium_concentration(
    a: &OsmoticSolution,
    vol_a: f32,
    b: &OsmoticSolution,
    vol_b: f32,
) -> f32 {
    let total_vol = vol_a + vol_b;
    if total_vol <= 0.0 {
        return 0.0;
    }
    (a.concentration_mol_per_l * vol_a + b.concentration_mol_per_l * vol_b) / total_vol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* new solution has i=1 */
        let s = new_osmotic_solution(0.1, 298.0);
        assert!((s.vant_hoff_factor - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pressure_positive() {
        /* osmotic pressure is positive */
        let s = new_osmotic_solution(0.1, 298.0);
        assert!(osmotic_pressure_pa(&s) > 0.0);
    }

    #[test]
    fn test_hypertonic() {
        /* higher concentration is hypertonic */
        let a = new_osmotic_solution(0.3, 298.0);
        let b = new_osmotic_solution(0.1, 298.0);
        assert!(osmotic_is_hypertonic(&a, &b));
        assert!(!osmotic_is_hypertonic(&b, &a));
    }

    #[test]
    fn test_flow_direction_positive() {
        /* flow direction into high-concentration side is positive */
        let high = new_osmotic_solution(0.3, 298.0);
        let low = new_osmotic_solution(0.1, 298.0);
        assert!(osmotic_flow_direction(&high, &low) > 0.0);
    }

    #[test]
    fn test_equilibrium_equal() {
        /* equal concentrations stay equal */
        let a = new_osmotic_solution(0.2, 298.0);
        let b = new_osmotic_solution(0.2, 298.0);
        let eq = osmotic_equilibrium_concentration(&a, 1.0, &b, 1.0);
        assert!((eq - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_equilibrium_mixing() {
        /* mixing 0.1 and 0.3 in equal volumes gives 0.2 */
        let a = new_osmotic_solution(0.1, 298.0);
        let b = new_osmotic_solution(0.3, 298.0);
        let eq = osmotic_equilibrium_concentration(&a, 1.0, &b, 1.0);
        assert!((eq - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_pressure_scales_with_concentration() {
        /* doubling concentration doubles pressure */
        let s1 = new_osmotic_solution(0.1, 298.0);
        let s2 = new_osmotic_solution(0.2, 298.0);
        let ratio = osmotic_pressure_pa(&s2) / osmotic_pressure_pa(&s1);
        assert!((ratio - 2.0).abs() < 1e-4);
    }
}
