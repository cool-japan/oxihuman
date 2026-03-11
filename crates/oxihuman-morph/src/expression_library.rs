// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Named facial expression preset library.
//!
//! Provides a collection of morph-target weight maps for common facial expressions,
//! along with blending, combining, and nearest-neighbour search utilities.

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ExpressionPreset
// ---------------------------------------------------------------------------

/// A named facial expression preset: a set of morph target weights.
#[derive(Debug, Clone)]
pub struct ExpressionPreset {
    /// Unique expression name (e.g. "smile", "anger").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Morph target name → base weight [0..1].
    pub weights: HashMap<String, f32>,
    /// Global intensity scale [0..1] applied on top of individual weights.
    pub intensity: f32,
    /// Semantic tags, e.g. `["happy", "positive", "mouth"]`.
    pub tags: Vec<String>,
}

impl ExpressionPreset {
    /// Create a new preset with intensity 1.0 and no tags.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        weights: HashMap<String, f32>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            weights,
            intensity: 1.0,
            tags: Vec::new(),
        }
    }

    /// Builder: set intensity.
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.clamp(0.0, 1.0);
        self
    }

    /// Builder: set tags.
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }
}

// ---------------------------------------------------------------------------
// ExpressionLibrary
// ---------------------------------------------------------------------------

/// A library of named expression presets.
pub struct ExpressionLibrary {
    presets: HashMap<String, ExpressionPreset>,
}

impl ExpressionLibrary {
    /// Create an empty library.
    pub fn new() -> Self {
        Self {
            presets: HashMap::new(),
        }
    }

    /// Add a preset to the library (keyed by `preset.name`).
    pub fn add(&mut self, preset: ExpressionPreset) {
        self.presets.insert(preset.name.clone(), preset);
    }

    /// Look up a preset by name.
    pub fn get(&self, name: &str) -> Option<&ExpressionPreset> {
        self.presets.get(name)
    }

