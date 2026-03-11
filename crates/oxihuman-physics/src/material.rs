// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Physical cloth material definitions and presets.
//!
//! Provides [`ClothMaterial`] with parameters used by the cloth simulation,
//! along with common preset constructors (cotton, silk, denim, etc.)
//! and [`ClothStack`] for layered clothing setups.

// ── ClothMaterial ─────────────────────────────────────────────────────────────

/// Physical properties of a cloth material.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClothMaterial {
    pub name: String,
    /// Mass per unit area (kg/m²). Typical: 0.1 (silk) to 0.8 (denim).
    pub surface_density: f32,
    /// Structural spring stiffness [0, 1]. Higher = stiffer.
    pub structural_stiffness: f32,
    /// Shear spring stiffness [0, 1].
    pub shear_stiffness: f32,
    /// Bending stiffness [0, 1]. Higher = less drape.
    pub bending_stiffness: f32,
    /// Damping coefficient [0, 1]. Higher = faster settling.
    pub damping: f32,
    /// Friction coefficient [0, 1] for collisions.
    pub friction: f32,
    /// Stretchability [0, 1]. 0 = inelastic, 1 = very stretchy.
    pub stretch: f32,
    /// Thickness in meters (for collision detection).
    pub thickness: f32,
}

impl ClothMaterial {
    /// Create a new material with default (neutral) values.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            surface_density: 0.2,
            structural_stiffness: 0.5,
            shear_stiffness: 0.5,
            bending_stiffness: 0.5,
            damping: 0.3,
            friction: 0.5,
            stretch: 0.1,
            thickness: 0.001,
        }
    }

    // ── Preset constructors ────────────────────────────────────────────────────

    /// Cotton preset: medium weight, moderate stiffness, low stretch.
    pub fn cotton() -> Self {
        Self {
            name: "cotton".to_string(),
            surface_density: 0.2,
            structural_stiffness: 0.8,
            shear_stiffness: 0.6,
            bending_stiffness: 0.4,
            damping: 0.3,
            friction: 0.5,
            stretch: 0.1,
            thickness: 0.001,
        }
    }

    /// Silk preset: lightweight, low stiffness, excellent drape.
    pub fn silk() -> Self {
        Self {
            name: "silk".to_string(),
            surface_density: 0.1,
            structural_stiffness: 0.5,
            shear_stiffness: 0.3,
            bending_stiffness: 0.1,
            damping: 0.1,
            friction: 0.2,
            stretch: 0.2,
            thickness: 0.0005,
        }
    }

    /// Denim preset: heavy, very stiff, low stretch.
    pub fn denim() -> Self {
        Self {
            name: "denim".to_string(),
            surface_density: 0.5,
            structural_stiffness: 0.95,
            shear_stiffness: 0.8,
            bending_stiffness: 0.8,
            damping: 0.4,
            friction: 0.7,
            stretch: 0.05,
            thickness: 0.002,
        }
    }

    /// Leather preset: heavy, very stiff, minimal stretch.
    pub fn leather() -> Self {
        Self {
            name: "leather".to_string(),
            surface_density: 0.7,
            structural_stiffness: 0.98,
            shear_stiffness: 0.9,
            bending_stiffness: 0.95,
            damping: 0.5,
            friction: 0.8,
            stretch: 0.02,
            thickness: 0.003,
        }
    }

    /// Rubber preset: medium weight, low stiffness, very stretchy.
    pub fn rubber() -> Self {
        Self {
            name: "rubber".to_string(),
            surface_density: 0.4,
            structural_stiffness: 0.3,
            shear_stiffness: 0.2,
            bending_stiffness: 0.2,
            damping: 0.6,
            friction: 0.9,
            stretch: 0.9,
            thickness: 0.002,
        }
    }

    /// Wool preset: medium weight, moderate stiffness, some stretch.
    pub fn wool() -> Self {
        Self {
            name: "wool".to_string(),
            surface_density: 0.3,
            structural_stiffness: 0.7,
            shear_stiffness: 0.5,
            bending_stiffness: 0.5,
            damping: 0.5,
            friction: 0.6,
            stretch: 0.3,
            thickness: 0.002,
        }
    }

    // ── Queries ────────────────────────────────────────────────────────────────

    /// Whether the material would behave "stiff" (low drape).
    ///
    /// Returns `true` when `bending_stiffness > 0.7`.
    pub fn is_stiff(&self) -> bool {
        self.bending_stiffness > 0.7
    }

    /// Whether the material would drape well.
    ///
    /// Returns `true` when `bending_stiffness < 0.3`.
    pub fn is_drapeable(&self) -> bool {
        self.bending_stiffness < 0.3
    }

    /// Compute approximate terminal velocity for gravity = 9.8 m/s².
    ///
    /// Uses a simplified aerodynamic drag model:
    /// `v_t = sqrt(2 * g * surface_density / (air_density * drag_coeff))`
    /// with `air_density = 1.225 kg/m³` and `drag_coeff = 1.0`.
    pub fn terminal_velocity(&self) -> f32 {
        let g = 9.8_f32;
        let air_density = 1.225_f32;
        let drag_coeff = 1.0_f32;
        (2.0 * g * self.surface_density / (air_density * drag_coeff)).sqrt()
    }

    // ── Transformations ───────────────────────────────────────────────────────

    /// Interpolate between two materials by factor `t` in `[0, 1]`.
    ///
    /// `t = 0` returns a clone of `self`; `t = 1` returns a clone of `other`.
    pub fn lerp(&self, other: &ClothMaterial, t: f32) -> ClothMaterial {
        let t = t.clamp(0.0, 1.0);
        let s = 1.0 - t;
        ClothMaterial {
            name: if t < 0.5 {
                self.name.clone()
            } else {
                other.name.clone()
            },
            surface_density: self.surface_density * s + other.surface_density * t,
            structural_stiffness: self.structural_stiffness * s + other.structural_stiffness * t,
            shear_stiffness: self.shear_stiffness * s + other.shear_stiffness * t,
            bending_stiffness: self.bending_stiffness * s + other.bending_stiffness * t,
            damping: self.damping * s + other.damping * t,
            friction: self.friction * s + other.friction * t,
            stretch: self.stretch * s + other.stretch * t,
            thickness: self.thickness * s + other.thickness * t,
        }
    }

    /// Scale all stiffness values by `scale` (useful for LOD reduction).
    ///
    /// Clamps resulting stiffness values to `[0, 1]`.
    pub fn with_stiffness_scale(&self, scale: f32) -> ClothMaterial {
        ClothMaterial {
            name: self.name.clone(),
            surface_density: self.surface_density,
            structural_stiffness: (self.structural_stiffness * scale).clamp(0.0, 1.0),
            shear_stiffness: (self.shear_stiffness * scale).clamp(0.0, 1.0),
            bending_stiffness: (self.bending_stiffness * scale).clamp(0.0, 1.0),
            damping: self.damping,
            friction: self.friction,
            stretch: self.stretch,
            thickness: self.thickness,
        }
    }

    /// Return a material adjusted for a given wind strength in `[0, 1]`.
    ///
    /// Higher wind increases damping (settling faster) and slightly reduces
    /// structural/shear stiffness (cloth becomes more responsive).
    pub fn for_wind_strength(&self, wind: f32) -> ClothMaterial {
        let wind = wind.clamp(0.0, 1.0);
        ClothMaterial {
            name: self.name.clone(),
            surface_density: self.surface_density,
            structural_stiffness: (self.structural_stiffness * (1.0 - wind * 0.2)).clamp(0.0, 1.0),
            shear_stiffness: (self.shear_stiffness * (1.0 - wind * 0.1)).clamp(0.0, 1.0),
            bending_stiffness: self.bending_stiffness,
            damping: (self.damping + wind * 0.4).clamp(0.0, 1.0),
            friction: self.friction,
            stretch: self.stretch,
            thickness: self.thickness,
        }
    }

    // ── Preset registry ───────────────────────────────────────────────────────

    /// Return all built-in preset materials.
    pub fn all_presets() -> Vec<ClothMaterial> {
        vec![
            ClothMaterial::cotton(),
            ClothMaterial::silk(),
            ClothMaterial::denim(),
            ClothMaterial::leather(),
            ClothMaterial::rubber(),
            ClothMaterial::wool(),
        ]
    }

    /// Find a preset by name (case-insensitive).
    ///
    /// Returns `None` if no preset matches.
    pub fn find_preset(name: &str) -> Option<ClothMaterial> {
        let lower = name.to_lowercase();
        ClothMaterial::all_presets()
            .into_iter()
            .find(|m| m.name == lower)
    }
}

