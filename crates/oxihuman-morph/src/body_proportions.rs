// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use crate::params::ParamState;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

/// Canonical proportion schema (ratios expressed as multiples of head height).
pub struct ProportionSchema {
    pub name: String,
    /// Total height in heads.
    pub heads_tall: f32,
    /// Shoulder width / head width.
    pub shoulder_ratio: f32,
    /// Hip width / head width.
    pub hip_ratio: f32,
    /// Leg length / total height.
    pub leg_ratio: f32,
    /// Arm length / total height.
    pub arm_ratio: f32,
    pub description: String,
}

/// Analysis of a character against a reference schema.
pub struct ProportionAnalysis {
    pub schema_name: String,
    /// key → how far from ideal (signed).
    pub deviations: HashMap<String, f32>,
    pub rms_deviation: f32,
    pub closest_schema: String,
}

/// A set of named proportion schemas.
pub struct ProportionLibrary {
    schemas: Vec<ProportionSchema>,
}

// ---------------------------------------------------------------------------
// ProportionLibrary impl
// ---------------------------------------------------------------------------

impl Default for ProportionLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl ProportionLibrary {
    /// Create an empty library.
    pub fn new() -> Self {
        Self {
            schemas: Vec::new(),
        }
    }

    /// Add a schema to the library.
    pub fn add(&mut self, schema: ProportionSchema) {
        self.schemas.push(schema);
    }

    /// Find a schema by name (case-sensitive).
    pub fn find(&self, name: &str) -> Option<&ProportionSchema> {
        self.schemas.iter().find(|s| s.name == name)
    }

    /// Return all schemas in the library.
    pub fn schemas(&self) -> &[ProportionSchema] {
        &self.schemas
    }