    /// Return sorted list of all preset names.
    pub fn list_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.presets.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
    }

    /// Return all presets that carry the given tag.
    pub fn list_by_tag(&self, tag: &str) -> Vec<&ExpressionPreset> {
        let mut result: Vec<&ExpressionPreset> = self
            .presets
            .values()
            .filter(|p| p.tags.iter().any(|t| t == tag))
            .collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    /// Number of presets in the library.
    pub fn count(&self) -> usize {
        self.presets.len()
    }

    // -----------------------------------------------------------------------
    // Default library
    // -----------------------------------------------------------------------

    /// Build a default library with ~11 common facial expressions.
    pub fn default_library() -> Self {
        let mut lib = Self::new();

        // Helper macro to reduce boilerplate
        macro_rules! preset {
            ($name:expr, $desc:expr, $intensity:expr, $tags:expr, $($key:expr => $val:expr),* $(,)?) => {{
                #[allow(unused_mut)]
                let mut w = HashMap::new();
                $( w.insert($key.to_string(), $val as f32); )*
                ExpressionPreset::new($name, $desc, w)
                    .with_intensity($intensity)
                    .with_tags($tags)
            }};
        }

        // neutral
        lib.add(preset!(
            "neutral",
            "Relaxed, expressionless face",
            1.0,
            ["neutral", "base"],
            // No morphs active — all at zero is represented by absence of keys
        ));

        // smile
        lib.add(preset!(
            "smile", "Genuine open smile with cheek raise", 1.0,
            ["happy", "positive", "mouth", "cheek"],
            "mouth_smile_L"   => 0.9_f32,
            "mouth_smile_R"   => 0.9_f32,
            "cheek_raise_L"   => 0.6_f32,
            "cheek_raise_R"   => 0.6_f32,
            "lip_upper_raise" => 0.3_f32,
        ));

        // frown
        lib.add(preset!(
            "frown", "Downturned mouth with furrowed brows", 1.0,
            ["sad", "negative", "mouth", "brow"],
            "mouth_frown_L"   => 0.8_f32,
            "mouth_frown_R"   => 0.8_f32,
            "brow_lower_L"    => 0.5_f32,
            "brow_lower_R"    => 0.5_f32,
            "lip_lower_drop"  => 0.2_f32,
        ));

        // surprise
        lib.add(preset!(
            "surprise", "Wide eyes and open mouth", 1.0,
            ["surprise", "positive", "mouth", "eye", "brow"],
            "brow_raise_L"    => 0.9_f32,
            "brow_raise_R"    => 0.9_f32,
            "eye_wide_L"      => 0.8_f32,
            "eye_wide_R"      => 0.8_f32,
            "jaw_open"        => 0.7_f32,
            "lip_upper_raise" => 0.4_f32,
            "lip_lower_drop"  => 0.4_f32,
        ));

        // anger
        lib.add(preset!(
            "anger", "Furrowed brows and compressed lips", 1.0,
            ["angry", "negative", "brow", "mouth"],
            "brow_lower_L"       => 0.8_f32,
            "brow_lower_R"       => 0.8_f32,
            "brow_inner_raise_L" => 0.4_f32,
            "brow_inner_raise_R" => 0.4_f32,
            "nose_wrinkle"       => 0.3_f32,
            "mouth_compress"     => 0.7_f32,
            "jaw_clench"         => 0.5_f32,
        ));

        // disgust
        lib.add(preset!(
            "disgust", "Nose wrinkle and upper lip curl", 1.0,
            ["disgust", "negative", "nose", "mouth"],
            "nose_wrinkle"    => 0.8_f32,
            "lip_upper_raise" => 0.6_f32,
            "mouth_frown_L"   => 0.4_f32,
            "mouth_frown_R"   => 0.4_f32,
            "brow_lower_L"    => 0.3_f32,
            "brow_lower_R"    => 0.3_f32,
        ));

        // fear
        lib.add(preset!(
            "fear", "Raised brows and parted lips", 1.0,
            ["fear", "negative", "brow", "mouth", "eye"],
            "brow_raise_L"       => 0.7_f32,
            "brow_raise_R"       => 0.7_f32,
            "brow_inner_raise_L" => 0.8_f32,
            "brow_inner_raise_R" => 0.8_f32,
            "eye_wide_L"         => 0.6_f32,
            "eye_wide_R"         => 0.6_f32,
            "jaw_open"           => 0.3_f32,
            "lip_stretch_L"      => 0.5_f32,
            "lip_stretch_R"      => 0.5_f32,
        ));

        // contempt
        lib.add(preset!(
            "contempt", "One-sided mouth raise (sneer)", 1.0,
            ["contempt", "negative", "mouth"],
            "mouth_smile_L"   => 0.5_f32,
            "mouth_frown_R"   => 0.3_f32,
            "brow_raise_L"    => 0.2_f32,
            "brow_lower_R"    => 0.3_f32,
        ));

        // blink
        lib.add(preset!(
            "blink", "Both eyelids fully closed", 1.0,
            ["blink", "eye"],
            "eye_close_L"  => 1.0_f32,
            "eye_close_R"  => 1.0_f32,
        ));

        // wink_left
        lib.add(preset!(
            "wink_left", "Left eye wink (right eye open)", 1.0,
            ["wink", "eye", "left"],
            "eye_close_L"  => 1.0_f32,
            "eye_close_R"  => 0.0_f32,
        ));

        // wink_right
        lib.add(preset!(
            "wink_right", "Right eye wink (left eye open)", 1.0,
            ["wink", "eye", "right"],
            "eye_close_L"  => 0.0_f32,
            "eye_close_R"  => 1.0_f32,
        ));

        lib
    }

    // -----------------------------------------------------------------------
    // Blending utilities
    // -----------------------------------------------------------------------

    /// Linearly interpolate morph weights between two named presets.
    ///
    /// `t = 0.0` → all weights from `name_a`; `t = 1.0` → all weights from `name_b`.
    /// Keys not present in a preset are treated as weight 0.0.
    pub fn blend(&self, name_a: &str, name_b: &str, t: f32) -> Option<HashMap<String, f32>> {
        let a = self.presets.get(name_a)?;
        let b = self.presets.get(name_b)?;
        Some(lerp_weight_maps(&a.weights, &b.weights, t))
    }

    /// Scale a preset's weights by its `intensity` field.
    pub fn apply_intensity(preset: &ExpressionPreset) -> HashMap<String, f32> {
        preset
            .weights
            .iter()
            .map(|(k, &v)| (k.clone(), (v * preset.intensity).clamp(0.0, 1.0)))
            .collect()
    }

    /// Additively combine multiple presets (clamped to [0, 1]).
    pub fn combine(presets: &[&ExpressionPreset]) -> HashMap<String, f32> {
        let mut result: HashMap<String, f32> = HashMap::new();
        for preset in presets {
            for (key, &val) in &preset.weights {
                let entry = result.entry(key.clone()).or_insert(0.0);
                *entry = (*entry + val * preset.intensity).clamp(0.0, 1.0);
            }
        }
        result
    }

    /// Generate a pseudo-random blend of 2–3 presets from the given slice.
    ///
    /// Uses a simple LCG seeded by `seed` to pick indices and mixing weights.
    pub fn random_blend(presets: &[&ExpressionPreset], seed: u32) -> HashMap<String, f32> {
        if presets.is_empty() {
            return HashMap::new();
        }
        if presets.len() == 1 {
            return presets[0].weights.clone();
        }

        // LCG random
        let mut state = seed.wrapping_add(1);
        let mut lcg = || -> f32 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            (state >> 16) as f32 / 65535.0
        };

        // Pick 2 or 3 presets
        let n = if presets.len() >= 3 && lcg() > 0.5 {
            3
        } else {
            2
        };
        let n = n.min(presets.len());

        // Pick distinct indices
        let mut indices: Vec<usize> = Vec::with_capacity(n);
        while indices.len() < n {
            let idx = (lcg() * presets.len() as f32) as usize % presets.len();
            if !indices.contains(&idx) {
                indices.push(idx);
            }
        }

        // Random mixing weights (normalised)
        let mut raw_weights: Vec<f32> = (0..n).map(|_| lcg() + 0.1).collect();
        let total: f32 = raw_weights.iter().sum();
        for w in &mut raw_weights {
            *w /= total;
        }

        // Weighted sum
        let mut result: HashMap<String, f32> = HashMap::new();
        for (i, &preset_idx) in indices.iter().enumerate() {
            let mix = raw_weights[i];
            for (key, &val) in &presets[preset_idx].weights {
                let entry = result.entry(key.clone()).or_insert(0.0);
                *entry = (*entry + val * mix).clamp(0.0, 1.0);
            }
        }
        result
    }

    /// Find the preset most similar to the given weight map (L2 distance).
    ///
    /// Returns `None` if the library is empty.
    pub fn find_nearest<'a>(&'a self, weights: &HashMap<String, f32>) -> Option<&'a str> {
        self.presets
            .values()
            .min_by(|a, b| {
                let da = expression_distance(&a.weights, weights);
                let db = expression_distance(&b.weights, weights);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.name.as_str())
    }
}

