// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Linear Elastic Fracture Mechanics (LEFM) material property models.

/// Material properties for fracture mechanics.
#[derive(Debug, Clone)]
pub struct FractureProps {
    /// Fracture toughness K_Ic [MPa·√m].
    pub k_ic: f32,
    /// Young's modulus `[GPa]`.
    pub young_mod: f32,
    /// Poisson's ratio.
    pub poisson: f32,
}

/// Create a new FractureProps.
pub fn new_fracture_props(k_ic: f32, young_mod: f32, poisson: f32) -> FractureProps {
    FractureProps {
        k_ic,
        young_mod,
        poisson,
    }
}

/// Stress intensity factor K_I = sigma * sqrt(pi * a) * F for mode I crack.
pub fn stress_intensity_mode_i(sigma: f32, crack_half_len: f32, geometry_factor: f32) -> f32 {
    sigma * (std::f32::consts::PI * crack_half_len).sqrt() * geometry_factor
}

/// Critical crack length: a_c = (K_Ic / (sigma * F))^2 / pi.
pub fn critical_crack_length(props: &FractureProps, sigma: f32, geometry_factor: f32) -> f32 {
    if sigma < 1e-12 || geometry_factor < 1e-12 {
        return f32::MAX;
    }
    let denom = sigma * geometry_factor;
    (props.k_ic / denom).powi(2) / std::f32::consts::PI
}

/// Fracture energy G_c from K_Ic, E, nu (plane strain).
pub fn fracture_energy(props: &FractureProps) -> f32 {
    let e = props.young_mod * 1e9;
    props.k_ic * props.k_ic * (1.0 - props.poisson * props.poisson) / e
}

/// Determine if fracture occurs: K_I >= K_Ic.
pub fn is_fracture_critical(k_i: f32, props: &FractureProps) -> bool {
    k_i >= props.k_ic
}

/// J-integral (simplified) = G_c for linear elastic case.
pub fn j_integral(props: &FractureProps) -> f32 {
    fracture_energy(props)
}

/// Crack growth rate per cycle (Paris law): da/dN = C * (delta_K)^m.
pub fn paris_law(c: f32, delta_k: f32, m: f32) -> f32 {
    c * delta_k.powf(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fracture_props() {
        /* constructor stores values */
        let p = new_fracture_props(50.0, 200.0, 0.3);
        assert!((p.k_ic - 50.0).abs() < 1e-5);
        assert!((p.young_mod - 200.0).abs() < 1e-5);
    }

    #[test]
    fn test_stress_intensity_mode_i() {
        /* K_I > 0 for positive inputs */
        let k = stress_intensity_mode_i(100.0, 0.01, 1.0);
        assert!(k > 0.0);
    }

    #[test]
    fn test_critical_crack_length_positive() {
        /* a_c > 0 for valid inputs */
        let p = new_fracture_props(50.0, 200.0, 0.3);
        let a = critical_crack_length(&p, 300.0, 1.0);
        assert!(a > 0.0);
    }

    #[test]
    fn test_fracture_energy_positive() {
        /* G_c > 0 */
        let p = new_fracture_props(50.0, 200.0, 0.3);
        let g = fracture_energy(&p);
        assert!(g > 0.0);
    }

    #[test]
    fn test_is_fracture_critical_true() {
        /* K_I >= K_Ic -> fracture */
        let p = new_fracture_props(50.0, 200.0, 0.3);
        assert!(is_fracture_critical(60.0, &p));
    }

    #[test]
    fn test_is_fracture_critical_false() {
        /* K_I < K_Ic -> no fracture */
        let p = new_fracture_props(50.0, 200.0, 0.3);
        assert!(!is_fracture_critical(30.0, &p));
    }

    #[test]
    fn test_paris_law_positive() {
        /* crack growth rate > 0 */
        let rate = paris_law(1e-12, 30.0, 3.0);
        assert!(rate > 0.0);
    }

    #[test]
    fn test_j_integral_equals_fracture_energy() {
        /* J = G_c for linear elastic */
        let p = new_fracture_props(50.0, 200.0, 0.3);
        assert!((j_integral(&p) - fracture_energy(&p)).abs() < 1e-20);
    }
}
