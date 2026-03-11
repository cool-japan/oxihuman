// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashMap;

use crate::params::ParamState;

/// Body regions that can have independent morphing parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BodyRegion {
    Head,
    Neck,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
    Hands,
    Feet,
}

impl BodyRegion {
    /// Returns all body regions in anatomical order (head-to-toe).
    pub fn all() -> &'static [BodyRegion] {
        &[
            BodyRegion::Head,
            BodyRegion::Neck,
            BodyRegion::Torso,
            BodyRegion::LeftArm,
            BodyRegion::RightArm,
            BodyRegion::LeftLeg,
            BodyRegion::RightLeg,
            BodyRegion::Hands,
            BodyRegion::Feet,
        ]
    }

    /// Returns the region's display name.
    pub fn name(&self) -> &'static str {
        match self {
            BodyRegion::Head => "Head",
            BodyRegion::Neck => "Neck",
            BodyRegion::Torso => "Torso",
            BodyRegion::LeftArm => "Left Arm",
            BodyRegion::RightArm => "Right Arm",
            BodyRegion::LeftLeg => "Left Leg",
            BodyRegion::RightLeg => "Right Leg",
            BodyRegion::Hands => "Hands",
            BodyRegion::Feet => "Feet",
        }
    }
}

/// Per-region overrides for morph parameters.
/// If a region has no override, it inherits from the global ParamState.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RegionParams {
    pub overrides: HashMap<BodyRegion, ParamState>,
}

fn lerp_param(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn blend_param_state(override_state: &ParamState, global: &ParamState, t: f32) -> ParamState {
    let height = lerp_param(override_state.height, global.height, t);
    let weight = lerp_param(override_state.weight, global.weight, t);
    let muscle = lerp_param(override_state.muscle, global.muscle, t);
    let age = lerp_param(override_state.age, global.age, t);

    let mut extra: HashMap<String, f32> = HashMap::new();

    // Keys present in the override: blend toward global (or toward 0.0 if not in global)
    for (k, &ov) in &override_state.extra {
        let target = global.extra.get(k).copied().unwrap_or(0.0);
        extra.insert(k.clone(), lerp_param(ov, target, t));
    }

    // Keys present only in global but not in override: blend from 0.0 toward global value
    for (k, &gv) in &global.extra {
        if !override_state.extra.contains_key(k) {
            extra.insert(k.clone(), lerp_param(0.0, gv, t));
        }
    }

    ParamState {
        height,
        weight,
        muscle,
        age,
        extra,
    }
}

impl RegionParams {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set an override for a specific region.
    pub fn set_region(&mut self, region: BodyRegion, params: ParamState) {
        self.overrides.insert(region, params);
    }

    /// Get the effective ParamState for a region, falling back to `global` if no override.
    pub fn effective_params(&self, region: BodyRegion, global: &ParamState) -> ParamState {
        self.overrides
            .get(&region)
            .cloned()
            .unwrap_or_else(|| global.clone())
    }

    /// Clear the override for a region (revert to global).
    pub fn clear_region(&mut self, region: BodyRegion) {
        self.overrides.remove(&region);
    }

    /// Returns true if any region has an override.
    pub fn has_overrides(&self) -> bool {
        !self.overrides.is_empty()
    }

    /// Blend all region overrides toward global by `t` (0.0 = keep overrides, 1.0 = full global).
    /// Useful for smooth transitions.
    pub fn blend_toward_global(&self, global: &ParamState, t: f32) -> RegionParams {
        let mut result = RegionParams::new();
        for (&region, override_state) in &self.overrides {
            let blended = blend_param_state(override_state, global, t);
            result.overrides.insert(region, blended);
        }
        result
    }
}

/// A morph target tag associating it with one or more body regions.
/// Targets are tagged so the engine knows which region's params to use.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegionTag {
    pub target_name: String,
    pub regions: Vec<BodyRegion>,
}

impl RegionTag {
    pub fn new(target_name: impl Into<String>, regions: Vec<BodyRegion>) -> Self {
        RegionTag {
            target_name: target_name.into(),
            regions,
        }
    }

