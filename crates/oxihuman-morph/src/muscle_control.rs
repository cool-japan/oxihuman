// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

//! Muscle-driven deformation control for OxiHuman morph system.
//!
//! Provides named muscles with flex/contract states that drive morph weights.
//! Each [`MuscleDefinition`] maps to a set of morph targets with weighted influence.
//! A [`MuscleRig`] aggregates multiple muscles and evaluates collective morph output.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// MuscleGroup
// ---------------------------------------------------------------------------

/// Anatomical muscle group classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MuscleGroup {
    Chest,
    Back,
    Shoulder,
    Bicep,
    Tricep,
    Forearm,
    Abs,
    Oblique,
    Glute,
    Hamstring,
    Quad,
    Calf,
    Neck,
    Face,
}

impl MuscleGroup {
    /// Returns all muscle groups in anatomical order.
    pub fn all() -> Vec<MuscleGroup> {
        vec![
            MuscleGroup::Chest,
            MuscleGroup::Back,
            MuscleGroup::Shoulder,
            MuscleGroup::Bicep,
            MuscleGroup::Tricep,
            MuscleGroup::Forearm,
            MuscleGroup::Abs,
            MuscleGroup::Oblique,
            MuscleGroup::Glute,
            MuscleGroup::Hamstring,
            MuscleGroup::Quad,
            MuscleGroup::Calf,
            MuscleGroup::Neck,
            MuscleGroup::Face,
        ]
    }

    /// Returns the human-readable name of this muscle group.
    pub fn name(&self) -> &'static str {
        match self {
            MuscleGroup::Chest => "Chest",
            MuscleGroup::Back => "Back",
            MuscleGroup::Shoulder => "Shoulder",
            MuscleGroup::Bicep => "Bicep",
            MuscleGroup::Tricep => "Tricep",
            MuscleGroup::Forearm => "Forearm",
            MuscleGroup::Abs => "Abs",
            MuscleGroup::Oblique => "Oblique",
            MuscleGroup::Glute => "Glute",
            MuscleGroup::Hamstring => "Hamstring",
            MuscleGroup::Quad => "Quad",
            MuscleGroup::Calf => "Calf",
            MuscleGroup::Neck => "Neck",
            MuscleGroup::Face => "Face",
        }
    }
}

// ---------------------------------------------------------------------------
// Side
// ---------------------------------------------------------------------------

/// Lateral side discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Left,
    Right,
    Center,
}

// ---------------------------------------------------------------------------
// MuscleDefinition
// ---------------------------------------------------------------------------

/// A named muscle definition that drives one or more morph targets.
pub struct MuscleDefinition {
    /// Unique name for this muscle (e.g. "bicep_left").
    pub name: String,
    /// Anatomical group this muscle belongs to.
    pub group: MuscleGroup,
    /// Morph targets driven when this muscle flexes (0 = relaxed, 1 = full flex).
    /// Each entry is `(morph_name, max_weight)`.
    pub flex_morphs: Vec<(String, f32)>,
    /// Morph targets driven when contracted (e.g., shortened).
    pub contract_morphs: Vec<(String, f32)>,
    /// If `true`, this muscle has a left/right counterpart.
    pub symmetrical: bool,
    /// Which side this muscle is on, if applicable.
    pub side: Option<Side>,
    /// Reference length in normalised units (0..1).
    pub rest_length: f32,
}

impl MuscleDefinition {
    /// Convenience constructor.
    pub fn new(name: impl Into<String>, group: MuscleGroup) -> Self {
        Self {
            name: name.into(),
            group,
            flex_morphs: Vec::new(),
            contract_morphs: Vec::new(),
            symmetrical: false,
            side: None,
            rest_length: 1.0,
        }
    }

    /// Builder: add a flex morph.
    pub fn with_flex_morph(mut self, morph: impl Into<String>, max_weight: f32) -> Self {
        self.flex_morphs.push((morph.into(), max_weight));
        self
    }

