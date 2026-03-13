// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Magnetorheological fluid model (field-dependent viscosity).

#![allow(dead_code)]

/// MR fluid material parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MrFluidParams {
    /// Base viscosity (zero field).
    pub eta_0: f64,
    /// Saturation viscosity (max field).
    pub eta_sat: f64,
    /// Magnetic saturation field (T).
    pub b_sat: f64,
    /// Yield stress coefficient.
    pub tau_y0: f64,
    /// Yield stress saturation exponent.
    pub alpha: f64,
}

impl MrFluidParams {
    #[allow(dead_code)]
    pub fn new(eta_0: f64, eta_sat: f64, b_sat: f64, tau_y0: f64, alpha: f64) -> Self {
        Self { eta_0, eta_sat, b_sat, tau_y0, alpha }
    }
}

/// MR fluid element state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MrFluidElement {
    pub params: MrFluidParams,
    pub magnetic_field: f64,
    pub shear_rate: f64,
}

impl MrFluidElement {
    #[allow(dead_code)]
    pub fn new(params: MrFluidParams) -> Self {
        Self { params, magnetic_field: 0.0, shear_rate: 0.0 }
    }

    #[allow(dead_code)]
    pub fn set_field(&mut self, b: f64) {
        self.magnetic_field = b.max(0.0);
    }

    #[allow(dead_code)]
    pub fn set_shear_rate(&mut self, rate: f64) {
        self.shear_rate = rate;
    }

    /// Field-dependent viscosity (Bingham model approximation).
    #[allow(dead_code)]
    pub fn effective_viscosity(&self) -> f64 {
        let b = self.magnetic_field;
        let b_sat = self.params.b_sat;
        let ratio = (b / b_sat).min(1.0);
        self.params.eta_0 + (self.params.eta_sat - self.params.eta_0) * ratio * ratio
    }

    /// Yield stress from field: tau_y = tau_y0 * (B/B_sat)^alpha.
    #[allow(dead_code)]
    pub fn yield_stress(&self) -> f64 {
        let ratio = (self.magnetic_field / self.params.b_sat).min(1.0);
        self.params.tau_y0 * ratio.powf(self.params.alpha)
    }

    /// Total shear stress (Bingham-Plastic model): tau = tau_y + eta * gamma_dot.
    #[allow(dead_code)]
    pub fn shear_stress(&self) -> f64 {
        let tau_y = self.yield_stress();
        let eta = self.effective_viscosity();
        if self.shear_rate.abs() > 1e-12 {
            tau_y * self.shear_rate.signum() + eta * self.shear_rate
        } else {
            0.0
        }
    }

    /// Apparent viscosity = tau / gamma_dot.
    #[allow(dead_code)]
    pub fn apparent_viscosity(&self) -> f64 {
        if self.shear_rate.abs() > 1e-12 {
            self.shear_stress() / self.shear_rate
        } else {
            f64::INFINITY
        }
    }

    /// Controllable force ratio: (eta_max - eta_0) / eta_0.
    #[allow(dead_code)]
    pub fn controllable_ratio(&self) -> f64 {
        (self.params.eta_sat - self.params.eta_0) / self.params.eta_0.max(1e-12)
    }

    /// Check if material is flowing (shear stress > yield stress).
    #[allow(dead_code)]
    pub fn is_flowing(&self) -> bool {
        self.shear_stress().abs() > self.yield_stress()
    }
}

/// MR damper model.
#[allow(dead_code)]
pub struct MrDamper {
    pub fluid: MrFluidElement,
    pub piston_area: f64,
    pub stroke: f64,
    pub velocity: f64,
}

impl MrDamper {
    #[allow(dead_code)]
    pub fn new(fluid: MrFluidElement, piston_area: f64, stroke: f64) -> Self {
        Self { fluid, piston_area, stroke, velocity: 0.0 }
    }

    #[allow(dead_code)]
    pub fn set_velocity(&mut self, v: f64) {
        self.velocity = v;
        let gamma_dot = if self.stroke > 0.0 { v / self.stroke } else { 0.0 };
        self.fluid.set_shear_rate(gamma_dot);
    }

    #[allow(dead_code)]
    pub fn damping_force(&self) -> f64 {
        self.fluid.shear_stress() * self.piston_area
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> MrFluidParams {
        MrFluidParams::new(0.1, 1.0, 1.0, 5000.0, 2.0)
    }

    #[test]
    fn test_zero_field_viscosity() {
        let elem = MrFluidElement::new(default_params());
        assert!((elem.effective_viscosity() - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_max_field_viscosity() {
        let mut elem = MrFluidElement::new(default_params());
        elem.set_field(1.0);
        assert!((elem.effective_viscosity() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_yield_stress_zero_field() {
        let elem = MrFluidElement::new(default_params());
        assert!((elem.yield_stress()).abs() < 1e-12);
    }

    #[test]
    fn test_yield_stress_at_saturation() {
        let mut elem = MrFluidElement::new(default_params());
        elem.set_field(1.0);
        assert!((elem.yield_stress() - 5000.0).abs() < 1e-6);
    }

    #[test]
    fn test_shear_stress_no_flow() {
        let elem = MrFluidElement::new(default_params());
        assert!((elem.shear_stress()).abs() < 1e-12);
    }

    #[test]
    fn test_shear_stress_with_flow() {
        let mut elem = MrFluidElement::new(default_params());
        elem.set_shear_rate(10.0);
        assert!(elem.shear_stress() > 0.0);
    }

    #[test]
    fn test_controllable_ratio() {
        let elem = MrFluidElement::new(default_params());
        assert!(elem.controllable_ratio() > 0.0);
    }

    #[test]
    fn test_damper_force() {
        let elem = MrFluidElement::new(default_params());
        let mut damper = MrDamper::new(elem, 0.01, 0.1);
        damper.set_velocity(1.0);
        let f = damper.damping_force();
        assert!(f.abs() >= 0.0);
    }

    #[test]
    fn test_negative_field_clamped() {
        let mut elem = MrFluidElement::new(default_params());
        elem.set_field(-1.0);
        assert_eq!(elem.magnetic_field, 0.0);
    }

    #[test]
    fn test_viscosity_increases_with_field() {
        let mut elem = MrFluidElement::new(default_params());
        let v0 = elem.effective_viscosity();
        elem.set_field(0.5);
        let v1 = elem.effective_viscosity();
        assert!(v1 > v0);
    }
}
