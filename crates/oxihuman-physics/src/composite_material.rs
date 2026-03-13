// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Layered composite material (rule of mixtures).

#![allow(dead_code)]

/// A single layer in the composite.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub volume_fraction: f64,
    pub young_modulus: f64,
    pub density: f64,
    pub thickness: f64,
}

impl Layer {
    #[allow(dead_code)]
    pub fn new(name: &str, vf: f64, young: f64, density: f64, thickness: f64) -> Self {
        Self {
            name: name.to_string(),
            volume_fraction: vf.clamp(0.0, 1.0),
            young_modulus: young,
            density,
            thickness,
        }
    }
}

/// Composite material built from stacked layers.
#[allow(dead_code)]
pub struct CompositeMaterial {
    pub layers: Vec<Layer>,
}

impl CompositeMaterial {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    /// Rule of mixtures (longitudinal) modulus.
    #[allow(dead_code)]
    pub fn longitudinal_modulus(&self) -> f64 {
        self.layers
            .iter()
            .map(|l| l.volume_fraction * l.young_modulus)
            .sum()
    }

    /// Inverse rule of mixtures (transverse) modulus.
    #[allow(dead_code)]
    pub fn transverse_modulus(&self) -> f64 {
        let inv_sum: f64 = self
            .layers
            .iter()
            .map(|l| {
                if l.young_modulus > 0.0 {
                    l.volume_fraction / l.young_modulus
                } else {
                    0.0
                }
            })
            .sum();
        if inv_sum > 0.0 {
            1.0 / inv_sum
        } else {
            0.0
        }
    }

    /// Composite density (rule of mixtures).
    #[allow(dead_code)]
    pub fn density(&self) -> f64 {
        self.layers
            .iter()
            .map(|l| l.volume_fraction * l.density)
            .sum()
    }

    /// Total thickness.
    #[allow(dead_code)]
    pub fn total_thickness(&self) -> f64 {
        self.layers.iter().map(|l| l.thickness).sum()
    }

    /// Volume fraction sum (should be ~1 for a valid composite).
    #[allow(dead_code)]
    pub fn total_volume_fraction(&self) -> f64 {
        self.layers.iter().map(|l| l.volume_fraction).sum()
    }

    #[allow(dead_code)]
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Specific stiffness = modulus / density.
    #[allow(dead_code)]
    pub fn specific_stiffness(&self) -> f64 {
        let e = self.longitudinal_modulus();
        let rho = self.density();
        if rho > 0.0 {
            e / rho
        } else {
            0.0
        }
    }

    /// Natural frequency estimate (spring-mass: sqrt(k/m) for unit length).
    #[allow(dead_code)]
    pub fn natural_frequency_estimate(&self) -> f64 {
        let e = self.longitudinal_modulus();
        let rho = self.density();
        let t = self.total_thickness();
        if rho > 0.0 && t > 0.0 {
            (e / (rho * t * t)).sqrt()
        } else {
            0.0
        }
    }
}

impl Default for CompositeMaterial {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_layer_composite() -> CompositeMaterial {
        let mut c = CompositeMaterial::new();
        c.add_layer(Layer::new("fiber", 0.6, 200e9, 2500.0, 0.001));
        c.add_layer(Layer::new("matrix", 0.4, 10e9, 1200.0, 0.002));
        c
    }

    #[test]
    fn test_longitudinal_modulus() {
        let c = two_layer_composite();
        let e = c.longitudinal_modulus();
        assert!((e - (0.6 * 200e9 + 0.4 * 10e9)).abs() < 1.0);
    }

    #[test]
    fn test_transverse_modulus_less_than_longitudinal() {
        let c = two_layer_composite();
        assert!(c.transverse_modulus() < c.longitudinal_modulus());
    }

    #[test]
    fn test_density() {
        let c = two_layer_composite();
        let rho = c.density();
        let expected = 0.6 * 2500.0 + 0.4 * 1200.0;
        assert!((rho - expected).abs() < 1e-6);
    }

    #[test]
    fn test_total_thickness() {
        let c = two_layer_composite();
        assert!((c.total_thickness() - 0.003).abs() < 1e-9);
    }

    #[test]
    fn test_volume_fraction_sum() {
        let c = two_layer_composite();
        assert!((c.total_volume_fraction() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_layer_count() {
        let c = two_layer_composite();
        assert_eq!(c.layer_count(), 2);
    }

    #[test]
    fn test_specific_stiffness_positive() {
        let c = two_layer_composite();
        assert!(c.specific_stiffness() > 0.0);
    }

    #[test]
    fn test_vf_clamp() {
        let l = Layer::new("x", 1.5, 100.0, 1000.0, 0.001);
        assert!(l.volume_fraction <= 1.0);
    }

    #[test]
    fn test_empty_composite() {
        let c = CompositeMaterial::new();
        assert_eq!(c.layer_count(), 0);
        assert_eq!(c.longitudinal_modulus(), 0.0);
    }

    #[test]
    fn test_natural_frequency_estimate() {
        let c = two_layer_composite();
        assert!(c.natural_frequency_estimate() > 0.0);
    }
}

// ── Wave 151A simple f32 composite API ─────────────────────────────────────

/// Simple two-phase composite material (fiber + matrix), f32 parameters.
#[derive(Debug, Clone)]
pub struct SimpleComposite {
    pub fiber_volume_fraction: f32,
    pub fiber_modulus: f32,
    pub matrix_modulus: f32,
    pub fiber_strength: f32,
    pub matrix_strength: f32,
}

/// Create a new SimpleComposite.
pub fn new_composite(vf: f32, ef: f32, em: f32, sf: f32, sm: f32) -> SimpleComposite {
    SimpleComposite {
        fiber_volume_fraction: vf.clamp(0.0, 1.0),
        fiber_modulus: ef,
        matrix_modulus: em,
        fiber_strength: sf,
        matrix_strength: sm,
    }
}

/// Longitudinal modulus (rule of mixtures): E_L = Vf*Ef + Vm*Em.
pub fn longitudinal_modulus(c: &SimpleComposite) -> f32 {
    let vm = 1.0 - c.fiber_volume_fraction;
    c.fiber_volume_fraction * c.fiber_modulus + vm * c.matrix_modulus
}

/// Transverse modulus (inverse rule of mixtures): 1/E_T = Vf/Ef + Vm/Em.
pub fn transverse_modulus(c: &SimpleComposite) -> f32 {
    let vm = 1.0 - c.fiber_volume_fraction;
    let inv = if c.fiber_modulus > 1e-9 {
        c.fiber_volume_fraction / c.fiber_modulus
    } else {
        0.0
    } + if c.matrix_modulus > 1e-9 {
        vm / c.matrix_modulus
    } else {
        0.0
    };
    if inv > 1e-20 {
        1.0 / inv
    } else {
        0.0
    }
}

/// Composite strength (rule of mixtures): S = Vf*Sf + Vm*Sm.
pub fn composite_strength(c: &SimpleComposite) -> f32 {
    let vm = 1.0 - c.fiber_volume_fraction;
    c.fiber_volume_fraction * c.fiber_strength + vm * c.matrix_strength
}

/// Returns true when the composite is fiber-dominated (Vf > 0.5).
pub fn is_fiber_dominated(c: &SimpleComposite) -> bool {
    c.fiber_volume_fraction > 0.5
}