    /// Builder: add a contract morph.
    pub fn with_contract_morph(mut self, morph: impl Into<String>, max_weight: f32) -> Self {
        self.contract_morphs.push((morph.into(), max_weight));
        self
    }

    /// Builder: set symmetry and side.
    pub fn with_side(mut self, side: Side) -> Self {
        self.symmetrical = true;
        self.side = Some(side);
        self
    }

    /// Builder: set rest length.
    pub fn with_rest_length(mut self, length: f32) -> Self {
        self.rest_length = length.clamp(0.0, 1.0);
        self
    }
}

// ---------------------------------------------------------------------------
// MuscleState
// ---------------------------------------------------------------------------

/// Current activation state of a single muscle.
#[derive(Debug, Clone)]
pub struct MuscleState {
    /// Flex activation: 0 = relaxed, 1 = fully flexed.
    pub flex: f32,
    /// Contraction: 0 = rest length, 1 = fully contracted.
    pub contract: f32,
    /// Fatigue: 0 = fresh, 1 = fatigued (reduces output).
    pub fatigue: f32,
}

impl Default for MuscleState {
    fn default() -> Self {
        Self {
            flex: 0.0,
            contract: 0.0,
            fatigue: 0.0,
        }
    }
}

impl MuscleState {
    /// Create a fully or partially flexed muscle state (no fatigue).
    pub fn flexed(v: f32) -> Self {
        Self {
            flex: v.clamp(0.0, 1.0),
            contract: 0.0,
            fatigue: 0.0,
        }
    }

    /// Create a completely relaxed muscle state.
    pub fn relaxed() -> Self {
        Self::default()
    }

    /// Effective flex output considering fatigue attenuation.
    ///
    /// Fatigue linearly reduces output: `effective = flex * (1 - fatigue)`.
    pub fn effective_flex(&self) -> f32 {
        (self.flex * (1.0 - self.fatigue)).clamp(0.0, 1.0)
    }

