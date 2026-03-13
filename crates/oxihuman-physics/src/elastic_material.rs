// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Elastic material model: stress/strain computation for 2D plane-stress problems.
//!
//! Implements linear isotropic elasticity using the Lamé parameters derived
//! from Young's modulus and Poisson's ratio. Supports computation of
//! stress from strain, von Mises stress, and the plane-stress constitutive matrix.

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Material constants for a linear elastic solid.
pub struct ElasticConfig {
    /// Young's modulus (stiffness, Pa).
    pub youngs_modulus: f32,
    /// Poisson's ratio (dimensionless, 0 < ν < 0.5).
    pub poissons_ratio: f32,
    /// Mass density (kg/m³).
    pub density: f32,
    /// Rayleigh-type damping coefficient.
    pub damping: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
/// 2D stress state (plane stress).
pub struct StressState {
    /// Normal stress in X.
    pub sigma_xx: f32,
    /// Normal stress in Y.
    pub sigma_yy: f32,
    /// Shear stress XY.
    pub sigma_xy: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
/// 2D strain state (engineering strains).
pub struct StrainState {
    /// Normal strain in X.
    pub epsilon_xx: f32,
    /// Normal strain in Y.
    pub epsilon_yy: f32,
    /// Engineering shear strain XY.
    pub epsilon_xy: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Bundled elastic computation result.
pub struct ElasticResult {
    pub stress: StressState,
    pub strain: StrainState,
    /// Von Mises equivalent stress.
    pub von_mises: f32,
}

// ─── Lamé parameters ────────────────────────────────────────────────────────

/// Compute Lamé's first parameter λ (plane-strain form).
#[allow(dead_code)]
pub fn lame_lambda(cfg: &ElasticConfig) -> f32 {
    let e = cfg.youngs_modulus;
    let nu = cfg.poissons_ratio;
    (e * nu) / ((1.0 + nu) * (1.0 - 2.0 * nu))
}

/// Compute Lamé's shear modulus μ (second parameter).
#[allow(dead_code)]
pub fn lame_mu(cfg: &ElasticConfig) -> f32 {
    cfg.youngs_modulus / (2.0 * (1.0 + cfg.poissons_ratio))
}

// ─── Core computations ──────────────────────────────────────────────────────

/// Return default elastic material configuration (rubber-like).
#[allow(dead_code)]
pub fn default_elastic_config() -> ElasticConfig {
    ElasticConfig {
        youngs_modulus: 1e6,   // 1 MPa
        poissons_ratio: 0.3,
        density: 1000.0,
        damping: 0.01,
    }
}

/// Compute engineering strain from a 2D displacement vector and element size `h`.
///
/// Uses a simplified central-difference approximation:
///   ε_xx = disp[0] / h,  ε_yy = disp[1] / h,  ε_xy = (disp[0]+disp[1]) / (2·h)
#[allow(dead_code)]
pub fn compute_strain(disp: [f32; 2], h: f32) -> StrainState {
    let inv_h = if h.abs() > 1e-12 { 1.0 / h } else { 0.0 };
    StrainState {
        epsilon_xx: disp[0] * inv_h,
        epsilon_yy: disp[1] * inv_h,
        epsilon_xy: (disp[0] + disp[1]) * inv_h * 0.5,
    }
}

/// Compute plane-stress σ from ε using the isotropic constitutive law.
///
/// σ_xx = E/(1-ν²) · (ε_xx + ν·ε_yy)
/// σ_yy = E/(1-ν²) · (ν·ε_xx + ε_yy)
/// σ_xy = E/(1+ν) · ε_xy
#[allow(dead_code)]
pub fn compute_stress(strain: &StrainState, cfg: &ElasticConfig) -> StressState {
    let e = cfg.youngs_modulus;
    let nu = cfg.poissons_ratio;
    let factor = e / (1.0 - nu * nu);
    StressState {
        sigma_xx: factor * (strain.epsilon_xx + nu * strain.epsilon_yy),
        sigma_yy: factor * (nu * strain.epsilon_xx + strain.epsilon_yy),
        sigma_xy: (e / (1.0 + nu)) * strain.epsilon_xy,
    }
}

/// Compute von Mises equivalent stress for plane-stress state.
///
/// σ_vm = √(σ_xx² − σ_xx·σ_yy + σ_yy² + 3·σ_xy²)
#[allow(dead_code)]
pub fn von_mises_stress(s: &StressState) -> f32 {
    let val = s.sigma_xx * s.sigma_xx
        - s.sigma_xx * s.sigma_yy
        + s.sigma_yy * s.sigma_yy
        + 3.0 * s.sigma_xy * s.sigma_xy;
    val.max(0.0).sqrt()
}

/// Build the 3×3 plane-stress constitutive matrix D.
///
/// D = E/(1-ν²) · [[1, ν, 0], [ν, 1, 0], [0, 0, (1-ν)/2]]
#[allow(dead_code)]
pub fn plane_stress_matrix(cfg: &ElasticConfig) -> [[f32; 3]; 3] {
    let e = cfg.youngs_modulus;
    let nu = cfg.poissons_ratio;
    let c = e / (1.0 - nu * nu);
    [
        [c,        c * nu,         0.0],
        [c * nu,   c,              0.0],
        [0.0,      0.0,  c * (1.0 - nu) / 2.0],
    ]
}

/// Run a full elastic computation and return the bundled result.
#[allow(dead_code)]
pub fn elastic_result_to_json(r: &ElasticResult) -> String {
    format!(
        "{{\"sigma_xx\":{:.6},\"sigma_yy\":{:.6},\"sigma_xy\":{:.6},\
         \"epsilon_xx\":{:.6},\"epsilon_yy\":{:.6},\"epsilon_xy\":{:.6},\
         \"von_mises\":{:.6}}}",
        r.stress.sigma_xx,
        r.stress.sigma_yy,
        r.stress.sigma_xy,
        r.strain.epsilon_xx,
        r.strain.epsilon_yy,
        r.strain.epsilon_xy,
        r.von_mises
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_elastic_config();
        assert!((cfg.youngs_modulus - 1e6).abs() < 1.0);
        assert!((cfg.poissons_ratio - 0.3).abs() < 1e-6);
        assert!((cfg.density - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn test_lame_mu() {
        let cfg = default_elastic_config();
        let mu = lame_mu(&cfg);
        // E=1e6, nu=0.3 → mu = 1e6/(2*1.3) ≈ 384615
        assert!((mu - 384_615.4).abs() < 1.0);
    }

    #[test]
    fn test_lame_lambda() {
        let cfg = default_elastic_config();
        let lambda = lame_lambda(&cfg);
        // E=1e6, nu=0.3 → lambda = 1e6*0.3/(1.3*0.4) = 576923
        assert!((lambda - 576_923.0).abs() < 1.0);
    }

    #[test]
    fn test_compute_strain_zero_disp() {
        let strain = compute_strain([0.0, 0.0], 1.0);
        assert!((strain.epsilon_xx).abs() < 1e-9);
        assert!((strain.epsilon_yy).abs() < 1e-9);
        assert!((strain.epsilon_xy).abs() < 1e-9);
    }

    #[test]
    fn test_compute_strain_unit_x() {
        let strain = compute_strain([1.0, 0.0], 1.0);
        assert!((strain.epsilon_xx - 1.0).abs() < 1e-6);
        assert!((strain.epsilon_yy).abs() < 1e-6);
        assert!((strain.epsilon_xy - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compute_strain_unit_h() {
        let strain = compute_strain([2.0, 2.0], 2.0);
        assert!((strain.epsilon_xx - 1.0).abs() < 1e-6);
        assert!((strain.epsilon_yy - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_stress_zero_strain() {
        let cfg = default_elastic_config();
        let strain = StrainState {
            epsilon_xx: 0.0,
            epsilon_yy: 0.0,
            epsilon_xy: 0.0,
        };
        let stress = compute_stress(&strain, &cfg);
        assert!((stress.sigma_xx).abs() < 1e-6);
        assert!((stress.sigma_yy).abs() < 1e-6);
        assert!((stress.sigma_xy).abs() < 1e-6);
    }

    #[test]
    fn test_compute_stress_uniaxial() {
        let cfg = default_elastic_config();
        // Uniaxial: epsilon_xx = 1e-3, rest zero
        let strain = StrainState {
            epsilon_xx: 1e-3,
            epsilon_yy: 0.0,
            epsilon_xy: 0.0,
        };
        let stress = compute_stress(&strain, &cfg);
        // sigma_xx = E/(1-nu^2) * eps_xx
        let expected = 1e6 / (1.0 - 0.09) * 1e-3;
        assert!((stress.sigma_xx - expected).abs() < 1.0);
        // sigma_yy = nu * sigma_xx
        assert!((stress.sigma_yy - 0.3 * expected).abs() < 1.0);
    }

    #[test]
    fn test_von_mises_zero() {
        let s = StressState {
            sigma_xx: 0.0,
            sigma_yy: 0.0,
            sigma_xy: 0.0,
        };
        assert!((von_mises_stress(&s)).abs() < 1e-6);
    }

    #[test]
    fn test_von_mises_uniaxial() {
        let s = StressState {
            sigma_xx: 100.0,
            sigma_yy: 0.0,
            sigma_xy: 0.0,
        };
        // von Mises = sqrt(100^2) = 100
        assert!((von_mises_stress(&s) - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_von_mises_equibiaxial() {
        // Both normal stresses equal, no shear
        let s = StressState {
            sigma_xx: 100.0,
            sigma_yy: 100.0,
            sigma_xy: 0.0,
        };
        // von Mises = sqrt(100^2 - 100*100 + 100^2) = 100
        assert!((von_mises_stress(&s) - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_plane_stress_matrix_diagonal() {
        let cfg = default_elastic_config();
        let d = plane_stress_matrix(&cfg);
        // D[0][0] = D[1][1] = E/(1-nu^2)
        let expected = 1e6 / (1.0 - 0.09);
        assert!((d[0][0] - expected).abs() < 1.0);
        assert!((d[1][1] - expected).abs() < 1.0);
        // D[0][2] = D[1][2] = 0
        assert!((d[0][2]).abs() < 1e-6);
        assert!((d[1][2]).abs() < 1e-6);
    }

    #[test]
    fn test_plane_stress_matrix_offdiag() {
        let cfg = default_elastic_config();
        let d = plane_stress_matrix(&cfg);
        let expected_nu = 1e6 * 0.3 / (1.0 - 0.09);
        assert!((d[0][1] - expected_nu).abs() < 1.0);
        assert!((d[1][0] - expected_nu).abs() < 1.0);
    }

    #[test]
    fn test_plane_stress_matrix_shear() {
        let cfg = default_elastic_config();
        let d = plane_stress_matrix(&cfg);
        // D[2][2] = E/(1-nu^2) * (1-nu)/2 = E/(2*(1+nu))
        let mu = lame_mu(&cfg);
        assert!((d[2][2] - mu).abs() < 1.0);
    }

    #[test]
    fn test_elastic_result_to_json() {
        let stress = StressState {
            sigma_xx: 1.0,
            sigma_yy: 2.0,
            sigma_xy: 0.5,
        };
        let strain = StrainState {
            epsilon_xx: 0.001,
            epsilon_yy: 0.002,
            epsilon_xy: 0.0005,
        };
        let vm = von_mises_stress(&stress);
        let result = ElasticResult {
            stress,
            strain,
            von_mises: vm,
        };
        let json = elastic_result_to_json(&result);
        assert!(json.contains("sigma_xx"));
        assert!(json.contains("von_mises"));
    }

    #[test]
    fn test_strain_zero_h() {
        // Should not panic with zero h
        let strain = compute_strain([1.0, 1.0], 0.0);
        // All strains should be zero (guarded)
        assert!((strain.epsilon_xx).abs() < 1e-9);
    }

    #[test]
    fn test_full_pipeline() {
        let cfg = default_elastic_config();
        let disp = [1e-4, 2e-4];
        let h = 0.01;
        let strain = compute_strain(disp, h);
        let stress = compute_stress(&strain, &cfg);
        let vm = von_mises_stress(&stress);
        assert!(vm >= 0.0);
        let result = ElasticResult {
            stress,
            strain,
            von_mises: vm,
        };
        let json = elastic_result_to_json(&result);
        assert!(!json.is_empty());
    }
}