impl Default for ExpressionLibrary {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// expression_distance
// ---------------------------------------------------------------------------

/// L2 distance between two morph weight maps over the union of their keys.
///
/// Keys absent from a map contribute a value of 0.0.
pub fn expression_distance(a: &HashMap<String, f32>, b: &HashMap<String, f32>) -> f32 {
    let mut sum_sq = 0.0f32;

    // Keys in a
    for (k, &va) in a {
        let vb = b.get(k).copied().unwrap_or(0.0);
        let d = va - vb;
        sum_sq += d * d;
    }
    // Keys only in b
    for (k, &vb) in b {
        if !a.contains_key(k) {
            sum_sq += vb * vb;
        }
    }
    sum_sq.sqrt()
}

// ---------------------------------------------------------------------------
// ExpressionBlender
// ---------------------------------------------------------------------------

/// Interpolate between multiple named expressions using (name, weight) anchors.
///
/// This is a generalised barycentric blender: each anchor contributes its
/// preset's weights scaled by the anchor weight, then all contributions are
/// summed and clamped.
#[derive(Debug, Clone)]
pub struct ExpressionBlender {
    /// (preset name, blending weight) pairs.
    pub anchors: Vec<(String, f32)>,
}

impl ExpressionBlender {
    /// Create a new blender with no anchors.
    pub fn new() -> Self {
        Self {
            anchors: Vec::new(),
        }
    }

    /// Add an anchor (preset name, weight).
    pub fn add_anchor(&mut self, name: String, weight: f32) {
        self.anchors.push((name, weight));
    }

    /// Evaluate the blend against the given library.
    ///
    /// Missing presets are silently skipped. Result is clamped to [0, 1].
    pub fn evaluate(&self, library: &ExpressionLibrary) -> HashMap<String, f32> {
        let mut result: HashMap<String, f32> = HashMap::new();
        for (name, anchor_w) in &self.anchors {
            if let Some(preset) = library.get(name) {
                for (key, &val) in &preset.weights {
                    let entry = result.entry(key.clone()).or_insert(0.0);
                    *entry = (*entry + val * preset.intensity * anchor_w).clamp(0.0, 1.0);
                }
            }
        }
        result
    }