// ── ClothStack ────────────────────────────────────────────────────────────────

/// A layered cloth setup: multiple material layers (e.g., shirt + jacket).
#[allow(dead_code)]
pub struct ClothStack {
    pub layers: Vec<ClothMaterial>,
}

impl ClothStack {
    /// Create an empty stack.
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Push a material layer onto the stack.
    pub fn push(&mut self, mat: ClothMaterial) {
        self.layers.push(mat);
    }

    /// Compute the blended effective material for the full stack.
    ///
    /// Returns an average over all layers weighted equally.
    /// If the stack is empty, returns a default `ClothMaterial::new("empty")`.
    pub fn effective_material(&self) -> ClothMaterial {
        if self.layers.is_empty() {
            return ClothMaterial::new("empty");
        }

        let n = self.layers.len() as f32;
        let mut result = ClothMaterial::new("stack");
        result.surface_density = 0.0;
        result.structural_stiffness = 0.0;
        result.shear_stiffness = 0.0;
        result.bending_stiffness = 0.0;
        result.damping = 0.0;
        result.friction = 0.0;
        result.stretch = 0.0;
        result.thickness = 0.0;

        for layer in &self.layers {
            result.surface_density += layer.surface_density;
            result.structural_stiffness += layer.structural_stiffness;
            result.shear_stiffness += layer.shear_stiffness;
            result.bending_stiffness += layer.bending_stiffness;
            result.damping += layer.damping;
            result.friction += layer.friction;
            result.stretch += layer.stretch;
            result.thickness += layer.thickness;
        }

        result.surface_density /= n;
        result.structural_stiffness /= n;
        result.shear_stiffness /= n;
        result.bending_stiffness /= n;
        result.damping /= n;
        result.friction /= n;
        result.stretch /= n;
        result.thickness /= n;

        result
    }

