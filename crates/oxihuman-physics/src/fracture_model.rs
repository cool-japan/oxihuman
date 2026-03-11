// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fracture simulation (crack propagation threshold).

#![allow(dead_code)]

/// A crack in the material.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Crack {
    pub origin: [f64; 3],
    pub tip: [f64; 3],
    pub length: f64,
    pub propagation_count: usize,
}

impl Crack {
    #[allow(dead_code)]
    pub fn new(origin: [f64; 3]) -> Self {
        Self {
            tip: origin,
            origin,
            length: 0.0,
            propagation_count: 0,
        }
    }

    /// Propagate crack by step in given direction.
    #[allow(dead_code)]
    pub fn propagate(&mut self, direction: [f64; 3], step: f64) {
        let norm = (direction[0] * direction[0]
            + direction[1] * direction[1]
            + direction[2] * direction[2])
            .sqrt();
        if norm < 1e-12 {
            return;
        }
        let d = [direction[0] / norm, direction[1] / norm, direction[2] / norm];
        self.tip[0] += d[0] * step;
        self.tip[1] += d[1] * step;
        self.tip[2] += d[2] * step;
        self.length += step;
        self.propagation_count += 1;
    }
}

/// Fracture material properties.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FractureProps {
    pub fracture_toughness: f64,
    pub critical_stress_intensity: f64,
    pub crack_speed: f64,
}

impl FractureProps {
    #[allow(dead_code)]
    pub fn new(toughness: f64, k_ic: f64, speed: f64) -> Self {
        Self {
            fracture_toughness: toughness,
            critical_stress_intensity: k_ic,
            crack_speed: speed,
        }
    }

    /// Stress intensity factor K_I for mode-I crack.
    #[allow(dead_code)]
    pub fn stress_intensity(&self, stress: f64, crack_length: f64) -> f64 {
        stress * (std::f64::consts::PI * crack_length).sqrt()
    }

    /// Check if crack propagates (K_I >= K_Ic).
    #[allow(dead_code)]
    pub fn will_propagate(&self, stress: f64, crack_length: f64) -> bool {
        self.stress_intensity(stress, crack_length) >= self.critical_stress_intensity
    }

    /// Critical crack length for given stress.
    #[allow(dead_code)]
    pub fn critical_crack_length(&self, stress: f64) -> f64 {
        if stress <= 0.0 {
            return f64::INFINITY;
        }
        (self.critical_stress_intensity / stress).powi(2) / std::f64::consts::PI
    }

    /// Energy release rate G = K_I^2 / E (plane stress).
    #[allow(dead_code)]
    pub fn energy_release_rate(&self, k_i: f64, young: f64) -> f64 {
        k_i * k_i / young
    }
}

/// A body that can fracture.
#[allow(dead_code)]
pub struct FractureBody {
    pub cracks: Vec<Crack>,
    pub props: FractureProps,
}

impl FractureBody {
    #[allow(dead_code)]
    pub fn new(props: FractureProps) -> Self {
        Self { cracks: Vec::new(), props }
    }

    #[allow(dead_code)]
    pub fn add_crack(&mut self, origin: [f64; 3]) -> usize {
        let id = self.cracks.len();
        self.cracks.push(Crack::new(origin));
        id
    }

    #[allow(dead_code)]
    pub fn update(&mut self, stress: f64, dt: f64) {
        let props = self.props.clone();
        for crack in &mut self.cracks {
            if props.will_propagate(stress, crack.length.max(1e-6)) {
                crack.propagate([1.0, 0.0, 0.0], props.crack_speed * dt);
            }
        }
    }

    #[allow(dead_code)]
    pub fn max_crack_length(&self) -> f64 {
        self.cracks.iter().map(|c| c.length).fold(0.0_f64, f64::max)
    }

    #[allow(dead_code)]
    pub fn crack_count(&self) -> usize {
        self.cracks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crack_propagates() {
        let mut crack = Crack::new([0.0; 3]);
        crack.propagate([1.0, 0.0, 0.0], 0.5);
        assert!((crack.length - 0.5).abs() < 1e-9);
        assert_eq!(crack.propagation_count, 1);
    }

    #[test]
    fn test_crack_no_propagate_zero_direction() {
        let mut crack = Crack::new([0.0; 3]);
        crack.propagate([0.0, 0.0, 0.0], 1.0);
        assert_eq!(crack.length, 0.0);
    }

    #[test]
    fn test_stress_intensity() {
        let props = FractureProps::new(10.0, 50.0, 1.0);
        let k = props.stress_intensity(100.0, 0.01);
        assert!(k > 0.0);
    }

    #[test]
    fn test_will_propagate() {
        let props = FractureProps::new(10.0, 10.0, 1.0);
        assert!(props.will_propagate(1000.0, 0.01));
        assert!(!props.will_propagate(1.0, 1e-10));
    }

    #[test]
    fn test_critical_crack_length() {
        let props = FractureProps::new(10.0, 50.0, 1.0);
        let a = props.critical_crack_length(100.0);
        assert!(a > 0.0);
    }

    #[test]
    fn test_energy_release_rate() {
        let props = FractureProps::new(10.0, 50.0, 1.0);
        let g = props.energy_release_rate(10.0, 200.0);
        assert!((g - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_fracture_body_update() {
        let props = FractureProps::new(1.0, 1.0, 10.0);
        let mut body = FractureBody::new(props);
        body.add_crack([0.0; 3]);
        body.update(1e6, 0.01);
        assert!(body.max_crack_length() > 0.0);
    }

    #[test]
    fn test_crack_count() {
        let props = FractureProps::new(1.0, 1.0, 1.0);
        let mut body = FractureBody::new(props);
        body.add_crack([0.0; 3]);
        body.add_crack([1.0, 0.0, 0.0]);
        assert_eq!(body.crack_count(), 2);
    }

    #[test]
    fn test_critical_crack_at_zero_stress() {
        let props = FractureProps::new(1.0, 50.0, 1.0);
        assert_eq!(props.critical_crack_length(0.0), f64::INFINITY);
    }

    #[test]
    fn test_crack_tip_moves() {
        let mut crack = Crack::new([0.0; 3]);
        crack.propagate([0.0, 1.0, 0.0], 2.0);
        assert!((crack.tip[1] - 2.0).abs() < 1e-9);
    }
}
