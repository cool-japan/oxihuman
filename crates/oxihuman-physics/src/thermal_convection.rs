// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Thermal convection model (Newton's law of cooling).
#[derive(Debug, Clone)]
pub struct ThermalConvectionModel {
    pub heat_transfer_coeff: f32,
    pub fluid_temp: f32,
    pub surface_area: f32,
}

/// Create a new ThermalConvectionModel.
pub fn new_thermal_convection(h: f32, t_fluid: f32, area: f32) -> ThermalConvectionModel {
    ThermalConvectionModel {
        heat_transfer_coeff: h,
        fluid_temp: t_fluid,
        surface_area: area,
    }
}

/// Convective heat transfer rate: Q = h * A * (T_surface - T_fluid).
pub fn convective_heat_rate(m: &ThermalConvectionModel, t_surface: f32) -> f32 {
    m.heat_transfer_coeff * m.surface_area * (t_surface - m.fluid_temp)
}

/// Approximate Nusselt number using flat-plate laminar correlation.
/// Nu ≈ 0.332 * Re^0.5 * Pr^(1/3)
pub fn nusselt_approx(reynolds: f32, prandtl: f32) -> f32 {
    0.332 * reynolds.sqrt() * prandtl.powf(1.0 / 3.0)
}

/// Equilibrium surface temperature given internal heat source q_internal (W).
/// At equilibrium: h * A * (T_s - T_f) = q_internal => T_s = T_f + q / (h*A)
pub fn convection_equilibrium_temp(m: &ThermalConvectionModel, q_internal: f32) -> f32 {
    if (m.heat_transfer_coeff * m.surface_area).abs() < 1e-12 {
        return m.fluid_temp;
    }
    m.fluid_temp + q_internal / (m.heat_transfer_coeff * m.surface_area)
}

/// Thermal resistance: R = 1 / (h * A).
pub fn thermal_resistance(m: &ThermalConvectionModel) -> f32 {
    let ha = m.heat_transfer_coeff * m.surface_area;
    if ha.abs() < 1e-12 {
        f32::INFINITY
    } else {
        1.0 / ha
    }
}

/// Time constant for lumped capacitance: tau = rho * V * cp / (h * A).
pub fn lumped_time_constant(m: &ThermalConvectionModel, mass: f32, cp: f32) -> f32 {
    let ha = m.heat_transfer_coeff * m.surface_area;
    if ha.abs() < 1e-12 {
        f32::INFINITY
    } else {
        mass * cp / ha
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_thermal_convection() {
        /* constructor */
        let m = new_thermal_convection(25.0, 20.0, 2.0);
        assert!((m.heat_transfer_coeff - 25.0).abs() < 1e-9);
        assert!((m.fluid_temp - 20.0).abs() < 1e-9);
        assert!((m.surface_area - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_convective_heat_rate_positive() {
        /* hot surface -> positive heat rate */
        let m = new_thermal_convection(10.0, 20.0, 1.0);
        let q = convective_heat_rate(&m, 30.0);
        assert!((q - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_convective_heat_rate_equilibrium() {
        /* T_surface = T_fluid -> zero heat */
        let m = new_thermal_convection(10.0, 25.0, 1.0);
        let q = convective_heat_rate(&m, 25.0);
        assert!(q.abs() < 1e-6);
    }

    #[test]
    fn test_nusselt_approx() {
        /* at Re=1e4, Pr=1 -> 0.332*100 = 33.2 */
        let nu = nusselt_approx(10000.0, 1.0);
        assert!((nu - 33.2).abs() < 0.1);
    }

    #[test]
    fn test_convection_equilibrium_temp() {
        /* equilibrium with 100 W source */
        let m = new_thermal_convection(10.0, 20.0, 1.0);
        let t = convection_equilibrium_temp(&m, 100.0);
        assert!((t - 30.0).abs() < 1e-4);
    }

    #[test]
    fn test_equilibrium_zero_ha() {
        /* zero h*A returns fluid temp */
        let m = new_thermal_convection(0.0, 25.0, 0.0);
        let t = convection_equilibrium_temp(&m, 100.0);
        assert!((t - 25.0).abs() < 1e-9);
    }

    #[test]
    fn test_thermal_resistance() {
        let m = new_thermal_convection(10.0, 20.0, 0.5);
        let r = thermal_resistance(&m);
        assert!((r - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_lumped_time_constant() {
        /* tau = m*cp / (h*A) */
        let m = new_thermal_convection(10.0, 20.0, 1.0);
        let tau = lumped_time_constant(&m, 2.0, 500.0);
        assert!((tau - 100.0).abs() < 1e-3);
    }

    #[test]
    fn test_nusselt_prandtl_effect() {
        /* higher Prandtl -> higher Nu */
        let nu1 = nusselt_approx(1000.0, 1.0);
        let nu2 = nusselt_approx(1000.0, 7.0);
        assert!(nu2 > nu1);
    }
}