    /// Compute total surface density across all layers (kg/m²).
    pub fn total_surface_density(&self) -> f32 {
        self.layers.iter().map(|l| l.surface_density).sum()
    }

    /// Number of layers in the stack.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
}

impl Default for ClothStack {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cotton_preset_name() {
        assert_eq!(ClothMaterial::cotton().name, "cotton");
    }

    #[test]
    fn silk_preset_lighter_than_denim() {
        assert!(
            ClothMaterial::silk().surface_density < ClothMaterial::denim().surface_density,
            "silk should be lighter than denim"
        );
    }

    #[test]
    fn denim_is_stiff() {
        assert!(
            ClothMaterial::denim().is_stiff(),
            "denim bending_stiffness should be > 0.7"
        );
    }

    #[test]
    fn silk_is_drapeable() {
        assert!(
            ClothMaterial::silk().is_drapeable(),
            "silk bending_stiffness should be < 0.3"
        );
    }

    #[test]
    fn lerp_at_zero_equals_self() {
        let cotton = ClothMaterial::cotton();
        let denim = ClothMaterial::denim();
        let result = cotton.lerp(&denim, 0.0);
        assert!(
            (result.surface_density - cotton.surface_density).abs() < 1e-5,
            "lerp(t=0) surface_density should match self"
        );
        assert!(
            (result.bending_stiffness - cotton.bending_stiffness).abs() < 1e-5,
            "lerp(t=0) bending_stiffness should match self"
        );
    }