    /// Normalise anchor weights so they sum to 1.0.
    ///
    /// If the total is zero the weights are left unchanged.
    pub fn normalize(&mut self) {
        let total: f32 = self.anchors.iter().map(|(_, w)| w).sum();
        if total > 0.0 {
            for (_, w) in &mut self.anchors {
                *w /= total;
            }
        }
    }
}

impl Default for ExpressionBlender {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ExpressionLibConfig
// ---------------------------------------------------------------------------

/// Configuration for the expression preset library.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExpressionLibConfig {
    /// Maximum number of presets allowed in the library (0 = unlimited).
    pub max_presets: usize,
    /// Whether to allow overwriting presets with the same name.
    pub allow_overwrite: bool,
    /// Default intensity applied to newly added presets.
    pub default_intensity: f32,
}

/// Return a sensible default `ExpressionLibConfig`.
#[allow(dead_code)]
pub fn default_library_config() -> ExpressionLibConfig {
    ExpressionLibConfig {
        max_presets: 0,
        allow_overwrite: true,
        default_intensity: 1.0,
    }
}

/// Create an empty expression library.
#[allow(dead_code)]
pub fn new_expression_library() -> ExpressionLibrary {
    ExpressionLibrary::new()
}

/// Add a preset to the library (keyed by `preset.name`).
#[allow(dead_code)]
pub fn add_preset(lib: &mut ExpressionLibrary, preset: ExpressionPreset) {
    lib.add(preset);
}

/// Remove a preset by name. Returns `true` if it was present.
#[allow(dead_code)]
pub fn remove_preset(lib: &mut ExpressionLibrary, name: &str) -> bool {
    lib.presets.remove(name).is_some()
}

/// Get a preset by name.
#[allow(dead_code)]
pub fn get_preset<'a>(lib: &'a ExpressionLibrary, name: &str) -> Option<&'a ExpressionPreset> {
    lib.get(name)
}

/// Number of presets in the library.
#[allow(dead_code)]
pub fn preset_count(lib: &ExpressionLibrary) -> usize {
    lib.count()
}

/// Find a preset whose name contains `substring` (case-insensitive). Returns first match.
#[allow(dead_code)]
pub fn find_preset_by_name<'a>(
    lib: &'a ExpressionLibrary,
    substring: &str,
) -> Option<&'a ExpressionPreset> {
    let lower = substring.to_lowercase();
    lib.presets
        .values()
        .find(|p| p.name.to_lowercase().contains(&lower))
}

/// Linearly blend two named presets by `t` [0..1].
#[allow(dead_code)]
pub fn blend_presets(
    lib: &ExpressionLibrary,
    name_a: &str,
    name_b: &str,
    t: f32,
) -> Option<HashMap<String, f32>> {
    lib.blend(name_a, name_b, t)
}

/// Serialize the library to a simple JSON-like string.
#[allow(dead_code)]
pub fn library_to_json(lib: &ExpressionLibrary) -> String {
    let mut out = String::from("{\"presets\":[");
    let names = lib.list_names();
    for (i, name) in names.iter().enumerate() {
        let Some(p) = lib.get(name) else { continue };
        out.push_str(&format!(
            "{{\"name\":\"{}\",\"description\":\"{}\",\"intensity\":{:.4},\"weight_count\":{}}}",
            p.name,
            p.description,
            p.intensity,
            p.weights.len(),
        ));
        if i + 1 < names.len() {
            out.push(',');
        }
    }
    out.push_str("]}");
    out
}

