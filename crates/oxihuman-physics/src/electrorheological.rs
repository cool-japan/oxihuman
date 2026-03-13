// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Electrorheological fluid model (electric field-dependent viscosity).

#![allow(dead_code)]

/// ER fluid parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ErFluidParams {
    /// Zero-field viscosity.
    pub eta_0: f64,
    /// Saturated viscosity at max field.
    pub eta_sat: f64,
    /// Reference electric field strength (V/m).
    pub e_ref: f64,
    /// Yield stress coefficient.
    pub tau_y0: f64,
    /// Electric field exponent.
    pub alpha: f64,
    /// Dielectric constant ratio (particle/medium).
    pub dielectric_ratio: f64,
}

impl ErFluidParams {
    #[allow(dead_code)]
    pub fn new(eta_0: f64, eta_sat: f64, e_ref: f64, tau_y0: f64, alpha: f64, dielectric_ratio: f64) -> Self {
        Self { eta_0, eta_sat, e_ref, tau_y0, alpha, dielectric_ratio }
    }
}

/// ER fluid element.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ErFluidElement {
    pub params: ErFluidParams,
    pub electric_field: f64,
    pub shear_rate: f64,
    pub temperature: f64,
}

impl ErFluidElement {
    #[allow(dead_code)]
    pub fn new(params: ErFluidParams) -> Self {
        Self {
            params,
            electric_field: 0.0,
            shear_rate: 0.0,
            temperature: 298.15,
        }
    }

    #[allow(dead_code)]
    pub fn set_field(&mut self, e: f64) {
        self.electric_field = e.max(0.0);
    }

    #[allow(dead_code)]
    pub fn set_shear_rate(&mut self, rate: f64) {
        self.shear_rate = rate;
    }

    #[allow(dead_code)]
    pub fn set_temperature(&mut self, temp: f64) {
        self.temperature = temp;
    }

    /// Normalized field E/E_ref (clamped to [0,1]).
    fn normalized_field(&self) -> f64 {
        (self.electric_field / self.params.e_ref).min(1.0)
    }

    /// Field-dependent viscosity (quadratic saturation).
    #[allow(dead_code)]
    pub fn effective_viscosity(&self) -> f64 {
        let en = self.normalized_field();
        self.params.eta_0 + (self.params.eta_sat - self.params.eta_0) * en * en
    }

    /// Yield stress: tau_y = tau_y0 * (E/E_ref)^alpha.
    #[allow(dead_code)]
    pub fn yield_stress(&self) -> f64 {
        let en = self.normalized_field();
        self.params.tau_y0 * en.powf(self.params.alpha)
    }

    /// Shear stress (Bingham model).
    #[allow(dead_code)]
    pub fn shear_stress(&self) -> f64 {
        if self.shear_rate.abs() < 1e-12 {
            return 0.0;
        }
        let tau_y = self.yield_stress();
        let eta = self.effective_viscosity();
        tau_y * self.shear_rate.signum() + eta * self.shear_rate
    }

    /// Dielectrophoretic force contribution (simplified).
    #[allow(dead_code)]
    pub fn dielectrophoretic_stress(&self) -> f64 {
        let k = self.params.dielectric_ratio;
        let e = self.electric_field;
        let epsilon0 = 8.854e-12;
        0.5 * epsilon0 * k * e * e
    }

    /// Effective Mason number: Mn = eta_0 * gamma_dot / (tau_y + 1e-12).
    #[allow(dead_code)]
    pub fn mason_number(&self) -> f64 {
        let tau_y = self.yield_stress();
        self.params.eta_0 * self.shear_rate.abs() / (tau_y + 1e-12)
    }

    #[allow(dead_code)]
    pub fn is_yielded(&self) -> bool {
        self.shear_stress().abs() > self.yield_stress()
    }

    /// Temperature-corrected viscosity (Arrhenius approximation).
    #[allow(dead_code)]
    pub fn temp_corrected_viscosity(&self, activation_energy: f64) -> f64 {
        let r = 8.314;
        let t_ref = 298.15;
        let correction = (activation_energy / r * (1.0 / self.temperature - 1.0 / t_ref)).exp();
        self.effective_viscosity() * correction
    }
}

/// ER damper.
#[allow(dead_code)]
pub struct ErDamper {
    pub fluid: ErFluidElement,
    pub gap: f64,
    pub area: f64,
}

impl ErDamper {
    #[allow(dead_code)]
    pub fn new(fluid: ErFluidElement, gap: f64, area: f64) -> Self {
        Self { fluid, gap, area }
    }

    #[allow(dead_code)]
    pub fn set_velocity(&mut self, v: f64) {
        let rate = if self.gap > 0.0 { v / self.gap } else { 0.0 };
        self.fluid.set_shear_rate(rate);
    }

    #[allow(dead_code)]
    pub fn damping_force(&self) -> f64 {
        self.fluid.shear_stress() * self.area
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_er() -> ErFluidParams {
        ErFluidParams::new(0.05, 0.5, 1000.0, 2000.0, 2.0, 10.0)
    }

    #[test]
    fn test_zero_field_viscosity() {
        let elem = ErFluidElement::new(default_er());
        assert!((elem.effective_viscosity() - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_max_field_viscosity() {
        let mut elem = ErFluidElement::new(default_er());
        elem.set_field(1000.0);
        assert!((elem.effective_viscosity() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_yield_stress_zero_field() {
        let elem = ErFluidElement::new(default_er());
        assert!((elem.yield_stress()).abs() < 1e-12);
    }

    #[test]
    fn test_yield_stress_at_reference() {
        let mut elem = ErFluidElement::new(default_er());
        elem.set_field(1000.0);
        assert!((elem.yield_stress() - 2000.0).abs() < 1e-6);
    }

    #[test]
    fn test_shear_stress_zero_rate() {
        let mut elem = ErFluidElement::new(default_er());
        elem.set_field(500.0);
        assert!((elem.shear_stress()).abs() < 1e-12);
    }

    #[test]
    fn test_shear_stress_with_rate() {
        let mut elem = ErFluidElement::new(default_er());
        elem.set_shear_rate(5.0);
        assert!(elem.shear_stress() > 0.0);
    }

    #[test]
    fn test_negative_field_clamped() {
        let mut elem = ErFluidElement::new(default_er());
        elem.set_field(-500.0);
        assert_eq!(elem.electric_field, 0.0);
    }

    #[test]
    fn test_dielectrophoretic_stress() {
        let mut elem = ErFluidElement::new(default_er());
        elem.set_field(1000.0);
        assert!(elem.dielectrophoretic_stress() > 0.0);
    }

    #[test]
    fn test_damper_force() {
        let mut elem = ErFluidElement::new(default_er());
        elem.set_field(500.0);
        let mut damper = ErDamper::new(elem, 0.001, 0.01);
        damper.set_velocity(0.5);
        let f = damper.damping_force();
        assert!(f.abs() >= 0.0);
    }

    #[test]
    fn test_mason_number() {
        let mut elem = ErFluidElement::new(default_er());
        elem.set_field(1000.0);
        elem.set_shear_rate(1.0);
        assert!(elem.mason_number() >= 0.0);
    }
}
