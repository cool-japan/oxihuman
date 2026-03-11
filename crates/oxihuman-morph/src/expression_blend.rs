// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Facial expression blending system.
//!
//! Provides [`ExpressionDef`], a descriptor for a named compound expression that
//! is composed of weighted morph-target contributions, and [`ExpressionBlender`],
//! a runtime library that resolves expression names to concrete morph-target
//! weight maps, supports additive multi-expression blending, smooth lerp
//! between two expressions, and FACS Action Unit → weight mapping.
//!
//! # Quick start
//!
//! ```rust
//! use oxihuman_morph::expression_blend::ExpressionBlender;
//!
//! let blender = ExpressionBlender::with_defaults();
//! let weights = blender.blend_to_weights("Happy", 0.8).unwrap_or_default();
//! assert!(weights.contains_key("mouth-corner-puller"));
//! ```

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ExpressionDef
// ---------------------------------------------------------------------------

/// Definition of a named compound expression made up of weighted morph-target
/// contributions.
#[derive(Debug, Clone)]
pub struct ExpressionDef {
    /// Unique name, e.g. `"Happy"`.
    pub name: String,
    /// Ordered list of `(morph_target_name, weight)` pairs that together
    /// compose this expression.  Weights are in `[0.0, 1.0]`.
    pub targets: Vec<(String, f64)>,
    /// Semantic tags (e.g. `["positive", "mouth", "eyes"]`).
    pub tags: Vec<String>,
    /// Optional name of the bilateral-symmetric counterpart expression,
    /// e.g. `"Contempt"` for `"Contempt_R"`.
    pub symmetry_pair: Option<String>,
}

impl ExpressionDef {
    /// Construct a new expression definition.
    pub fn new(name: impl Into<String>, targets: Vec<(impl Into<String>, f64)>) -> Self {
        Self {
            name: name.into(),
            targets: targets.into_iter().map(|(t, w)| (t.into(), w)).collect(),
            tags: Vec::new(),
            symmetry_pair: None,
        }
    }