    /// Find the closest schema by L2 distance of ratio deviations from params.
    pub fn closest(&self, params: &ParamState) -> Option<&ProportionSchema> {
        let ratios = params_to_ratios(params);
        self.schemas.iter().min_by(|a, b| {
            let da = schema_l2_distance(a, &ratios);
            let db = schema_l2_distance(b, &ratios);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Analyze params against the named schema.
    pub fn analyze(&self, params: &ParamState, schema_name: &str) -> Option<ProportionAnalysis> {
        let schema = self.find(schema_name)?;
        let ratios = params_to_ratios(params);
        let deviations = schema_deviations(schema, &ratios);

        let rms_deviation = {
            let sum_sq: f32 = deviations.values().map(|v| v * v).sum();
            (sum_sq / deviations.len() as f32).sqrt()
        };

        let closest_schema = self
            .closest(params)
            .map(|s| s.name.clone())
            .unwrap_or_default();

        Some(ProportionAnalysis {
            schema_name: schema_name.to_string(),
            deviations,
            rms_deviation,
            closest_schema,
        })
    }
}

// ---------------------------------------------------------------------------
// Built-in schemas factory
// ---------------------------------------------------------------------------

/// Returns a library pre-loaded with standard artistic proportion schemas.
pub fn standard_schemas() -> ProportionLibrary {
    let mut lib = ProportionLibrary::new();

    lib.add(ProportionSchema {
        name: "vitruvian".to_string(),
        heads_tall: 8.0,
        shoulder_ratio: 1.5,
        hip_ratio: 1.3,
        leg_ratio: 0.53,
        arm_ratio: 0.45,
        description: "Classical Vitruvian Man proportions (Leonardo da Vinci)".to_string(),
    });

    lib.add(ProportionSchema {
        name: "fashion".to_string(),
        heads_tall: 9.0,
        shoulder_ratio: 1.4,
        hip_ratio: 1.2,
        leg_ratio: 0.56,
        arm_ratio: 0.44,
        description: "Fashion illustration proportions — elongated legs".to_string(),
    });

    lib.add(ProportionSchema {
        name: "heroic".to_string(),
        heads_tall: 8.5,
        shoulder_ratio: 1.8,
        hip_ratio: 1.2,
        leg_ratio: 0.54,
        arm_ratio: 0.46,
        description: "Heroic/comic-book proportions — broad shoulders".to_string(),
    });

    lib.add(ProportionSchema {
        name: "child_6yr".to_string(),
        heads_tall: 6.0,
        shoulder_ratio: 1.2,
        hip_ratio: 1.1,
        leg_ratio: 0.47,
        arm_ratio: 0.40,
        description: "Approximate proportions of a 6-year-old child".to_string(),
    });

    lib.add(ProportionSchema {
        name: "realistic".to_string(),
        heads_tall: 7.5,
        shoulder_ratio: 1.4,
        hip_ratio: 1.3,
        leg_ratio: 0.52,
        arm_ratio: 0.44,
        description: "Realistic adult human proportions".to_string(),
    });

    lib
}

// ---------------------------------------------------------------------------
// Standalone functions
// ---------------------------------------------------------------------------

/// Derive approximate artistic ratios from ParamState.
///
/// Mapping conventions (all values are [0, 1]):
/// - `height` (0=short ~1.5 m, 1=tall ~2.0 m) maps to heads_tall ≈ 6.0–9.0
/// - `weight` (0=thin, 1=heavy) affects shoulder/hip ratios
/// - `muscle` (0=none, 1=maximum) increases shoulder_ratio
/// - `age`   (0=child, 1=elder) affects leg_ratio
pub fn params_to_ratios(params: &ParamState) -> HashMap<String, f32> {
    let mut map = HashMap::new();

    // heads_tall: 6.0 (short/child) → 9.0 (tall/fashion)
    let heads_tall = 6.0 + params.height * 3.0;
    map.insert("heads_tall".to_string(), heads_tall);

    // shoulder_ratio: base 1.2–1.8, muscle adds up to 0.4
    let shoulder_ratio = 1.2 + params.weight * 0.2 + params.muscle * 0.4;
    map.insert("shoulder_ratio".to_string(), shoulder_ratio);

    // hip_ratio: 1.1–1.5, increases slightly with weight
    let hip_ratio = 1.1 + params.weight * 0.4;
    map.insert("hip_ratio".to_string(), hip_ratio);

    // leg_ratio: child (low age) → shorter legs; adult → longer
    let leg_ratio = 0.47 + params.age.clamp(0.0, 1.0) * 0.09;
    map.insert("leg_ratio".to_string(), leg_ratio);

    // arm_ratio: fairly stable, slight muscle influence
    let arm_ratio = 0.40 + params.muscle * 0.06;
    map.insert("arm_ratio".to_string(), arm_ratio);

    map
}

/// Adjust params to better match the given schema proportions.
///
/// Solves the inverse of `params_to_ratios` for the four primary params.
pub fn normalize_to_schema(params: &mut ParamState, schema: &ProportionSchema) {
    // Invert heads_tall → height
    // heads_tall = 6.0 + height * 3.0  →  height = (heads_tall - 6.0) / 3.0
    params.height = ((schema.heads_tall - 6.0) / 3.0).clamp(0.0, 1.0);

    // Invert leg_ratio → age
    // leg_ratio = 0.47 + age * 0.09  →  age = (leg_ratio - 0.47) / 0.09
    params.age = ((schema.leg_ratio - 0.47) / 0.09).clamp(0.0, 1.0);

    // Invert shoulder_ratio → muscle (holding weight at current value)
    // shoulder_ratio = 1.2 + weight * 0.2 + muscle * 0.4
    // muscle = (shoulder_ratio - 1.2 - weight * 0.2) / 0.4
    let muscle_raw = (schema.shoulder_ratio - 1.2 - params.weight * 0.2) / 0.4;
    params.muscle = muscle_raw.clamp(0.0, 1.0);

    // Invert hip_ratio → weight
    // hip_ratio = 1.1 + weight * 0.4  →  weight = (hip_ratio - 1.1) / 0.4
    params.weight = ((schema.hip_ratio - 1.1) / 0.4).clamp(0.0, 1.0);

    // Re-solve muscle with the newly determined weight
    let muscle_corrected = (schema.shoulder_ratio - 1.2 - params.weight * 0.2) / 0.4;
    params.muscle = muscle_corrected.clamp(0.0, 1.0);
}

/// Compute a proportion score: 0.0 = perfect match, higher = more deviation.
///
/// Returns the RMS of all ratio deviations (normalised to the schema value).
pub fn proportion_score(params: &ParamState, schema: &ProportionSchema) -> f32 {
    let ratios = params_to_ratios(params);
    let devs = schema_deviations(schema, &ratios);
    if devs.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = devs.values().map(|v| v * v).sum();
    (sum_sq / devs.len() as f32).sqrt()
}

/// Return a ParamState that approximates golden-ratio body proportions.
///
/// The golden ratio φ ≈ 1.618 governs classic aesthetic ideals.
/// We target the "vitruvian" schema (8 heads tall) with φ-derived tweaks.
pub fn golden_ratio_params() -> ParamState {
    const PHI: f32 = 1.618_034;
    // heads_tall ≈ 8.0  → height = (8.0 - 6.0) / 3.0 ≈ 0.667
    let height = (8.0_f32 - 6.0) / 3.0;
    // Navel divides body at φ: leg fraction ≈ 1/φ ≈ 0.618 → leg_ratio ≈ 0.53 maps to vitruvian
    // age = (0.53 - 0.47) / 0.09 ≈ 0.667
    let age = (0.53_f32 - 0.47) / 0.09;
    // Shoulder/hip ratio at golden proportion: shoulder ~ 1.5, hip ~ 1.3 (vitruvian)
    // muscle = (1.5 - 1.2 - 0.5 * 0.2) / 0.4 = (0.3 - 0.1) / 0.4 = 0.5
    let weight = (1.3_f32 - 1.1) / 0.4; // 0.5
    let muscle = (1.5_f32 - 1.2 - weight * 0.2) / 0.4;

    let mut p = ParamState::new(
        height.clamp(0.0, 1.0),
        weight.clamp(0.0, 1.0),
        muscle.clamp(0.0, 1.0),
        age.clamp(0.0, 1.0),
    );
    p.extra.insert("phi".to_string(), PHI);
    p
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Extract the five canonical ratios from a ProportionSchema into a map.
fn schema_to_ratio_map(schema: &ProportionSchema) -> HashMap<String, f32> {
    let mut m = HashMap::new();
    m.insert("heads_tall".to_string(), schema.heads_tall);
    m.insert("shoulder_ratio".to_string(), schema.shoulder_ratio);
    m.insert("hip_ratio".to_string(), schema.hip_ratio);
    m.insert("leg_ratio".to_string(), schema.leg_ratio);
    m.insert("arm_ratio".to_string(), schema.arm_ratio);
    m
}

/// Compute signed deviations (params_ratio - schema_ratio) for each key.
fn schema_deviations(
    schema: &ProportionSchema,
    ratios: &HashMap<String, f32>,
) -> HashMap<String, f32> {
    let ideal = schema_to_ratio_map(schema);
    let mut devs = HashMap::new();
    for (key, ideal_val) in &ideal {
        if let Some(&actual_val) = ratios.get(key.as_str()) {
            devs.insert(key.clone(), actual_val - ideal_val);
        }
    }
    devs
}

/// L2 distance between a schema's ratios and a ratio map.
fn schema_l2_distance(schema: &ProportionSchema, ratios: &HashMap<String, f32>) -> f32 {
    let ideal = schema_to_ratio_map(schema);
    let mut sum_sq = 0.0_f32;
    for (key, ideal_val) in &ideal {
        let actual = ratios.get(key.as_str()).copied().unwrap_or(*ideal_val);
        let d = actual - ideal_val;
        sum_sq += d * d;
    }
    sum_sq.sqrt()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> ParamState {
        ParamState::new(0.5, 0.5, 0.5, 0.5)
    }

    // -----------------------------------------------------------------------
    // 1. ProportionLibrary::new / add / find
    // -----------------------------------------------------------------------

    #[test]
    fn library_add_and_find() {
        let mut lib = ProportionLibrary::new();
        lib.add(ProportionSchema {
            name: "test".to_string(),
            heads_tall: 7.0,
            shoulder_ratio: 1.3,
            hip_ratio: 1.2,
            leg_ratio: 0.50,
            arm_ratio: 0.43,
            description: "test schema".to_string(),
        });
        let found = lib.find("test");
        assert!(found.is_some());
        assert!((found.unwrap().heads_tall - 7.0).abs() < 1e-6);
    }

    #[test]
    fn library_find_missing_returns_none() {
        let lib = ProportionLibrary::new();
        assert!(lib.find("nonexistent").is_none());
    }

    #[test]
    fn library_find_is_case_sensitive() {
        let mut lib = ProportionLibrary::new();
        lib.add(ProportionSchema {
            name: "Vitruvian".to_string(),
            heads_tall: 8.0,
            shoulder_ratio: 1.5,
            hip_ratio: 1.3,
            leg_ratio: 0.53,
            arm_ratio: 0.45,
            description: String::new(),
        });
        assert!(lib.find("vitruvian").is_none());
        assert!(lib.find("Vitruvian").is_some());
    }

    // -----------------------------------------------------------------------
    // 2. standard_schemas
    // -----------------------------------------------------------------------

    #[test]
    fn standard_schemas_has_five_entries() {
        let lib = standard_schemas();
        let names = ["vitruvian", "fashion", "heroic", "child_6yr", "realistic"];
        for name in &names {
            assert!(lib.find(name).is_some(), "missing schema: {}", name);
        }
    }

    #[test]
    fn vitruvian_schema_values() {
        let lib = standard_schemas();
        let s = lib.find("vitruvian").unwrap();
        assert!((s.heads_tall - 8.0).abs() < 1e-6);
        assert!((s.shoulder_ratio - 1.5).abs() < 1e-6);
        assert!((s.hip_ratio - 1.3).abs() < 1e-6);
        assert!((s.leg_ratio - 0.53).abs() < 1e-6);
        assert!((s.arm_ratio - 0.45).abs() < 1e-6);
    }

    #[test]
    fn fashion_schema_is_tallest() {
        let lib = standard_schemas();
        let fashion = lib.find("fashion").unwrap();
        let vitruvian = lib.find("vitruvian").unwrap();
        assert!(fashion.heads_tall > vitruvian.heads_tall);
    }

    #[test]
    fn heroic_schema_has_widest_shoulders() {
        let lib = standard_schemas();
        let heroic = lib.find("heroic").unwrap();
        let vitruvian = lib.find("vitruvian").unwrap();
        assert!(heroic.shoulder_ratio > vitruvian.shoulder_ratio);
    }

    // -----------------------------------------------------------------------
    // 3. params_to_ratios
    // -----------------------------------------------------------------------

    #[test]
    fn params_to_ratios_zero_params() {
        let p = ParamState::new(0.0, 0.0, 0.0, 0.0);
        let r = params_to_ratios(&p);
        assert!((r["heads_tall"] - 6.0).abs() < 1e-5);
        assert!((r["shoulder_ratio"] - 1.2).abs() < 1e-5);
        assert!((r["hip_ratio"] - 1.1).abs() < 1e-5);
        assert!((r["leg_ratio"] - 0.47).abs() < 1e-5);
        assert!((r["arm_ratio"] - 0.40).abs() < 1e-5);
    }

    #[test]
    fn params_to_ratios_one_params() {
        let p = ParamState::new(1.0, 1.0, 1.0, 1.0);
        let r = params_to_ratios(&p);
        assert!((r["heads_tall"] - 9.0).abs() < 1e-5);
        assert!((r["shoulder_ratio"] - 1.8).abs() < 1e-5);
        assert!((r["hip_ratio"] - 1.5).abs() < 1e-5);
        assert!((r["leg_ratio"] - 0.56).abs() < 1e-5);
        assert!((r["arm_ratio"] - 0.46).abs() < 1e-5);
    }

    #[test]
    fn params_to_ratios_contains_all_keys() {
        let r = params_to_ratios(&default_params());
        for key in &[
            "heads_tall",
            "shoulder_ratio",
            "hip_ratio",
            "leg_ratio",
            "arm_ratio",
        ] {
            assert!(r.contains_key(*key), "missing key: {}", key);
        }
    }

    // -----------------------------------------------------------------------
    // 4. proportion_score
    // -----------------------------------------------------------------------

    #[test]
    fn proportion_score_exact_match_is_zero() {
        // Build params that exactly map to vitruvian
        let lib = standard_schemas();
        let schema = lib.find("vitruvian").unwrap();
        let mut p = ParamState::default();
        normalize_to_schema(&mut p, schema);
        let score = proportion_score(&p, schema);
        // After normalization the score should be very close to zero
        assert!(score < 0.05, "expected near-zero score, got {}", score);
    }

    #[test]
    fn proportion_score_different_params_is_nonzero() {
        let lib = standard_schemas();
        let schema = lib.find("heroic").unwrap();
        let p = ParamState::new(0.0, 0.0, 0.0, 0.0); // child-like params
        let score = proportion_score(&p, schema);
        assert!(score > 0.0, "expected non-zero score");
    }

    // -----------------------------------------------------------------------
    // 5. ProportionLibrary::closest
    // -----------------------------------------------------------------------

    #[test]
    fn closest_child_params_returns_child_schema() {
        let lib = standard_schemas();
        // height=0.0, weight=0.0, muscle=0.0, age=0.0 → child-like
        let p = ParamState::new(0.0, 0.0, 0.0, 0.0);
        let closest = lib.closest(&p).unwrap();
        assert_eq!(closest.name, "child_6yr");
    }

    #[test]
    fn closest_tall_muscular_params_returns_heroic_or_fashion() {
        let lib = standard_schemas();
        // height=1.0, muscle=1.0 → heroic proportions
        let p = ParamState::new(1.0, 0.0, 1.0, 1.0);
        let closest = lib.closest(&p).unwrap();
        assert!(
            closest.name == "heroic" || closest.name == "fashion",
            "unexpected schema: {}",
            closest.name
        );
    }

    #[test]
    fn closest_empty_library_returns_none() {
        let lib = ProportionLibrary::new();
        let p = default_params();
        assert!(lib.closest(&p).is_none());
    }

    // -----------------------------------------------------------------------
    // 6. ProportionLibrary::analyze
    // -----------------------------------------------------------------------

    #[test]
    fn analyze_returns_correct_schema_name() {
        let lib = standard_schemas();
        let p = default_params();
        let analysis = lib.analyze(&p, "vitruvian").unwrap();
        assert_eq!(analysis.schema_name, "vitruvian");
    }

    #[test]
    fn analyze_deviations_has_all_keys() {
        let lib = standard_schemas();
        let p = default_params();
        let analysis = lib.analyze(&p, "realistic").unwrap();
        for key in &[
            "heads_tall",
            "shoulder_ratio",
            "hip_ratio",
            "leg_ratio",
            "arm_ratio",
        ] {
            assert!(analysis.deviations.contains_key(*key));
        }
    }

    #[test]
    fn analyze_rms_deviation_nonnegative() {
        let lib = standard_schemas();
        let p = default_params();
        let analysis = lib.analyze(&p, "fashion").unwrap();
        assert!(analysis.rms_deviation >= 0.0);
    }

    #[test]
    fn analyze_missing_schema_returns_none() {
        let lib = standard_schemas();
        let p = default_params();
        assert!(lib.analyze(&p, "does_not_exist").is_none());
    }

    // -----------------------------------------------------------------------
    // 7. normalize_to_schema
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_to_schema_then_score_is_low() {
        let lib = standard_schemas();
        for name in &["vitruvian", "fashion", "heroic", "child_6yr", "realistic"] {
            let schema = lib.find(name).unwrap();
            let mut p = ParamState::default();
            normalize_to_schema(&mut p, schema);
            let score = proportion_score(&p, schema);
            assert!(
                score < 0.1,
                "schema '{}': score {} is too high after normalization",
                name,
                score
            );
        }
    }

    #[test]
    fn normalize_clamps_params_to_unit_interval() {
        // Use a schema with extreme values that might push params out of [0,1]
        let schema = ProportionSchema {
            name: "extreme".to_string(),
            heads_tall: 12.0,    // would map to height > 1
            shoulder_ratio: 3.0, // would map to muscle > 1
            hip_ratio: 0.5,      // would map to weight < 0
            leg_ratio: 0.10,     // would map to age < 0
            arm_ratio: 0.50,
            description: String::new(),
        };
        let mut p = ParamState::default();
        normalize_to_schema(&mut p, &schema);
        assert!(p.height >= 0.0 && p.height <= 1.0);
        assert!(p.weight >= 0.0 && p.weight <= 1.0);
        assert!(p.muscle >= 0.0 && p.muscle <= 1.0);
        assert!(p.age >= 0.0 && p.age <= 1.0);
    }

    // -----------------------------------------------------------------------
    // 8. golden_ratio_params
    // -----------------------------------------------------------------------

    #[test]
    fn golden_ratio_params_in_unit_range() {
        let p = golden_ratio_params();
        assert!(p.height >= 0.0 && p.height <= 1.0);
        assert!(p.weight >= 0.0 && p.weight <= 1.0);
        assert!(p.muscle >= 0.0 && p.muscle <= 1.0);
        assert!(p.age >= 0.0 && p.age <= 1.0);
    }

    #[test]
    fn golden_ratio_params_close_to_vitruvian() {
        let lib = standard_schemas();
        let vitruvian = lib.find("vitruvian").unwrap();
        let p = golden_ratio_params();
        let score = proportion_score(&p, vitruvian);
        // Should be reasonably close to vitruvian proportions
        assert!(
            score < 0.5,
            "golden ratio params score {} vs vitruvian",
            score
        );
    }

    #[test]
    fn golden_ratio_params_contains_phi_extra() {
        let p = golden_ratio_params();
        let phi = p.extra.get("phi").copied().unwrap_or(0.0);
        assert!((phi - 1.618_034).abs() < 1e-4);
    }

    // -----------------------------------------------------------------------
    // 9. Default trait
    // -----------------------------------------------------------------------

    #[test]
    fn proportion_library_default_is_empty() {
        let lib = ProportionLibrary::default();
        assert!(lib.find("anything").is_none());
    }
}