/// Build a basic library with 7 core emotion presets.
#[allow(dead_code)]
pub fn build_basic_library() -> ExpressionLibrary {
    let mut lib = ExpressionLibrary::new();

    macro_rules! p {
        ($name:expr, $desc:expr, $tags:expr, $($k:expr => $v:expr),* $(,)?) => {{
            #[allow(unused_mut)]
            let mut w = HashMap::new();
            $( w.insert($k.to_string(), $v as f32); )*
            ExpressionPreset::new($name, $desc, w).with_tags($tags)
        }};
    }

    lib.add(p!("neutral", "Neutral resting expression", ["neutral"],));
    lib.add(p!(
        "happy", "Happiness / joy", ["happy", "positive"],
        "mouth_smile_L" => 0.9_f32,
        "mouth_smile_R" => 0.9_f32,
        "cheek_raise_L" => 0.55_f32,
        "cheek_raise_R" => 0.55_f32,
    ));
    lib.add(p!(
        "sad", "Sadness / sorrow", ["sad", "negative"],
        "mouth_frown_L"  => 0.8_f32,
        "mouth_frown_R"  => 0.8_f32,
        "brow_lower_L"   => 0.4_f32,
        "brow_lower_R"   => 0.4_f32,
        "lip_lower_drop" => 0.2_f32,
    ));
    lib.add(p!(
        "angry", "Anger / aggression", ["angry", "negative"],
        "brow_lower_L"   => 0.85_f32,
        "brow_lower_R"   => 0.85_f32,
        "nose_wrinkle"   => 0.3_f32,
        "mouth_compress" => 0.7_f32,
    ));
    lib.add(p!(
        "fearful", "Fear / apprehension", ["fear", "negative"],
        "brow_raise_L"       => 0.7_f32,
        "brow_raise_R"       => 0.7_f32,
        "brow_inner_raise_L" => 0.8_f32,
        "brow_inner_raise_R" => 0.8_f32,
        "eye_wide_L"         => 0.6_f32,
        "eye_wide_R"         => 0.6_f32,
        "jaw_open"           => 0.25_f32,
    ));
    lib.add(p!(
        "disgusted", "Disgust", ["disgust", "negative"],
        "nose_wrinkle"    => 0.85_f32,
        "lip_upper_raise" => 0.6_f32,
        "mouth_frown_L"   => 0.35_f32,
        "mouth_frown_R"   => 0.35_f32,
    ));
    lib.add(p!(
        "surprised", "Surprise", ["surprise", "positive"],
        "brow_raise_L"    => 0.9_f32,
        "brow_raise_R"    => 0.9_f32,
        "eye_wide_L"      => 0.8_f32,
        "eye_wide_R"      => 0.8_f32,
        "jaw_open"        => 0.7_f32,
        "lip_upper_raise" => 0.4_f32,
        "lip_lower_drop"  => 0.4_f32,
    ));

    lib
}

/// Return the morph weight map for a named preset (with intensity applied).
#[allow(dead_code)]
pub fn preset_morph_weights(lib: &ExpressionLibrary, name: &str) -> Option<HashMap<String, f32>> {
    let p = lib.get(name)?;
    Some(ExpressionLibrary::apply_intensity(p))
}