    /// Infer regions from a target name (MakeHuman naming convention).
    /// e.g. "head/head-age-*.target" → `[Head]`, "l-arm/*" → `[LeftArm]`, etc.
    pub fn infer_from_name(target_name: &str) -> Self {
        let lower = target_name.to_lowercase();
        let mut regions = Vec::new();

        // Head-related keywords
        if lower.contains("head")
            || lower.contains("face")
            || lower.contains("eye")
            || lower.contains("nose")
            || lower.contains("mouth")
            || lower.contains("ear")
        {
            regions.push(BodyRegion::Head);
        }

        // Neck
        if lower.contains("neck") {
            regions.push(BodyRegion::Neck);
        }

        // Torso
        if lower.contains("torso")
            || lower.contains("chest")
            || lower.contains("belly")
            || lower.contains("back")
            || lower.contains("waist")
        {
            regions.push(BodyRegion::Torso);
        }

        // Arms — check more specific before generic
        if lower.contains("l-arm")
            || lower.contains("larm")
            || lower.contains("left-arm")
            || lower.contains("leftarm")
        {
            regions.push(BodyRegion::LeftArm);
        }
        if lower.contains("r-arm")
            || lower.contains("rarm")
            || lower.contains("right-arm")
            || lower.contains("rightarm")
        {
            regions.push(BodyRegion::RightArm);
        }

        // Legs
        if lower.contains("l-leg")
            || lower.contains("lleg")
            || lower.contains("left-leg")
            || lower.contains("leftleg")
        {
            regions.push(BodyRegion::LeftLeg);
        }
        if lower.contains("r-leg")
            || lower.contains("rleg")
            || lower.contains("right-leg")
            || lower.contains("rightleg")
        {
            regions.push(BodyRegion::RightLeg);
        }

        // Hands
        if lower.contains("hand") {
            regions.push(BodyRegion::Hands);
        }

        // Feet
        if lower.contains("foot") || lower.contains("feet") {
            regions.push(BodyRegion::Feet);
        }

        // Default to Torso if nothing matched
        if regions.is_empty() {
            regions.push(BodyRegion::Torso);
        }

        RegionTag {
            target_name: target_name.to_string(),
            regions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_regions_has_nine() {
        assert_eq!(BodyRegion::all().len(), 9);
    }

    #[test]
    fn effective_params_falls_back_to_global() {
        let rp = RegionParams::new();
        let global = ParamState::new(0.7, 0.3, 0.6, 0.4);
        let effective = rp.effective_params(BodyRegion::Head, &global);
        assert_eq!(effective, global);
    }

    #[test]
    fn effective_params_uses_override() {
        let mut rp = RegionParams::new();
        let global = ParamState::new(0.5, 0.5, 0.5, 0.5);
        let head_override = ParamState::new(0.9, 0.1, 0.8, 0.2);
        rp.set_region(BodyRegion::Head, head_override.clone());

        let effective_head = rp.effective_params(BodyRegion::Head, &global);
        assert_eq!(effective_head, head_override);

        // Other regions still fall back to global
        let effective_torso = rp.effective_params(BodyRegion::Torso, &global);
        assert_eq!(effective_torso, global);
    }

    #[test]
    fn clear_region_reverts_to_global() {
        let mut rp = RegionParams::new();
        let global = ParamState::new(0.5, 0.5, 0.5, 0.5);
        let head_override = ParamState::new(0.9, 0.1, 0.8, 0.2);
        rp.set_region(BodyRegion::Head, head_override);
        rp.clear_region(BodyRegion::Head);

        let effective = rp.effective_params(BodyRegion::Head, &global);
        assert_eq!(effective, global);
    }

    #[test]
    fn has_overrides_false_when_empty() {
        let rp = RegionParams::new();
        assert!(!rp.has_overrides());
    }

    #[test]
    fn blend_toward_global_at_t1_equals_global() {
        let mut rp = RegionParams::new();
        let global = ParamState::new(0.5, 0.5, 0.5, 0.5);
        let head_override = ParamState::new(0.9, 0.1, 0.8, 0.2);
        rp.set_region(BodyRegion::Head, head_override);

        let blended = rp.blend_toward_global(&global, 1.0);
        let effective = blended.effective_params(BodyRegion::Head, &global);

        assert!((effective.height - global.height).abs() < 1e-6);
        assert!((effective.weight - global.weight).abs() < 1e-6);
        assert!((effective.muscle - global.muscle).abs() < 1e-6);
        assert!((effective.age - global.age).abs() < 1e-6);
    }

    #[test]
    fn infer_from_name_head() {
        let tag = RegionTag::infer_from_name("head/head-age-young.target");
        assert!(tag.regions.contains(&BodyRegion::Head));
    }

    #[test]
    fn infer_from_name_default_torso() {
        let tag = RegionTag::infer_from_name("other/unknown.target");
        assert!(tag.regions.contains(&BodyRegion::Torso));
    }

    #[test]
    fn region_params_serialization() {
        let mut rp = RegionParams::new();
        let global = ParamState::new(0.5, 0.5, 0.5, 0.5);
        let head_override = ParamState::new(0.9, 0.1, 0.8, 0.2);
        rp.set_region(BodyRegion::Head, head_override);

        let json = serde_json::to_string(&rp).expect("serialize");
        let deserialized: RegionParams = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(rp.has_overrides(), deserialized.has_overrides());
        let effective_orig = rp.effective_params(BodyRegion::Head, &global);
        let effective_deser = deserialized.effective_params(BodyRegion::Head, &global);
        assert!((effective_orig.height - effective_deser.height).abs() < 1e-6);
    }
}