    #[test]
    fn lerp_at_one_equals_other() {
        let cotton = ClothMaterial::cotton();
        let denim = ClothMaterial::denim();
        let result = cotton.lerp(&denim, 1.0);
        assert!(
            (result.surface_density - denim.surface_density).abs() < 1e-5,
            "lerp(t=1) surface_density should match other"
        );
        assert!(
            (result.bending_stiffness - denim.bending_stiffness).abs() < 1e-5,
            "lerp(t=1) bending_stiffness should match other"
        );
    }

    #[test]
    fn lerp_midpoint_density() {
        let cotton = ClothMaterial::cotton(); // density = 0.2
        let denim = ClothMaterial::denim(); // density = 0.5
        let result = cotton.lerp(&denim, 0.5);
        let expected = (0.2 + 0.5) / 2.0;
        assert!(
            (result.surface_density - expected).abs() < 1e-5,
            "lerp(0.5) density expected {expected}, got {}",
            result.surface_density
        );
    }

    #[test]
    fn with_stiffness_scale_halves_structural() {
        let cotton = ClothMaterial::cotton(); // structural_stiffness = 0.8
        let scaled = cotton.with_stiffness_scale(0.5);
        let expected = 0.8 * 0.5;
        assert!(
            (scaled.structural_stiffness - expected).abs() < 1e-5,
            "expected structural_stiffness {expected}, got {}",
            scaled.structural_stiffness
        );
    }

    #[test]
    fn for_wind_strong_increases_damping() {
        let silk = ClothMaterial::silk(); // damping = 0.1
        let windy = silk.for_wind_strength(1.0);
        assert!(
            windy.damping > silk.damping,
            "strong wind should increase damping: {} -> {}",
            silk.damping,
            windy.damping
        );
    }

    #[test]
    fn terminal_velocity_positive() {
        for mat in ClothMaterial::all_presets() {
            let tv = mat.terminal_velocity();
            assert!(
                tv > 0.0,
                "terminal_velocity for '{}' should be positive, got {tv}",
                mat.name
            );
        }
    }

    #[test]
    fn all_presets_count() {
        assert_eq!(
            ClothMaterial::all_presets().len(),
            6,
            "expected 6 presets: cotton, silk, denim, leather, rubber, wool"
        );
    }

    #[test]
    fn find_preset_by_name() {
        let mat = ClothMaterial::find_preset("denim").expect("denim preset should exist");
        assert_eq!(mat.name, "denim");
    }

    #[test]
    fn find_preset_case_insensitive() {
        let mat = ClothMaterial::find_preset("SILK").expect("SILK should match silk preset");
        assert_eq!(mat.name, "silk");
    }

    #[test]
    fn cloth_stack_effective_material() {
        let mut stack = ClothStack::new();
        stack.push(ClothMaterial::cotton()); // density = 0.2
        stack.push(ClothMaterial::denim()); // density = 0.5
        let eff = stack.effective_material();
        let expected_density = (0.2 + 0.5) / 2.0;
        assert!(
            (eff.surface_density - expected_density).abs() < 1e-5,
            "effective density expected {expected_density}, got {}",
            eff.surface_density
        );
    }

    #[test]
    fn cloth_stack_total_density() {
        let mut stack = ClothStack::new();
        stack.push(ClothMaterial::cotton()); // 0.2
        stack.push(ClothMaterial::silk()); // 0.1
        stack.push(ClothMaterial::wool()); // 0.3
        let total = stack.total_surface_density();
        let expected = 0.2 + 0.1 + 0.3;
        assert!(
            (total - expected).abs() < 1e-5,
            "total density expected {expected}, got {total}"
        );
    }
}