    /// Effective contract output considering fatigue attenuation.
    pub fn effective_contract(&self) -> f32 {
        (self.contract * (1.0 - self.fatigue)).clamp(0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// MuscleRig
// ---------------------------------------------------------------------------

/// A complete muscle rig: a collection of [`MuscleDefinition`]s with current
/// [`MuscleState`]s that can be evaluated into morph weights.
pub struct MuscleRig {
    muscles: Vec<MuscleDefinition>,
    states: HashMap<String, MuscleState>,
}

impl Default for MuscleRig {
    fn default() -> Self {
        Self::new()
    }
}

impl MuscleRig {
    /// Create an empty muscle rig.
    pub fn new() -> Self {
        Self {
            muscles: Vec::new(),
            states: HashMap::new(),
        }
    }

    /// Add a muscle definition to this rig.
    /// The state is initialised to the relaxed default.
    pub fn add_muscle(&mut self, def: MuscleDefinition) {
        let name = def.name.clone();
        self.muscles.push(def);
        self.states.entry(name).or_default();
    }

    /// Set the current state for a named muscle.
    pub fn set_state(&mut self, name: &str, state: MuscleState) {
        self.states.insert(name.to_owned(), state);
    }

    /// Get the current state for a named muscle, if it exists.
    pub fn get_state(&self, name: &str) -> Option<&MuscleState> {
        self.states.get(name)
    }

    /// Return all muscle names in definition order.
    pub fn muscle_names(&self) -> Vec<&str> {
        self.muscles.iter().map(|m| m.name.as_str()).collect()
    }

    /// Return all muscle definitions belonging to a given group.
    pub fn muscles_in_group(&self, group: &MuscleGroup) -> Vec<&MuscleDefinition> {
        self.muscles.iter().filter(|m| &m.group == group).collect()
    }

    /// Total number of muscles in this rig.
    pub fn count(&self) -> usize {
        self.muscles.len()
    }

    /// Evaluate all muscle states and accumulate morph weights.
    ///
    /// Multiple muscles can drive the same morph target; weights are additive
    /// and clamped to `[0, 1]` per morph.
    pub fn evaluate(&self) -> HashMap<String, f32> {
        let mut weights: HashMap<String, f32> = HashMap::new();

        for muscle in &self.muscles {
            let state = self.states.get(&muscle.name).cloned().unwrap_or_default();

            let eff_flex = state.effective_flex();
            let eff_contract = state.effective_contract();

            for (morph_name, max_weight) in &muscle.flex_morphs {
                let w = eff_flex * max_weight;
                let entry = weights.entry(morph_name.clone()).or_insert(0.0);
                *entry = (*entry + w).clamp(0.0, 1.0);
            }

            for (morph_name, max_weight) in &muscle.contract_morphs {
                let w = eff_contract * max_weight;
                let entry = weights.entry(morph_name.clone()).or_insert(0.0);
                *entry = (*entry + w).clamp(0.0, 1.0);
            }
        }

        weights
    }

    /// Flex all muscles in a group simultaneously to `amount` [0..1].
    pub fn flex_group(&mut self, group: &MuscleGroup, amount: f32) {
        let names: Vec<String> = self
            .muscles
            .iter()
            .filter(|m| &m.group == group)
            .map(|m| m.name.clone())
            .collect();

        let amount = amount.clamp(0.0, 1.0);
        for name in names {
            let state = self.states.entry(name).or_default();
            state.flex = amount;
        }
    }

    /// Set all muscle states to relaxed (zero flex, zero contract).
    pub fn relax_all(&mut self) {
        for state in self.states.values_mut() {
            state.flex = 0.0;
            state.contract = 0.0;
        }
    }

    /// Build a standard human muscle rig with approximately 20 named muscles.
    pub fn default_rig() -> Self {
        let mut rig = Self::new();

        // --- Chest ---
        rig.add_muscle(
            MuscleDefinition::new("pectoralis_major", MuscleGroup::Chest)
                .with_flex_morph("chest_flex", 1.0)
                .with_contract_morph("chest_contracted", 0.8)
                .with_rest_length(0.9),
        );

        // --- Back ---
        rig.add_muscle(
            MuscleDefinition::new("latissimus_dorsi", MuscleGroup::Back)
                .with_flex_morph("back_lat_flex", 1.0)
                .with_contract_morph("back_contracted", 0.7)
                .with_rest_length(0.95),
        );
        rig.add_muscle(
            MuscleDefinition::new("trapezius", MuscleGroup::Back)
                .with_flex_morph("trap_flex", 0.9)
                .with_rest_length(0.85),
        );

        // --- Shoulder ---
        rig.add_muscle(
            MuscleDefinition::new("deltoid_left", MuscleGroup::Shoulder)
                .with_flex_morph("shoulder_flex_left", 1.0)
                .with_side(Side::Left)
                .with_rest_length(0.8),
        );
        rig.add_muscle(
            MuscleDefinition::new("deltoid_right", MuscleGroup::Shoulder)
                .with_flex_morph("shoulder_flex_right", 1.0)
                .with_side(Side::Right)
                .with_rest_length(0.8),
        );

        // --- Bicep ---
        rig.add_muscle(
            MuscleDefinition::new("bicep_left", MuscleGroup::Bicep)
                .with_flex_morph("bicep_flex_left", 1.0)
                .with_contract_morph("bicep_contracted_left", 0.9)
                .with_side(Side::Left)
                .with_rest_length(0.75),
        );
        rig.add_muscle(
            MuscleDefinition::new("bicep_right", MuscleGroup::Bicep)
                .with_flex_morph("bicep_flex_right", 1.0)
                .with_contract_morph("bicep_contracted_right", 0.9)
                .with_side(Side::Right)
                .with_rest_length(0.75),
        );

        // --- Tricep ---
        rig.add_muscle(
            MuscleDefinition::new("tricep_left", MuscleGroup::Tricep)
                .with_flex_morph("tricep_flex_left", 0.85)
                .with_side(Side::Left)
                .with_rest_length(0.8),
        );
        rig.add_muscle(
            MuscleDefinition::new("tricep_right", MuscleGroup::Tricep)
                .with_flex_morph("tricep_flex_right", 0.85)
                .with_side(Side::Right)
                .with_rest_length(0.8),
        );

        // --- Forearm ---
        rig.add_muscle(
            MuscleDefinition::new("brachioradialis_left", MuscleGroup::Forearm)
                .with_flex_morph("forearm_flex_left", 0.7)
                .with_side(Side::Left)
                .with_rest_length(0.9),
        );
        rig.add_muscle(
            MuscleDefinition::new("brachioradialis_right", MuscleGroup::Forearm)
                .with_flex_morph("forearm_flex_right", 0.7)
                .with_side(Side::Right)
                .with_rest_length(0.9),
        );

        // --- Abs ---
        rig.add_muscle(
            MuscleDefinition::new("rectus_abdominis", MuscleGroup::Abs)
                .with_flex_morph("abs_flex", 1.0)
                .with_contract_morph("abs_crunch", 0.8)
                .with_rest_length(0.85),
        );

        // --- Oblique ---
        rig.add_muscle(
            MuscleDefinition::new("oblique_left", MuscleGroup::Oblique)
                .with_flex_morph("oblique_flex_left", 0.8)
                .with_side(Side::Left)
                .with_rest_length(0.9),
        );
        rig.add_muscle(
            MuscleDefinition::new("oblique_right", MuscleGroup::Oblique)
                .with_flex_morph("oblique_flex_right", 0.8)
                .with_side(Side::Right)
                .with_rest_length(0.9),
        );

        // --- Glute ---
        rig.add_muscle(
            MuscleDefinition::new("gluteus_maximus", MuscleGroup::Glute)
                .with_flex_morph("glute_flex", 1.0)
                .with_rest_length(0.95),
        );

        // --- Quad ---
        rig.add_muscle(
            MuscleDefinition::new("quadricep_left", MuscleGroup::Quad)
                .with_flex_morph("quad_flex_left", 1.0)
                .with_side(Side::Left)
                .with_rest_length(0.85),
        );
        rig.add_muscle(
            MuscleDefinition::new("quadricep_right", MuscleGroup::Quad)
                .with_flex_morph("quad_flex_right", 1.0)
                .with_side(Side::Right)
                .with_rest_length(0.85),
        );

        // --- Hamstring ---
        rig.add_muscle(
            MuscleDefinition::new("hamstring_left", MuscleGroup::Hamstring)
                .with_flex_morph("hamstring_flex_left", 0.9)
                .with_side(Side::Left)
                .with_rest_length(0.9),
        );

        // --- Calf ---
        rig.add_muscle(
            MuscleDefinition::new("gastrocnemius_left", MuscleGroup::Calf)
                .with_flex_morph("calf_flex_left", 0.9)
                .with_contract_morph("calf_contracted_left", 0.7)
                .with_side(Side::Left)
                .with_rest_length(0.8),
        );

        // --- Neck ---
        rig.add_muscle(
            MuscleDefinition::new("sternocleidomastoid", MuscleGroup::Neck)
                .with_flex_morph("neck_flex", 0.7)
                .with_rest_length(0.85),
        );

        rig
    }

    /// Apply fatigue to muscles whose flex exceeds `threshold`.
    ///
    /// Each qualifying muscle has its fatigue increased by `fatigue_rate`,
    /// clamped to `[0, 1]`.
    pub fn apply_fatigue(&mut self, threshold: f32, fatigue_rate: f32) {
        for state in self.states.values_mut() {
            if state.flex > threshold {
                state.fatigue = (state.fatigue + fatigue_rate).clamp(0.0, 1.0);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Linearly blend two rig state maps by factor `t` (0 = all `a`, 1 = all `b`).
///
/// Keys present in only one map are included with the missing side treated as
/// the default relaxed state.
pub fn blend_rig_states(
    a: &HashMap<String, MuscleState>,
    b: &HashMap<String, MuscleState>,
    t: f32,
) -> HashMap<String, MuscleState> {
    let t = t.clamp(0.0, 1.0);
    let one_minus_t = 1.0 - t;

    let mut result: HashMap<String, MuscleState> = HashMap::new();

    // Collect all keys from both maps
    let mut keys: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for k in a.keys() {
        keys.insert(k.as_str());
    }
    for k in b.keys() {
        keys.insert(k.as_str());
    }

    let default_state = MuscleState::default();

    for key in keys {
        let sa = a.get(key).unwrap_or(&default_state);
        let sb = b.get(key).unwrap_or(&default_state);

        result.insert(
            key.to_owned(),
            MuscleState {
                flex: (sa.flex * one_minus_t + sb.flex * t).clamp(0.0, 1.0),
                contract: (sa.contract * one_minus_t + sb.contract * t).clamp(0.0, 1.0),
                fatigue: (sa.fatigue * one_minus_t + sb.fatigue * t).clamp(0.0, 1.0),
            },
        );
    }

    result
}

/// Convenience wrapper: evaluate a [`MuscleRig`] into morph weights.
///
/// Identical to [`MuscleRig::evaluate`] but available as a free function for
/// ergonomic use in pipelines.
pub fn rig_to_morphs(rig: &MuscleRig) -> HashMap<String, f32> {
    rig.evaluate()
}

/// Estimate muscle activation from a normalised body parameter [0..1].
///
/// High `muscle_param` values (muscular body) result in higher activation,
/// modulated by group-specific sensitivity factors.
pub fn params_to_muscle_activation(muscle_param: f32, group: &MuscleGroup) -> f32 {
    let param = muscle_param.clamp(0.0, 1.0);

    // Group-specific sensitivity: some groups respond more strongly to the
    // general muscularity parameter.
    let sensitivity = match group {
        MuscleGroup::Bicep => 1.0,
        MuscleGroup::Tricep => 0.95,
        MuscleGroup::Chest => 0.9,
        MuscleGroup::Back => 0.9,
        MuscleGroup::Shoulder => 0.85,
        MuscleGroup::Forearm => 0.8,
        MuscleGroup::Abs => 0.85,
        MuscleGroup::Oblique => 0.75,
        MuscleGroup::Glute => 0.8,
        MuscleGroup::Quad => 0.9,
        MuscleGroup::Hamstring => 0.85,
        MuscleGroup::Calf => 0.8,
        MuscleGroup::Neck => 0.7,
        MuscleGroup::Face => 0.3,
    };

    (param * sensitivity).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Helper: write a string to a temp file in /tmp/
    fn write_tmp(filename: &str, content: &str) {
        fs::write(format!("/tmp/{}", filename), content).expect("write /tmp/ file");
    }

    // -----------------------------------------------------------------------
    // MuscleGroup tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_muscle_group_all_count() {
        let all = MuscleGroup::all();
        assert_eq!(all.len(), 14);
        write_tmp("muscle_group_all.txt", &format!("{} groups", all.len()));
    }

    #[test]
    fn test_muscle_group_names() {
        assert_eq!(MuscleGroup::Bicep.name(), "Bicep");
        assert_eq!(MuscleGroup::Face.name(), "Face");
        assert_eq!(MuscleGroup::Abs.name(), "Abs");
        write_tmp("muscle_group_names.txt", "OK");
    }

    #[test]
    fn test_muscle_group_all_unique_names() {
        let names: Vec<&str> = MuscleGroup::all().iter().map(|g| g.name()).collect();
        let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
        assert_eq!(names.len(), unique.len(), "all group names must be unique");
        write_tmp("muscle_group_unique.txt", "OK");
    }

    // -----------------------------------------------------------------------
    // MuscleState tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_muscle_state_default_is_relaxed() {
        let s = MuscleState::default();
        assert_eq!(s.flex, 0.0);
        assert_eq!(s.contract, 0.0);
        assert_eq!(s.fatigue, 0.0);
        write_tmp("muscle_state_default.txt", "OK");
    }

    #[test]
    fn test_muscle_state_flexed() {
        let s = MuscleState::flexed(0.7);
        assert!((s.flex - 0.7).abs() < 1e-6);
        assert_eq!(s.fatigue, 0.0);
        write_tmp("muscle_state_flexed.txt", "OK");
    }

    #[test]
    fn test_muscle_state_effective_flex_with_fatigue() {
        let mut s = MuscleState::flexed(1.0);
        s.fatigue = 0.5;
        let eff = s.effective_flex();
        assert!((eff - 0.5).abs() < 1e-6, "eff={eff}");
        write_tmp("muscle_state_fatigue.txt", &format!("eff={eff}"));
    }

    #[test]
    fn test_muscle_state_relaxed() {
        let s = MuscleState::relaxed();
        assert_eq!(s.effective_flex(), 0.0);
        write_tmp("muscle_state_relaxed.txt", "OK");
    }

    // -----------------------------------------------------------------------
    // MuscleDefinition tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_muscle_definition_builder() {
        let def = MuscleDefinition::new("bicep_test", MuscleGroup::Bicep)
            .with_flex_morph("flex_shape", 1.0)
            .with_contract_morph("contract_shape", 0.5)
            .with_side(Side::Left)
            .with_rest_length(0.8);

        assert_eq!(def.name, "bicep_test");
        assert_eq!(def.group, MuscleGroup::Bicep);
        assert_eq!(def.flex_morphs.len(), 1);
        assert_eq!(def.contract_morphs.len(), 1);
        assert!(def.symmetrical);
        assert_eq!(def.side, Some(Side::Left));
        assert!((def.rest_length - 0.8).abs() < 1e-6);
        write_tmp("muscle_definition_builder.txt", "OK");
    }

    // -----------------------------------------------------------------------
    // MuscleRig tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_rig_add_and_count() {
        let mut rig = MuscleRig::new();
        rig.add_muscle(MuscleDefinition::new("m1", MuscleGroup::Bicep));
        rig.add_muscle(MuscleDefinition::new("m2", MuscleGroup::Tricep));
        assert_eq!(rig.count(), 2);
        write_tmp("rig_count.txt", "OK");
    }

    #[test]
    fn test_rig_set_and_get_state() {
        let mut rig = MuscleRig::new();
        rig.add_muscle(MuscleDefinition::new("test_muscle", MuscleGroup::Abs));
        rig.set_state("test_muscle", MuscleState::flexed(0.6));
        let state = rig.get_state("test_muscle").expect("state must exist");
        assert!((state.flex - 0.6).abs() < 1e-6);
        write_tmp("rig_state.txt", "OK");
    }

    #[test]
    fn test_rig_evaluate_flex() {
        let mut rig = MuscleRig::new();
        rig.add_muscle(
            MuscleDefinition::new("bicep_l", MuscleGroup::Bicep)
                .with_flex_morph("bicep_shape", 1.0),
        );
        rig.set_state("bicep_l", MuscleState::flexed(0.8));
        let weights = rig.evaluate();
        let w = weights.get("bicep_shape").copied().unwrap_or(0.0);
        assert!((w - 0.8).abs() < 1e-6, "w={w}");
        write_tmp("rig_evaluate_flex.txt", &format!("w={w}"));
    }

    #[test]
    fn test_rig_evaluate_multiple_morphs() {
        let mut rig = MuscleRig::new();
        rig.add_muscle(
            MuscleDefinition::new("chest", MuscleGroup::Chest)
                .with_flex_morph("chest_shape_a", 0.5)
                .with_flex_morph("chest_shape_b", 0.8),
        );
        rig.set_state("chest", MuscleState::flexed(1.0));
        let weights = rig.evaluate();
        assert!((weights["chest_shape_a"] - 0.5).abs() < 1e-6);
        assert!((weights["chest_shape_b"] - 0.8).abs() < 1e-6);
        write_tmp("rig_evaluate_multi.txt", "OK");
    }

    #[test]
    fn test_rig_relax_all() {
        let mut rig = MuscleRig::new();
        rig.add_muscle(MuscleDefinition::new("m1", MuscleGroup::Glute));
        rig.set_state("m1", MuscleState::flexed(1.0));
        rig.relax_all();
        let state = rig.get_state("m1").expect("state");
        assert_eq!(state.flex, 0.0);
        write_tmp("rig_relax_all.txt", "OK");
    }

    #[test]
    fn test_rig_muscles_in_group() {
        let rig = MuscleRig::default_rig();
        let biceps = rig.muscles_in_group(&MuscleGroup::Bicep);
        assert!(!biceps.is_empty(), "default rig should have biceps");
        write_tmp("rig_in_group.txt", &format!("biceps={}", biceps.len()));
    }

    #[test]
    fn test_rig_flex_group() {
        let mut rig = MuscleRig::default_rig();
        rig.flex_group(&MuscleGroup::Bicep, 1.0);
        let bicep_names: Vec<String> = rig
            .muscles_in_group(&MuscleGroup::Bicep)
            .iter()
            .map(|m| m.name.clone())
            .collect();
        for name in &bicep_names {
            let state = rig.get_state(name).expect("state");
            assert!((state.flex - 1.0).abs() < 1e-6, "{name} flex should be 1.0");
        }
        write_tmp("rig_flex_group.txt", "OK");
    }

    #[test]
    fn test_rig_default_count() {
        let rig = MuscleRig::default_rig();
        assert!(
            rig.count() >= 20,
            "default rig must have ≥20 muscles, got {}",
            rig.count()
        );
        write_tmp("rig_default_count.txt", &format!("{}", rig.count()));
    }

    #[test]
    fn test_rig_apply_fatigue() {
        let mut rig = MuscleRig::new();
        rig.add_muscle(MuscleDefinition::new("fatigued", MuscleGroup::Quad));
        rig.set_state("fatigued", MuscleState::flexed(1.0));
        rig.apply_fatigue(0.5, 0.3);
        let state = rig.get_state("fatigued").expect("state");
        assert!(
            (state.fatigue - 0.3).abs() < 1e-6,
            "fatigue={}",
            state.fatigue
        );
        write_tmp("rig_apply_fatigue.txt", "OK");
    }

    #[test]
    fn test_rig_apply_fatigue_below_threshold() {
        let mut rig = MuscleRig::new();
        rig.add_muscle(MuscleDefinition::new("resting", MuscleGroup::Calf));
        rig.set_state("resting", MuscleState::flexed(0.2));
        rig.apply_fatigue(0.5, 0.3);
        let state = rig.get_state("resting").expect("state");
        assert_eq!(
            state.fatigue, 0.0,
            "below-threshold muscle must not fatigue"
        );
        write_tmp("rig_no_fatigue.txt", "OK");
    }

    // -----------------------------------------------------------------------
    // blend_rig_states tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_blend_rig_states_midpoint() {
        let mut a: HashMap<String, MuscleState> = HashMap::new();
        a.insert(
            "m".to_owned(),
            MuscleState {
                flex: 0.0,
                contract: 0.0,
                fatigue: 0.0,
            },
        );
        let mut b: HashMap<String, MuscleState> = HashMap::new();
        b.insert(
            "m".to_owned(),
            MuscleState {
                flex: 1.0,
                contract: 0.0,
                fatigue: 0.0,
            },
        );

        let blended = blend_rig_states(&a, &b, 0.5);
        let s = &blended["m"];
        assert!((s.flex - 0.5).abs() < 1e-6, "flex={}", s.flex);
        write_tmp("blend_states_midpoint.txt", "OK");
    }

    #[test]
    fn test_blend_rig_states_t0_equals_a() {
        let mut a: HashMap<String, MuscleState> = HashMap::new();
        a.insert("x".to_owned(), MuscleState::flexed(0.7));
        let b: HashMap<String, MuscleState> = HashMap::new();

        let blended = blend_rig_states(&a, &b, 0.0);
        let s = &blended["x"];
        assert!((s.flex - 0.7).abs() < 1e-6);
        write_tmp("blend_states_t0.txt", "OK");
    }

    #[test]
    fn test_blend_rig_states_missing_key_treated_as_default() {
        let a: HashMap<String, MuscleState> = HashMap::new();
        let mut b: HashMap<String, MuscleState> = HashMap::new();
        b.insert("only_in_b".to_owned(), MuscleState::flexed(1.0));

        let blended = blend_rig_states(&a, &b, 1.0);
        let s = &blended["only_in_b"];
        assert!((s.flex - 1.0).abs() < 1e-6);
        write_tmp("blend_missing_key.txt", "OK");
    }

    // -----------------------------------------------------------------------
    // rig_to_morphs / params_to_muscle_activation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_rig_to_morphs_convenience() {
        let mut rig = MuscleRig::new();
        rig.add_muscle(
            MuscleDefinition::new("neck", MuscleGroup::Neck).with_flex_morph("neck_shape", 0.6),
        );
        rig.set_state("neck", MuscleState::flexed(1.0));
        let morphs = rig_to_morphs(&rig);
        assert!((morphs["neck_shape"] - 0.6).abs() < 1e-6);
        write_tmp("rig_to_morphs.txt", "OK");
    }

    #[test]
    fn test_params_to_activation_range() {
        for group in MuscleGroup::all() {
            let low = params_to_muscle_activation(0.0, &group);
            let high = params_to_muscle_activation(1.0, &group);
            assert!((0.0..=1.0).contains(&low), "{} low={low}", group.name());
            assert!((0.0..=1.0).contains(&high), "{} high={high}", group.name());
        }
        write_tmp("params_activation_range.txt", "OK");
    }

    #[test]
    fn test_params_activation_monotone() {
        // Higher muscle_param must produce >= activation across all groups
        for group in MuscleGroup::all() {
            let a = params_to_muscle_activation(0.3, &group);
            let b = params_to_muscle_activation(0.8, &group);
            assert!(b >= a, "{} must be monotone (a={a} b={b})", group.name());
        }
        write_tmp("params_activation_monotone.txt", "OK");
    }

    #[test]
    fn test_muscle_names_list() {
        let rig = MuscleRig::default_rig();
        let names = rig.muscle_names();
        assert_eq!(names.len(), rig.count());
        assert!(
            names.contains(&"bicep_left"),
            "expected bicep_left in names"
        );
        write_tmp("muscle_names_list.txt", &names.join("\n"));
    }

    #[test]
    fn test_evaluate_relaxed_rig_all_zero() {
        let rig = MuscleRig::default_rig();
        // Default rig starts relaxed
        let weights = rig.evaluate();
        for (k, v) in &weights {
            assert_eq!(*v, 0.0, "relaxed rig: {k} should be 0 but is {v}");
        }
        write_tmp("evaluate_relaxed_all_zero.txt", "OK");
    }

    #[test]
    fn test_side_equality() {
        assert_eq!(Side::Left, Side::Left);
        assert_ne!(Side::Left, Side::Right);
        assert_ne!(Side::Center, Side::Left);
        write_tmp("side_equality.txt", "OK");
    }
}
