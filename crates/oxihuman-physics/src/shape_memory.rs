// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shape memory alloy (SMA) temperature-driven actuation.

#![allow(dead_code)]

/// SMA phase state.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmaPhase {
    Martensite,
    Austenite,
    Mixed,
}

/// Shape memory alloy material parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmaParams {
    /// Martensite start temperature.
    pub m_s: f64,
    /// Martensite finish temperature.
    pub m_f: f64,
    /// Austenite start temperature.
    pub a_s: f64,
    /// Austenite finish temperature.
    pub a_f: f64,
    /// Maximum transformation strain.
    pub max_strain: f64,
    pub elastic_modulus_martensite: f64,
    pub elastic_modulus_austenite: f64,
}

/// SMA element state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmaElement {
    pub params: SmaParams,
    pub temperature: f64,
    pub martensite_fraction: f64,
    pub strain: f64,
    pub stress: f64,
}

impl SmaElement {
    #[allow(dead_code)]
    pub fn new(params: SmaParams) -> Self {
        let temp = params.m_f - 10.0;
        Self {
            temperature: temp,
            martensite_fraction: 1.0,
            strain: 0.0,
            stress: 0.0,
            params,
        }
    }

    #[allow(dead_code)]
    pub fn phase(&self) -> SmaPhase {
        if self.martensite_fraction >= 0.999 {
            SmaPhase::Martensite
        } else if self.martensite_fraction <= 0.001 {
            SmaPhase::Austenite
        } else {
            SmaPhase::Mixed
        }
    }

    /// Update martensite fraction based on temperature (cosine model).
    #[allow(dead_code)]
    pub fn set_temperature(&mut self, temp: f64) {
        self.temperature = temp;
        let p = &self.params;
        if temp <= p.m_f {
            self.martensite_fraction = 1.0;
        } else if temp >= p.a_f {
            self.martensite_fraction = 0.0;
        } else if temp >= p.a_s && temp <= p.a_f {
            let t = (temp - p.a_s) / (p.a_f - p.a_s);
            let angle = t * std::f64::consts::PI;
            self.martensite_fraction = 0.5 * (1.0 + angle.cos());
        } else if temp >= p.m_f && temp <= p.m_s {
            let t = (temp - p.m_f) / (p.m_s - p.m_f);
            let angle = t * std::f64::consts::PI;
            self.martensite_fraction = 0.5 * (1.0 + angle.cos());
        }
        self.martensite_fraction = self.martensite_fraction.clamp(0.0, 1.0);
    }

    /// Current elastic modulus (mixture rule).
    #[allow(dead_code)]
    pub fn elastic_modulus(&self) -> f64 {
        let f = self.martensite_fraction;
        f * self.params.elastic_modulus_martensite + (1.0 - f) * self.params.elastic_modulus_austenite
    }

    /// Recovery strain available (transformation strain * martensite fraction).
    #[allow(dead_code)]
    pub fn recovery_strain(&self) -> f64 {
        self.params.max_strain * self.martensite_fraction
    }

    /// Apply mechanical strain and compute stress.
    #[allow(dead_code)]
    pub fn apply_strain(&mut self, strain: f64) {
        self.strain = strain;
        let elastic = self.elastic_modulus();
        let transformation = self.params.max_strain * (1.0 - self.martensite_fraction);
        self.stress = elastic * (strain - transformation);
    }

    #[allow(dead_code)]
    pub fn is_shape_recovered(&self) -> bool {
        self.martensite_fraction < 0.05
    }
}

/// SMA actuator with temperature control.
#[allow(dead_code)]
pub struct SmaActuator {
    pub element: SmaElement,
    pub target_temp: f64,
    pub heating_rate: f64,
    pub cooling_rate: f64,
}

impl SmaActuator {
    #[allow(dead_code)]
    pub fn new(element: SmaElement, heating_rate: f64, cooling_rate: f64) -> Self {
        let temp = element.temperature;
        Self {
            element,
            target_temp: temp,
            heating_rate,
            cooling_rate,
        }
    }

    #[allow(dead_code)]
    pub fn step(&mut self, dt: f64) {
        let diff = self.target_temp - self.element.temperature;
        let rate = if diff > 0.0 { self.heating_rate } else { self.cooling_rate };
        let d_temp = diff.signum() * rate * dt;
        let new_temp = if d_temp.abs() > diff.abs() {
            self.target_temp
        } else {
            self.element.temperature + d_temp
        };
        self.element.set_temperature(new_temp);
    }

    #[allow(dead_code)]
    pub fn heat_to(&mut self, temp: f64) {
        self.target_temp = temp;
    }

    #[allow(dead_code)]
    pub fn cool_to(&mut self, temp: f64) {
        self.target_temp = temp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> SmaParams {
        SmaParams {
            m_s: 30.0, m_f: 10.0,
            a_s: 50.0, a_f: 80.0,
            max_strain: 0.05,
            elastic_modulus_martensite: 30e9,
            elastic_modulus_austenite: 60e9,
        }
    }

    #[test]
    fn test_initial_phase_martensite() {
        let elem = SmaElement::new(default_params());
        assert_eq!(elem.phase(), SmaPhase::Martensite);
    }

    #[test]
    fn test_heating_to_austenite() {
        let mut elem = SmaElement::new(default_params());
        elem.set_temperature(90.0);
        assert_eq!(elem.phase(), SmaPhase::Austenite);
    }

    #[test]
    fn test_mixed_phase() {
        let mut elem = SmaElement::new(default_params());
        elem.set_temperature(65.0);
        assert_eq!(elem.phase(), SmaPhase::Mixed);
    }

    #[test]
    fn test_martensite_fraction_clamped() {
        let mut elem = SmaElement::new(default_params());
        elem.set_temperature(-100.0);
        assert!((elem.martensite_fraction - 1.0).abs() < 1e-9);
        elem.set_temperature(200.0);
        assert!((elem.martensite_fraction).abs() < 1e-9);
    }

    #[test]
    fn test_elastic_modulus_martensite() {
        let elem = SmaElement::new(default_params());
        assert!((elem.elastic_modulus() - 30e9).abs() < 1.0);
    }

    #[test]
    fn test_elastic_modulus_austenite() {
        let mut elem = SmaElement::new(default_params());
        elem.set_temperature(100.0);
        assert!((elem.elastic_modulus() - 60e9).abs() < 1.0);
    }

    #[test]
    fn test_recovery_strain() {
        let elem = SmaElement::new(default_params());
        assert!((elem.recovery_strain() - 0.05).abs() < 1e-9);
    }

    #[test]
    fn test_apply_strain() {
        let mut elem = SmaElement::new(default_params());
        elem.apply_strain(0.001);
        assert!(elem.stress != 0.0);
    }

    #[test]
    fn test_actuator_step_heating() {
        let elem = SmaElement::new(default_params());
        let mut act = SmaActuator::new(elem, 10.0, 5.0);
        act.heat_to(100.0);
        act.step(1.0);
        assert!(act.element.temperature > act.element.params.m_f - 10.0);
    }

    #[test]
    fn test_shape_recovered() {
        let mut elem = SmaElement::new(default_params());
        elem.set_temperature(100.0);
        assert!(elem.is_shape_recovered());
    }
}
