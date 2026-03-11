// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Negative Poisson ratio (auxetic) material stub.
//!
//! Models re-entrant honeycomb structures that exhibit a negative Poisson ratio,
//! meaning they expand laterally under tension.

/// Parameters for an auxetic re-entrant honeycomb unit cell.
#[derive(Debug, Clone)]
pub struct AuxeticParams {
    /// Rib length `[m]`.
    pub rib_length: f64,
    /// Re-entrant angle θ `[degrees]` (typically 0–45°).
    pub reentrant_angle_deg: f64,
    /// Rib thickness `[m]`.
    pub rib_thickness: f64,
    /// Base material Young's modulus `[GPa]`.
    pub base_modulus: f64,
    /// Base material Poisson ratio.
    pub base_poisson: f64,
}

impl Default for AuxeticParams {
    fn default() -> Self {
        Self {
            rib_length: 0.01,
            reentrant_angle_deg: 30.0,
            rib_thickness: 0.001,
            base_modulus: 70.0,
            base_poisson: 0.33,
        }
    }
}

/// Effective properties of an auxetic structure.
#[derive(Debug, Clone)]
pub struct AuxeticProperties {
    pub poisson_ratio_xy: f64,
    pub young_modulus_x: f64,
    pub young_modulus_y: f64,
    pub relative_density: f64,
}

/// Compute the effective Poisson ratio of a re-entrant honeycomb.
///
/// ν_xy = -(sin θ)(h/l + sin θ) / cos²θ  (Gibson-Ashby formula)
pub fn effective_poisson_ratio(params: &AuxeticParams, h_over_l: f64) -> f64 {
    let theta = params.reentrant_angle_deg.to_radians();
    let num = -(theta.sin()) * (h_over_l + theta.sin());
    let den = theta.cos() * theta.cos();
    if den.abs() < 1e-12 {
        return -1.0;
    }
    num / den
}

/// Compute effective Young's modulus in x-direction.
pub fn effective_modulus_x(params: &AuxeticParams, h_over_l: f64) -> f64 {
    let theta = params.reentrant_angle_deg.to_radians();
    let t_over_l = params.rib_thickness / params.rib_length;
    let e_s = params.base_modulus * 1e9;
    e_s * t_over_l.powi(3) * theta.cos() / ((h_over_l + theta.sin()) * theta.sin().powi(2))
}

/// Compute relative density of the honeycomb.
pub fn relative_density(params: &AuxeticParams, h_over_l: f64) -> f64 {
    let theta = params.reentrant_angle_deg.to_radians();
    let t_over_l = params.rib_thickness / params.rib_length;
    t_over_l * (h_over_l + 2.0) / (2.0 * theta.cos() * (h_over_l + theta.sin()))
}

/// Evaluate all effective auxetic properties at once.
pub fn compute_auxetic_properties(params: &AuxeticParams, h_over_l: f64) -> AuxeticProperties {
    let nu_xy = effective_poisson_ratio(params, h_over_l);
    let e_x = effective_modulus_x(params, h_over_l);
    let rho = relative_density(params, h_over_l);
    AuxeticProperties {
        poisson_ratio_xy: nu_xy,
        young_modulus_x: e_x,
        young_modulus_y: e_x * nu_xy.abs(), /* Approximate symmetry */
        relative_density: rho,
    }
}

/// Compute the axial strain when an auxetic material is stretched.
pub fn lateral_strain(axial_strain: f64, poisson_ratio: f64) -> f64 {
    -poisson_ratio * axial_strain
}

/// Check if the Poisson ratio is negative (auxetic behaviour).
pub fn is_auxetic(poisson_ratio: f64) -> bool {
    poisson_ratio < 0.0
}

/// Estimate the impact absorption improvement factor vs. conventional foam.
pub fn impact_absorption_factor(poisson_ratio: f64) -> f64 {
    /* Larger negative Poisson → more synclastic curvature → better energy absorption */
    1.0 + poisson_ratio.abs()
}

/// Compute the anisotropy ratio E_x / E_y.
pub fn anisotropy_ratio(props: &AuxeticProperties) -> f64 {
    if props.young_modulus_y <= 0.0 {
        return f64::INFINITY;
    }
    props.young_modulus_x / props.young_modulus_y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> AuxeticParams {
        AuxeticParams::default()
    }

    #[test]
    fn test_poisson_ratio_negative() {
        let nu = effective_poisson_ratio(&default_params(), 2.0);
        assert!(is_auxetic(nu), "Expected negative Poisson ratio, got {nu}");
    }

    #[test]
    fn test_effective_modulus_positive() {
        let e = effective_modulus_x(&default_params(), 2.0);
        assert!(e > 0.0);
    }

    #[test]
    fn test_relative_density_in_range() {
        let rho = relative_density(&default_params(), 2.0);
        assert!((0.0..=1.0).contains(&rho));
    }

    #[test]
    fn test_compute_properties() {
        let props = compute_auxetic_properties(&default_params(), 2.0);
        assert!(is_auxetic(props.poisson_ratio_xy));
    }

    #[test]
    fn test_lateral_strain_sign() {
        /* Auxetic: negative Poisson → positive lateral strain under tension */
        let eps_lat = lateral_strain(0.01, -0.5);
        assert!(eps_lat > 0.0);
    }

    #[test]
    fn test_is_auxetic_false_for_positive() {
        assert!(!is_auxetic(0.3));
    }

    #[test]
    fn test_impact_absorption_factor_gt_one() {
        assert!(impact_absorption_factor(-0.5) > 1.0);
    }

    #[test]
    fn test_anisotropy_ratio_finite() {
        let props = compute_auxetic_properties(&default_params(), 2.0);
        let ratio = anisotropy_ratio(&props);
        assert!(ratio.is_finite());
    }

    #[test]
    fn test_default_params() {
        let p = default_params();
        assert!(p.reentrant_angle_deg > 0.0 && p.reentrant_angle_deg < 90.0);
    }
}