/// Return a sorted list of preset names.
#[allow(dead_code)]
pub fn list_preset_names(lib: &ExpressionLibrary) -> Vec<String> {
    lib.list_names().iter().map(|s| s.to_string()).collect()
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Lerp two weight maps over the union of their keys.
fn lerp_weight_maps(
    a: &HashMap<String, f32>,
    b: &HashMap<String, f32>,
    t: f32,
) -> HashMap<String, f32> {
    let t = t.clamp(0.0, 1.0);
    let mut result: HashMap<String, f32> = HashMap::new();

    for (k, &va) in a {
        let vb = b.get(k).copied().unwrap_or(0.0);
        result.insert(k.clone(), va + (vb - va) * t);
    }
    for (k, &vb) in b {
        if !a.contains_key(k) {
            result.insert(k.clone(), vb * t);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a small weight map
    fn wmap(pairs: &[(&str, f32)]) -> HashMap<String, f32> {
        pairs.iter().map(|&(k, v)| (k.to_string(), v)).collect()
    }

    // -----------------------------------------------------------------------
    // 1. default_library has all expected presets
    // -----------------------------------------------------------------------
    #[test]
    fn default_library_has_eleven_presets() {
        let lib = ExpressionLibrary::default_library();
        assert!(
            lib.count() >= 11,
            "expected ≥11 presets, got {}",
            lib.count()
        );
    }

    // -----------------------------------------------------------------------
    // 2. list_names returns sorted names
    // -----------------------------------------------------------------------
    #[test]
    fn list_names_is_sorted() {
        let lib = ExpressionLibrary::default_library();
        let names = lib.list_names();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
    }

    // -----------------------------------------------------------------------
    // 3. get known preset returns Some
    // -----------------------------------------------------------------------
    #[test]
    fn get_known_preset_returns_some() {
        let lib = ExpressionLibrary::default_library();
        for name in &[
            "neutral",
            "smile",
            "frown",
            "blink",
            "wink_left",
            "wink_right",
        ] {
            assert!(lib.get(name).is_some(), "preset '{}' must exist", name);
        }
    }

    // -----------------------------------------------------------------------
    // 4. get unknown preset returns None
    // -----------------------------------------------------------------------
    #[test]
    fn get_unknown_preset_returns_none() {
        let lib = ExpressionLibrary::default_library();
        assert!(lib.get("__nonexistent__").is_none());
    }

    // -----------------------------------------------------------------------
    // 5. list_by_tag
    // -----------------------------------------------------------------------
    #[test]
    fn list_by_tag_returns_correct_presets() {
        let lib = ExpressionLibrary::default_library();
        let eye_presets = lib.list_by_tag("eye");
        // blink, wink_left, wink_right at minimum
        let names: Vec<&str> = eye_presets.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"blink"), "blink should be tagged 'eye'");
        assert!(
            names.contains(&"wink_left"),
            "wink_left should be tagged 'eye'"
        );
        assert!(
            names.contains(&"wink_right"),
            "wink_right should be tagged 'eye'"
        );
    }

    // -----------------------------------------------------------------------
    // 6. blend at t=0 gives weights of a
    // -----------------------------------------------------------------------
    #[test]
    fn blend_at_t0_equals_a() {
        let lib = ExpressionLibrary::default_library();
        let blended = lib
            .blend("smile", "frown", 0.0)
            .expect("blend must succeed");
        let smile = lib.get("smile").unwrap();
        for (k, &va) in &smile.weights {
            let bv = blended.get(k).copied().unwrap_or(0.0);
            assert!(
                (bv - va).abs() < 1e-5,
                "key '{}': expected {}, got {}",
                k,
                va,
                bv
            );
        }
    }

    // -----------------------------------------------------------------------
    // 7. blend at t=1 gives weights of b
    // -----------------------------------------------------------------------
    #[test]
    fn blend_at_t1_equals_b() {
        let lib = ExpressionLibrary::default_library();
        let blended = lib
            .blend("smile", "frown", 1.0)
            .expect("blend must succeed");
        let frown = lib.get("frown").unwrap();
        for (k, &vb) in &frown.weights {
            let bv = blended.get(k).copied().unwrap_or(0.0);
            assert!(
                (bv - vb).abs() < 1e-5,
                "key '{}': expected {}, got {}",
                k,
                vb,
                bv
            );
        }
    }

    // -----------------------------------------------------------------------
    // 8. blend with unknown name returns None
    // -----------------------------------------------------------------------
    #[test]
    fn blend_unknown_name_returns_none() {
        let lib = ExpressionLibrary::default_library();
        assert!(lib.blend("smile", "__no__", 0.5).is_none());
        assert!(lib.blend("__no__", "frown", 0.5).is_none());
    }

    // -----------------------------------------------------------------------
    // 9. apply_intensity scales weights
    // -----------------------------------------------------------------------
    #[test]
    fn apply_intensity_scales_correctly() {
        let mut preset = ExpressionPreset::new(
            "test",
            "desc",
            wmap(&[("eye_close_L", 0.8), ("eye_close_R", 0.6)]),
        );
        preset.intensity = 0.5;
        let scaled = ExpressionLibrary::apply_intensity(&preset);
        assert!((scaled["eye_close_L"] - 0.4).abs() < 1e-5);
        assert!((scaled["eye_close_R"] - 0.3).abs() < 1e-5);
    }

    // -----------------------------------------------------------------------
    // 10. combine is clamped to [0, 1]
    // -----------------------------------------------------------------------
    #[test]
    fn combine_clamps_to_unit() {
        let lib = ExpressionLibrary::default_library();
        let smile = lib.get("smile").unwrap();
        let blink = lib.get("blink").unwrap();
        let combined = ExpressionLibrary::combine(&[smile, smile, blink]);
        for &v in combined.values() {
            assert!(
                (0.0..=1.0).contains(&v),
                "combined weight {} is out of [0, 1]",
                v
            );
        }
    }

    // -----------------------------------------------------------------------
    // 11. expression_distance — same map gives 0
    // -----------------------------------------------------------------------
    #[test]
    fn expression_distance_same_is_zero() {
        let m = wmap(&[("a", 0.5), ("b", 0.3)]);
        let d = expression_distance(&m, &m);
        assert!(
            d.abs() < 1e-6,
            "distance of map to itself must be 0, got {}",
            d
        );
    }

    // -----------------------------------------------------------------------
    // 12. expression_distance triangle inequality
    // -----------------------------------------------------------------------
    #[test]
    fn expression_distance_triangle_inequality() {
        let a = wmap(&[("x", 0.0), ("y", 0.0)]);
        let b = wmap(&[("x", 1.0), ("y", 0.0)]);
        let c = wmap(&[("x", 0.5), ("y", 0.5)]);
        let dab = expression_distance(&a, &b);
        let dac = expression_distance(&a, &c);
        let dcb = expression_distance(&c, &b);
        assert!(
            dab <= dac + dcb + 1e-5,
            "triangle inequality failed: {} > {} + {}",
            dab,
            dac,
            dcb
        );
    }

    // -----------------------------------------------------------------------
    // 13. find_nearest — neutral map closest to neutral preset
    // -----------------------------------------------------------------------
    #[test]
    fn find_nearest_empty_returns_neutral() {
        let lib = ExpressionLibrary::default_library();
        // Empty weight map should be closest to "neutral" (which has no weights)
        let nearest = lib.find_nearest(&HashMap::new()).expect("must return Some");
        assert_eq!(nearest, "neutral");
    }

    // -----------------------------------------------------------------------
    // 14. ExpressionBlender evaluate and normalize
    // -----------------------------------------------------------------------
    #[test]
    fn expression_blender_normalize_sums_to_one() {
        let mut blender = ExpressionBlender::new();
        blender.add_anchor("smile".to_string(), 2.0);
        blender.add_anchor("frown".to_string(), 2.0);
        blender.normalize();
        let total: f32 = blender.anchors.iter().map(|(_, w)| w).sum();
        assert!(
            (total - 1.0).abs() < 1e-5,
            "total after normalize = {}",
            total
        );
    }

    // -----------------------------------------------------------------------
    // 15. ExpressionBlender evaluate produces values in [0, 1]
    // -----------------------------------------------------------------------
    #[test]
    fn expression_blender_evaluate_in_unit() {
        let lib = ExpressionLibrary::default_library();
        let mut blender = ExpressionBlender::new();
        blender.add_anchor("smile".to_string(), 0.5);
        blender.add_anchor("surprise".to_string(), 0.5);
        blender.add_anchor("anger".to_string(), 0.5);
        let result = blender.evaluate(&lib);
        for &v in result.values() {
            assert!((0.0..=1.0).contains(&v), "value {} out of [0,1]", v);
        }
    }

    // -----------------------------------------------------------------------
    // 16. random_blend produces values in [0, 1] for various seeds
    // -----------------------------------------------------------------------
    #[test]
    fn random_blend_in_unit_range() {
        let lib = ExpressionLibrary::default_library();
        let names = lib.list_names();
        let presets: Vec<&ExpressionPreset> = names.iter().filter_map(|n| lib.get(n)).collect();
        for seed in [0u32, 1, 42, 999, u32::MAX] {
            let result = ExpressionLibrary::random_blend(&presets, seed);
            for &v in result.values() {
                assert!(
                    (0.0..=1.0).contains(&v),
                    "seed={seed}: value {v} out of [0,1]"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // 17. add and count
    // -----------------------------------------------------------------------
    #[test]
    fn add_increases_count() {
        let mut lib = ExpressionLibrary::new();
        assert_eq!(lib.count(), 0);
        lib.add(ExpressionPreset::new("a", "desc", HashMap::new()));
        assert_eq!(lib.count(), 1);
        lib.add(ExpressionPreset::new("b", "desc", HashMap::new()));
        assert_eq!(lib.count(), 2);
        // Overwrite same name
        lib.add(ExpressionPreset::new("a", "overwrite", HashMap::new()));
        assert_eq!(lib.count(), 2);
    }

    // -----------------------------------------------------------------------
    // 18. write test artefact to /tmp/
    // -----------------------------------------------------------------------
    #[test]
    fn write_expression_names_to_tmp() {
        let lib = ExpressionLibrary::default_library();
        let names = lib.list_names().join("\n");
        std::fs::write("/tmp/oxihuman_expression_library_names.txt", &names)
            .expect("write to /tmp/ must succeed");
        let read_back =
            std::fs::read_to_string("/tmp/oxihuman_expression_library_names.txt").unwrap();
        assert_eq!(read_back, names);
    }

    // -----------------------------------------------------------------------
    // 19. default_library_config has expected defaults
    // -----------------------------------------------------------------------
    #[test]
    fn default_library_config_defaults() {
        let cfg = default_library_config();
        assert_eq!(cfg.max_presets, 0);
        assert!(cfg.allow_overwrite);
        assert!((cfg.default_intensity - 1.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // 20. new_expression_library is empty
    // -----------------------------------------------------------------------
    #[test]
    fn new_expression_library_is_empty() {
        let lib = new_expression_library();
        assert_eq!(preset_count(&lib), 0);
    }

    // -----------------------------------------------------------------------
    // 21. add_preset / remove_preset round-trip
    // -----------------------------------------------------------------------
    #[test]
    fn add_remove_preset_round_trip() {
        let mut lib = new_expression_library();
        add_preset(&mut lib, ExpressionPreset::new("test", "d", HashMap::new()));
        assert_eq!(preset_count(&lib), 1);
        let removed = remove_preset(&mut lib, "test");
        assert!(removed);
        assert_eq!(preset_count(&lib), 0);
        assert!(!remove_preset(&mut lib, "test"));
    }

    // -----------------------------------------------------------------------
    // 22. get_preset returns correct entry
    // -----------------------------------------------------------------------
    #[test]
    fn get_preset_correct() {
        let lib = ExpressionLibrary::default_library();
        let p = get_preset(&lib, "smile");
        assert!(p.is_some());
        assert_eq!(p.unwrap().name, "smile");
    }

    // -----------------------------------------------------------------------
    // 23. find_preset_by_name substring match
    // -----------------------------------------------------------------------
    #[test]
    fn find_preset_by_name_substring() {
        let lib = ExpressionLibrary::default_library();
        let p = find_preset_by_name(&lib, "wink");
        assert!(p.is_some());
        assert!(p.unwrap().name.contains("wink"));
    }

    // -----------------------------------------------------------------------
    // 24. blend_presets at midpoint is between a and b
    // -----------------------------------------------------------------------
    #[test]
    fn blend_presets_at_midpoint() {
        let lib = ExpressionLibrary::default_library();
        let result = blend_presets(&lib, "smile", "frown", 0.5).expect("blend must succeed");
        assert!(!result.is_empty());
    }

    // -----------------------------------------------------------------------
    // 25. library_to_json contains preset names
    // -----------------------------------------------------------------------
    #[test]
    fn library_to_json_contains_names() {
        let lib = ExpressionLibrary::default_library();
        let json = library_to_json(&lib);
        assert!(json.contains("smile"));
        assert!(json.contains("neutral"));
    }

    // -----------------------------------------------------------------------
    // 26. build_basic_library has exactly 7 presets
    // -----------------------------------------------------------------------
    #[test]
    fn build_basic_library_has_seven_presets() {
        let lib = build_basic_library();
        assert_eq!(lib.count(), 7);
    }

    // -----------------------------------------------------------------------
    // 27. build_basic_library has all 7 emotion names
    // -----------------------------------------------------------------------
    #[test]
    fn build_basic_library_emotion_names() {
        let lib = build_basic_library();
        for name in &[
            "neutral",
            "happy",
            "sad",
            "angry",
            "fearful",
            "disgusted",
            "surprised",
        ] {
            assert!(lib.get(name).is_some(), "missing preset: {name}");
        }
    }

    // -----------------------------------------------------------------------
    // 28. preset_morph_weights applies intensity
    // -----------------------------------------------------------------------
    #[test]
    fn preset_morph_weights_applies_intensity() {
        let mut lib = new_expression_library();
        let mut p = ExpressionPreset::new("x", "d", wmap(&[("k", 0.8)]));
        p.intensity = 0.5;
        add_preset(&mut lib, p);
        let weights = preset_morph_weights(&lib, "x").unwrap();
        assert!((weights["k"] - 0.4).abs() < 1e-5);
    }

    // -----------------------------------------------------------------------
    // 29. list_preset_names returns sorted strings
    // -----------------------------------------------------------------------
    #[test]
    fn list_preset_names_sorted() {
        let lib = ExpressionLibrary::default_library();
        let names = list_preset_names(&lib);
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }
}