    /// Builder: set tags.
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }

    /// Builder: set the symmetry pair.
    pub fn with_symmetry_pair(mut self, pair: impl Into<String>) -> Self {
        self.symmetry_pair = Some(pair.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Pre-defined expression library helpers
// ---------------------------------------------------------------------------

fn make_def(
    name: &str,
    targets: &[(&str, f64)],
    tags: &[&str],
    sym: Option<&str>,
) -> ExpressionDef {
    let mut def = ExpressionDef::new(
        name,
        targets
            .iter()
            .map(|(t, w)| (t.to_string(), *w))
            .collect::<Vec<_>>(),
    )
    .with_tags(tags.iter().copied());
    if let Some(s) = sym {
        def = def.with_symmetry_pair(s);
    }
    def
}

/// Return the eight basic Ekman emotions plus several derived expressions as
/// pre-built [`ExpressionDef`]s, ready to be inserted into an
/// [`ExpressionBlender`].
pub fn default_expression_defs() -> Vec<ExpressionDef> {
    vec![
        // ----------------------------------------------------------------
        // 1. Neutral — baseline
        // ----------------------------------------------------------------
        make_def("Neutral", &[], &["neutral", "baseline"], None),
        // ----------------------------------------------------------------
        // 2. Happy — lip corners up, cheek raise, inner brow oblique
        // ----------------------------------------------------------------
        make_def(
            "Happy",
            &[
                ("mouth-corner-puller", 0.85),
                ("mouth-elevation", 0.55),
                ("cheek-raise-left", 0.60),
                ("cheek-raise-right", 0.60),
                ("eye-left-slight-close", 0.25),
                ("eye-right-slight-close", 0.25),
            ],
            &["positive", "smile", "basic"],
            None,
        ),
        // ----------------------------------------------------------------
        // 3. Sad — inner brow raise, mouth depression, eyes heavy
        // ----------------------------------------------------------------
        make_def(
            "Sad",
            &[
                ("eyebrows-left-inner-up", 0.75),
                ("eyebrows-right-inner-up", 0.75),
                ("mouth-depression", 0.65),
                ("eye-left-opened-down", 0.35),
                ("eye-right-opened-down", 0.35),
                ("lower-lip-depression", 0.40),
            ],
            &["negative", "basic", "brow"],
            Some("Happy"),
        ),
        // ----------------------------------------------------------------
        // 4. Angry — brow down-and-in, lip compression, nose wrinkle
        // ----------------------------------------------------------------
        make_def(
            "Angry",
            &[
                ("eyebrows-left-down", 0.80),
                ("eyebrows-right-down", 0.80),
                ("eyebrows-left-inner-down", 0.70),
                ("eyebrows-right-inner-down", 0.70),
                ("mouth-compression", 0.60),
                ("mouth-retraction", 0.30),
                ("nose-wrinkle", 0.40),
            ],
            &["negative", "basic", "brow", "mouth"],
            None,
        ),
        // ----------------------------------------------------------------
        // 5. Surprised — mouth open, brows up, eyes wide
        // ----------------------------------------------------------------
        make_def(
            "Surprised",
            &[
                ("mouth-open", 0.75),
                ("eyebrows-left-up", 0.85),
                ("eyebrows-right-up", 0.85),
                ("eye-left-opened-up", 0.65),
                ("eye-right-opened-up", 0.65),
                ("jaw-drop", 0.50),
            ],
            &["surprise", "basic", "eyes", "mouth"],
            None,
        ),
        // ----------------------------------------------------------------
        // 6. Disgusted — nose wrinkle, upper lip raise, brow down
        // ----------------------------------------------------------------
        make_def(
            "Disgusted",
            &[
                ("nose-wrinkle", 0.75),
                ("upper-lip-raise", 0.65),
                ("eyebrows-left-down", 0.45),
                ("eyebrows-right-down", 0.45),
                ("mouth-left-corner-depression", 0.30),
                ("mouth-right-corner-depression", 0.30),
            ],
            &["negative", "basic", "nose", "mouth"],
            None,
        ),
        // ----------------------------------------------------------------
        // 7. Fearful — brows up-and-together, eyes wide, mouth open
        // ----------------------------------------------------------------
        make_def(
            "Fearful",
            &[
                ("eyebrows-left-up", 0.70),
                ("eyebrows-right-up", 0.70),
                ("eyebrows-left-inner-up", 0.65),
                ("eyebrows-right-inner-up", 0.65),
                ("eye-left-opened-up", 0.80),
                ("eye-right-opened-up", 0.80),
                ("mouth-open", 0.45),
                ("mouth-stretch", 0.35),
            ],
            &["negative", "basic", "eyes", "brow"],
            None,
        ),
        // ----------------------------------------------------------------
        // 8. Contempt — unilateral lip corner raise (right side default)
        // ----------------------------------------------------------------
        make_def(
            "Contempt",
            &[
                ("mouth-right-corner-puller", 0.70),
                ("eyebrows-right-up", 0.30),
                ("nose-wrinkle", 0.20),
            ],
            &["negative", "basic", "asymmetric"],
            Some("Contempt_L"),
        ),
        // ----------------------------------------------------------------
        // Derived / compound expressions
        // ----------------------------------------------------------------

        // Smirk (contempt-adjacent, asymmetric smile)
        make_def(
            "Smirk",
            &[
                ("mouth-right-corner-puller", 0.60),
                ("eyebrows-right-up", 0.20),
            ],
            &["positive", "asymmetric"],
            None,
        ),
        // Wink (surprise + blink blend)
        make_def(
            "Wink",
            &[("eye-right-blink", 0.90), ("mouth-corner-puller", 0.30)],
            &["playful", "asymmetric"],
            None,
        ),
        // Pain
        make_def(
            "Pain",
            &[
                ("eyebrows-left-inner-up", 0.80),
                ("eyebrows-right-inner-up", 0.80),
                ("eyebrows-left-down", 0.50),
                ("eyebrows-right-down", 0.50),
                ("nose-wrinkle", 0.55),
                ("mouth-compression", 0.50),
                ("eye-left-slight-close", 0.70),
                ("eye-right-slight-close", 0.70),
            ],
            &["negative", "brow", "eyes"],
            None,
        ),
        // Boredom
        make_def(
            "Boredom",
            &[
                ("eye-left-slight-close", 0.50),
                ("eye-right-slight-close", 0.50),
                ("mouth-depression", 0.20),
                ("eyebrows-left-down", 0.20),
                ("eyebrows-right-down", 0.20),
            ],
            &["neutral", "eyes"],
            None,
        ),
    ]
}

// ---------------------------------------------------------------------------
// ExpressionBlender
// ---------------------------------------------------------------------------

/// Manages a library of [`ExpressionDef`]s and resolves them to morph-target
/// weight maps at runtime.
#[derive(Debug, Clone)]
pub struct ExpressionBlender {
    /// Map from expression name (lower-case-canonical) → definition.
    library: HashMap<String, ExpressionDef>,
}

impl ExpressionBlender {
    /// Create an empty blender with no pre-loaded expressions.
    pub fn new() -> Self {
        Self {
            library: HashMap::new(),
        }
    }

    /// Create a blender pre-loaded with the default expression library
    /// ([`default_expression_defs`]).
    pub fn with_defaults() -> Self {
        let mut blender = Self::new();
        for def in default_expression_defs() {
            blender.add(def);
        }
        blender
    }

    /// Add an expression definition to the library.
    /// If an expression with the same name already exists it is replaced.
    pub fn add(&mut self, def: ExpressionDef) {
        let key = Self::canonical(&def.name);
        self.library.insert(key, def);
    }

    /// Look up an expression definition by name (case-insensitive).
    pub fn get(&self, name: &str) -> Option<&ExpressionDef> {
        self.library.get(&Self::canonical(name))
    }

    /// Return a sorted list of all expression names (original casing).
    pub fn list_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.library.values().map(|d| d.name.as_str()).collect();
        names.sort_unstable();
        names
    }

    /// Number of expressions in the library.
    pub fn len(&self) -> usize {
        self.library.len()
    }

    /// Return `true` if the library is empty.
    pub fn is_empty(&self) -> bool {
        self.library.is_empty()
    }

    /// Map a single expression at `intensity` to a `HashMap<target, weight>`.
    ///
    /// Each per-target base weight is multiplied by `intensity`, then clamped
    /// to `[0.0, 1.0]`.  Returns `None` if the expression name is not found.
    pub fn blend_to_weights(&self, expr: &str, intensity: f64) -> Option<HashMap<String, f64>> {
        let def = self.get(expr)?;
        let scale = intensity.clamp(0.0, 1.0);
        let mut out = HashMap::with_capacity(def.targets.len());
        for (target, base_w) in &def.targets {
            let w = (base_w * scale).clamp(0.0, 1.0);
            out.insert(target.clone(), w);
        }
        Some(out)
    }

    /// Additively blend multiple expressions.
    ///
    /// `exprs` is a slice of `(expression_name, intensity)` pairs.
    /// For each pair the expression is resolved at the given intensity and
    /// its weights are **added** into the accumulator.  The final map is
    /// clamped per-target to `[0.0, 1.0]`.  Unknown expression names are
    /// silently skipped.
    pub fn blend_multi(&self, exprs: &[(String, f64)]) -> HashMap<String, f64> {
        let mut acc: HashMap<String, f64> = HashMap::new();
        for (name, intensity) in exprs {
            if let Some(weights) = self.blend_to_weights(name, *intensity) {
                for (target, w) in weights {
                    let entry = acc.entry(target).or_insert(0.0);
                    *entry = (*entry + w).clamp(0.0, 1.0);
                }
            }
        }
        acc
    }

    /// Linearly interpolate between two expressions at parameter `t ∈ [0, 1]`.
    ///
    /// At `t = 0.0` the result equals `blend_to_weights(from, 1.0)`.
    /// At `t = 1.0` the result equals `blend_to_weights(to, 1.0)`.
    /// Intermediate values are computed per-target as `w_from * (1-t) + w_to * t`.
    ///
    /// Targets that appear in only one expression are treated as having weight
    /// 0.0 in the other.  Returns an empty map if both expressions are unknown.
    pub fn lerp_expression(&self, from: &str, to: &str, t: f64) -> HashMap<String, f64> {
        let t = t.clamp(0.0, 1.0);
        let from_map = self.blend_to_weights(from, 1.0).unwrap_or_default();
        let to_map = self.blend_to_weights(to, 1.0).unwrap_or_default();

        // Collect union of all target keys
        let mut keys: Vec<String> = from_map.keys().cloned().collect();
        for k in to_map.keys() {
            if !from_map.contains_key(k) {
                keys.push(k.clone());
            }
        }

        let mut out = HashMap::with_capacity(keys.len());
        for key in keys {
            let a = from_map.get(&key).copied().unwrap_or(0.0);
            let b = to_map.get(&key).copied().unwrap_or(0.0);
            let v = (a * (1.0 - t) + b * t).clamp(0.0, 1.0);
            out.insert(key, v);
        }
        out
    }

    /// Map a FACS Action Unit code + intensity to morph-target weights.
    ///
    /// This implements an approximate mapping from the standard FACS AU numbers
    /// (as used by Paul Ekman et al.) to OxiHuman morph-target names.
    ///
    /// The mapping is intentionally broad: each AU drives one or more targets.
    /// `intensity` is in `[0.0, 1.0]`; the return value is a weight map with
    /// all values clamped to `[0.0, 1.0]`.
    ///
    /// Unknown AU codes return an empty map rather than erroring.
    pub fn au_to_expression(au_code: u32, intensity: f64) -> HashMap<String, f64> {
        let scale = intensity.clamp(0.0, 1.0);
        let targets: &[(&str, f64)] = match au_code {
            // AU1  — Inner Brow Raise
            1 => &[
                ("eyebrows-left-inner-up", 1.0),
                ("eyebrows-right-inner-up", 1.0),
            ],
            // AU2  — Outer Brow Raise
            2 => &[("eyebrows-left-up", 1.0), ("eyebrows-right-up", 1.0)],
            // AU4  — Brow Lowerer
            4 => &[
                ("eyebrows-left-down", 1.0),
                ("eyebrows-right-down", 1.0),
                ("eyebrows-left-inner-down", 0.70),
                ("eyebrows-right-inner-down", 0.70),
            ],
            // AU5  — Upper Lid Raiser
            5 => &[("eye-left-opened-up", 1.0), ("eye-right-opened-up", 1.0)],
            // AU6  — Cheek Raiser
            6 => &[("cheek-raise-left", 1.0), ("cheek-raise-right", 1.0)],
            // AU7  — Lid Tightener
            7 => &[
                ("eye-left-slight-close", 0.70),
                ("eye-right-slight-close", 0.70),
            ],
            // AU9  — Nose Wrinkler
            9 => &[("nose-wrinkle", 1.0)],
            // AU10 — Upper Lip Raiser
            10 => &[("upper-lip-raise", 1.0)],
            // AU11 — Nasolabial Deepener
            11 => &[
                ("nasolabial-deepener-left", 0.80),
                ("nasolabial-deepener-right", 0.80),
            ],
            // AU12 — Lip Corner Puller (Smile muscle)
            12 => &[("mouth-corner-puller", 1.0), ("mouth-elevation", 0.40)],
            // AU13 — Cheek Puffer
            13 => &[("cheek-puff-left", 0.80), ("cheek-puff-right", 0.80)],
            // AU14 — Dimpler
            14 => &[("cheek-dimple-left", 0.80), ("cheek-dimple-right", 0.80)],
            // AU15 — Lip Corner Depressor
            15 => &[
                ("mouth-left-corner-depression", 0.90),
                ("mouth-right-corner-depression", 0.90),
                ("mouth-depression", 0.50),
            ],
            // AU16 — Lower Lip Depressor
            16 => &[("lower-lip-depression", 1.0)],
            // AU17 — Chin Raiser
            17 => &[("chin-raise", 1.0)],
            // AU18 — Lip Puckerer
            18 => &[("lip-puckerer", 1.0)],
            // AU20 — Lip Stretcher
            20 => &[("mouth-stretch", 1.0)],
            // AU22 — Lip Funneler
            22 => &[("lip-funnel", 1.0)],
            // AU23 — Lip Tightener
            23 => &[("mouth-compression", 0.80)],
            // AU24 — Lip Pressor
            24 => &[("mouth-compression", 0.60), ("lip-press", 0.80)],
            // AU25 — Lips Part
            25 => &[("mouth-open", 0.50)],
            // AU26 — Jaw Drop
            26 => &[("jaw-drop", 1.0), ("mouth-open", 0.70)],
            // AU27 — Mouth Stretch
            27 => &[
                ("mouth-open", 1.0),
                ("jaw-drop", 0.80),
                ("mouth-stretch", 0.60),
            ],
            // AU28 — Lip Suck
            28 => &[("lip-suck", 1.0)],
            // AU41 — Lid Droop
            41 => &[
                ("eye-left-opened-down", 0.80),
                ("eye-right-opened-down", 0.80),
            ],
            // AU42 — Slit
            42 => &[
                ("eye-left-slight-close", 0.60),
                ("eye-right-slight-close", 0.60),
            ],
            // AU43 — Eyes Closed
            43 => &[("eye-left-blink", 1.0), ("eye-right-blink", 1.0)],
            // AU44 — Squint
            44 => &[
                ("eye-left-slight-close", 0.90),
                ("eye-right-slight-close", 0.90),
                ("cheek-raise-left", 0.50),
                ("cheek-raise-right", 0.50),
            ],
            // AU45 — Blink
            45 => &[("eye-left-blink", 1.0), ("eye-right-blink", 1.0)],
            // AU46 — Wink (right eye by convention)
            46 => &[("eye-right-blink", 1.0)],
            // Unknown AU
            _ => &[],
        };

        let mut out = HashMap::with_capacity(targets.len());
        for (t, base_w) in targets {
            let w = (base_w * scale).clamp(0.0, 1.0);
            out.insert(t.to_string(), w);
        }
        out
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn canonical(name: &str) -> String {
        name.to_lowercase()
    }
}

impl Default for ExpressionBlender {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn blender() -> ExpressionBlender {
        ExpressionBlender::with_defaults()
    }

    // ── ExpressionDef construction ──────────────────────────────────────────

    #[test]
    fn expression_def_basic_fields() {
        let def = ExpressionDef::new("TestExpr", vec![("target-a", 0.5), ("target-b", 1.0)]);
        assert_eq!(def.name, "TestExpr");
        assert_eq!(def.targets.len(), 2);
        assert!(def.tags.is_empty());
        assert!(def.symmetry_pair.is_none());
    }

    #[test]
    fn expression_def_with_tags_and_pair() {
        let def = ExpressionDef::new("Expr", vec![("t", 1.0)])
            .with_tags(["emotion", "face"])
            .with_symmetry_pair("Expr_Mirror");
        assert_eq!(def.tags, vec!["emotion", "face"]);
        assert_eq!(def.symmetry_pair.as_deref(), Some("Expr_Mirror"));
    }

    // ── Default library presence ────────────────────────────────────────────

    #[test]
    fn default_library_has_eight_basic_expressions() {
        let b = blender();
        for name in [
            "Neutral",
            "Happy",
            "Sad",
            "Angry",
            "Surprised",
            "Disgusted",
            "Fearful",
            "Contempt",
        ] {
            assert!(b.get(name).is_some(), "missing basic expression: {name}");
        }
    }

    #[test]
    fn default_library_len_at_least_eight() {
        let b = blender();
        assert!(b.len() >= 8, "expected >= 8 expressions, got {}", b.len());
    }

    #[test]
    fn list_names_is_sorted() {
        let b = blender();
        let names = b.list_names();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
    }

    // ── blend_to_weights ───────────────────────────────────────────────────

    #[test]
    fn blend_to_weights_happy_full() {
        let b = blender();
        let w = b.blend_to_weights("Happy", 1.0).expect("Happy not found");
        assert!(w.contains_key("mouth-corner-puller"));
        for v in w.values() {
            assert!(*v >= 0.0 && *v <= 1.0, "weight out of range: {v}");
        }
    }

    #[test]
    fn blend_to_weights_scales_with_intensity() {
        let b = blender();
        let w_full = b.blend_to_weights("Happy", 1.0).unwrap();
        let w_half = b.blend_to_weights("Happy", 0.5).unwrap();
        for (k, v_full) in &w_full {
            let v_half = w_half[k];
            assert!(
                (v_half - v_full * 0.5).abs() < 1e-10,
                "scale mismatch for {k}: full={v_full} half={v_half}"
            );
        }
    }

    #[test]
    fn blend_to_weights_zero_intensity_returns_all_zeros() {
        let b = blender();
        let w = b.blend_to_weights("Happy", 0.0).unwrap();
        for v in w.values() {
            assert_eq!(*v, 0.0);
        }
    }

    #[test]
    fn blend_to_weights_neutral_returns_empty_map() {
        let b = blender();
        let w = b.blend_to_weights("Neutral", 1.0).unwrap();
        assert!(w.is_empty(), "Neutral should have no targets");
    }

    #[test]
    fn blend_to_weights_unknown_expression_returns_none() {
        let b = blender();
        assert!(b.blend_to_weights("NonExistentXYZ", 1.0).is_none());
    }

    #[test]
    fn blend_to_weights_clamps_intensity_over_one() {
        let b = blender();
        let w_one = b.blend_to_weights("Happy", 1.0).unwrap();
        let w_over = b.blend_to_weights("Happy", 2.5).unwrap();
        assert_eq!(w_one, w_over, "intensity > 1.0 should be clamped to 1.0");
    }

    #[test]
    fn blend_to_weights_clamps_intensity_negative() {
        let b = blender();
        let w = b.blend_to_weights("Happy", -0.5).unwrap();
        for v in w.values() {
            assert_eq!(*v, 0.0);
        }
    }

    // ── blend_multi ────────────────────────────────────────────────────────

    #[test]
    fn blend_multi_two_expressions() {
        let b = blender();
        let exprs = vec![("Happy".to_string(), 0.6), ("Sad".to_string(), 0.4)];
        let w = b.blend_multi(&exprs);
        assert!(!w.is_empty());
        for v in w.values() {
            assert!(*v >= 0.0 && *v <= 1.0);
        }
    }

    #[test]
    fn blend_multi_skips_unknown_names() {
        let b = blender();
        let exprs = vec![("Happy".to_string(), 1.0), ("UnknownXYZ".to_string(), 1.0)];
        let w = b.blend_multi(&exprs);
        // Should match Happy alone
        let w_happy = b.blend_to_weights("Happy", 1.0).unwrap();
        for k in w_happy.keys() {
            assert!(w.contains_key(k));
        }
    }

    #[test]
    fn blend_multi_clamped_at_one() {
        let b = blender();
        // Blend Happy at full intensity twice — additive should not exceed 1.0
        let exprs = vec![("Happy".to_string(), 1.0), ("Happy".to_string(), 1.0)];
        let w = b.blend_multi(&exprs);
        for v in w.values() {
            assert!(*v <= 1.0, "clamp failed: {v}");
        }
    }

    #[test]
    fn blend_multi_empty_input() {
        let b = blender();
        let w = b.blend_multi(&[]);
        assert!(w.is_empty());
    }

    // ── lerp_expression ────────────────────────────────────────────────────

    #[test]
    fn lerp_at_t0_equals_from() {
        let b = blender();
        let lerped = b.lerp_expression("Happy", "Sad", 0.0);
        let from = b.blend_to_weights("Happy", 1.0).unwrap();
        for (k, v) in &from {
            let lv = lerped.get(k).copied().unwrap_or(0.0);
            assert!((lv - v).abs() < 1e-10, "t=0 mismatch for {k}: {lv} vs {v}");
        }
    }

    #[test]
    fn lerp_at_t1_equals_to() {
        let b = blender();
        let lerped = b.lerp_expression("Happy", "Sad", 1.0);
        let to_map = b.blend_to_weights("Sad", 1.0).unwrap();
        for (k, v) in &to_map {
            let lv = lerped.get(k).copied().unwrap_or(0.0);
            assert!((lv - v).abs() < 1e-10, "t=1 mismatch for {k}: {lv} vs {v}");
        }
    }

    #[test]
    fn lerp_at_t_half_midpoint() {
        let b = blender();
        let lerped = b.lerp_expression("Happy", "Angry", 0.5);
        let happy = b.blend_to_weights("Happy", 1.0).unwrap();
        let angry = b.blend_to_weights("Angry", 1.0).unwrap();
        for k in happy.keys().chain(angry.keys()) {
            let a = happy.get(k).copied().unwrap_or(0.0);
            let c = angry.get(k).copied().unwrap_or(0.0);
            let expected = (a * 0.5 + c * 0.5).clamp(0.0, 1.0);
            let got = lerped.get(k).copied().unwrap_or(0.0);
            assert!(
                (got - expected).abs() < 1e-10,
                "midpoint mismatch for {k}: expected {expected} got {got}"
            );
        }
    }

    #[test]
    fn lerp_unknown_from_returns_to_at_t1() {
        let b = blender();
        let lerped = b.lerp_expression("NonExistent", "Happy", 1.0);
        let to_map = b.blend_to_weights("Happy", 1.0).unwrap();
        for (k, v) in &to_map {
            let lv = lerped.get(k).copied().unwrap_or(0.0);
            assert!((lv - v).abs() < 1e-10, "t=1 mismatch for {k}");
        }
    }

    #[test]
    fn lerp_values_always_in_unit_interval() {
        let b = blender();
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let map = b.lerp_expression("Fearful", "Disgusted", t);
            for v in map.values() {
                assert!(*v >= 0.0 && *v <= 1.0, "out of range at t={t}: {v}");
            }
        }
    }

    // ── au_to_expression ───────────────────────────────────────────────────

    #[test]
    fn au_12_produces_lip_corner_puller() {
        let w = ExpressionBlender::au_to_expression(12, 1.0);
        assert!(
            w.contains_key("mouth-corner-puller"),
            "AU12 should drive mouth-corner-puller"
        );
    }

    #[test]
    fn au_4_produces_brow_lowerer() {
        let w = ExpressionBlender::au_to_expression(4, 1.0);
        assert!(
            w.contains_key("eyebrows-left-down"),
            "AU4 should lower brows"
        );
        assert!(w.contains_key("eyebrows-right-down"));
    }

    #[test]
    fn au_intensity_scales_correctly() {
        let w_full = ExpressionBlender::au_to_expression(12, 1.0);
        let w_half = ExpressionBlender::au_to_expression(12, 0.5);
        for (k, v_full) in &w_full {
            let v_half = w_half[k];
            assert!(
                (v_half - v_full * 0.5).abs() < 1e-10,
                "AU12 scale mismatch for {k}"
            );
        }
    }

    #[test]
    fn au_unknown_code_returns_empty() {
        let w = ExpressionBlender::au_to_expression(999, 1.0);
        assert!(w.is_empty(), "unknown AU should return empty map");
    }

    #[test]
    fn au_zero_intensity_returns_all_zeros() {
        let w = ExpressionBlender::au_to_expression(6, 0.0);
        for v in w.values() {
            assert_eq!(*v, 0.0);
        }
    }

    #[test]
    fn au_values_always_clamped() {
        for au in [1, 2, 4, 5, 6, 7, 9, 12, 15, 25, 26, 43, 45, 46] {
            let w = ExpressionBlender::au_to_expression(au, 1.5);
            for v in w.values() {
                assert!(*v <= 1.0, "AU{au} weight > 1.0: {v}");
            }
        }
    }

    #[test]
    fn au_45_blink_both_eyes() {
        let w = ExpressionBlender::au_to_expression(45, 1.0);
        assert!(w.contains_key("eye-left-blink"));
        assert!(w.contains_key("eye-right-blink"));
    }

    #[test]
    fn au_46_wink_right_eye_only() {
        let w = ExpressionBlender::au_to_expression(46, 1.0);
        assert!(w.contains_key("eye-right-blink"));
        assert!(
            !w.contains_key("eye-left-blink"),
            "AU46 wink should be right eye only"
        );
    }

    // ── case-insensitive lookup ─────────────────────────────────────────────

    #[test]
    fn get_is_case_insensitive() {
        let b = blender();
        assert!(b.get("happy").is_some());
        assert!(b.get("HAPPY").is_some());
        assert!(b.get("HaPpY").is_some());
    }

    // ── tags ───────────────────────────────────────────────────────────────

    #[test]
    fn happy_has_basic_tag() {
        let b = blender();
        let def = b.get("Happy").unwrap();
        assert!(
            def.tags.iter().any(|t| t == "basic"),
            "Happy should have 'basic' tag"
        );
    }

    #[test]
    fn contempt_has_symmetry_pair() {
        let b = blender();
        let def = b.get("Contempt").unwrap();
        assert!(
            def.symmetry_pair.is_some(),
            "Contempt should have a symmetry_pair"
        );
    }
}
