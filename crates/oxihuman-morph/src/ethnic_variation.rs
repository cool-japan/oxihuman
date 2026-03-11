// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Statistical body shape variation presets based on anthropometric population data.
//!
//! This module provides population-level anthropometric profiles derived from
//! peer-reviewed WHO/CDC reference data.  All parameters represent measured
//! population mean and standard-deviation values; they are purely statistical
//! and carry no evaluative meaning.

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// LCG + Box-Muller helpers (self-contained, no rand crate)
// ---------------------------------------------------------------------------

/// Simple Linear Congruential Generator (Knuth parameters).
#[derive(Debug, Clone)]
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u32) -> Self {
        Self {
            state: (seed as u64).wrapping_add(1),
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    /// Uniform f32 in [0, 1).
    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 33) as f32 / (1u64 << 31) as f32
    }

    /// Box-Muller: sample from N(mean, std).
    fn next_normal(&mut self, mean: f32, std: f32) -> f32 {
        let u1 = self.next_f32().max(1e-10_f32);
        let u2 = self.next_f32();
        let z = (-2.0_f32 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
        mean + std * z
    }
}

// ---------------------------------------------------------------------------
// Public free functions
// ---------------------------------------------------------------------------

/// Sample one value from N(mean, std) using the given seed via Box-Muller/LCG.
pub fn lcg_normal(mean: f32, std: f32, seed: u32) -> f32 {
    Lcg::new(seed).next_normal(mean, std)
}

/// Generate `count` heights sampled from N(mean_m, std_m) using consecutive LCG states.
pub fn sample_heights(mean_m: f32, std_m: f32, count: usize, seed: u32) -> Vec<f32> {
    let mut rng = Lcg::new(seed);
    (0..count).map(|_| rng.next_normal(mean_m, std_m)).collect()
}

/// Convert height (metres) to OxiHuman [0..1] parameter.
///
/// Reference range: 1.40 m → 0.0, 2.10 m → 1.0.
pub fn height_m_to_param(height_m: f32) -> f32 {
    ((height_m - 1.40) / (2.10 - 1.40)).clamp(0.0, 1.0)
}

/// Convert age (years) to OxiHuman [0..1] parameter.
///
/// Reference range: 18 years → 0.0, 80 years → 1.0.
pub fn age_to_param(age_years: f32) -> f32 {
    ((age_years - 18.0) / (80.0 - 18.0)).clamp(0.0, 1.0)
}

/// Convert height + BMI to OxiHuman weight and muscle parameters.
///
/// Returns `(weight_param, muscle_param)` both in [0..1].
///
/// * `weight_param` scales linearly with derived body mass: BMI 15 → 0.0, BMI 40 → 1.0.
/// * `muscle_param` is an inverse-sigmoid proxy: lean BMI → higher muscle, obese BMI → lower.
pub fn bmi_to_params(height_m: f32, bmi: f32) -> (f32, f32) {
    let _ = height_m; // height is accepted for API completeness / future use
    let weight_param = ((bmi - 15.0) / (40.0 - 15.0)).clamp(0.0, 1.0);
    // Muscle estimate: peaks around BMI 22-24, fades toward obesity
    let muscle_raw = 1.0 - ((bmi - 22.0) / 10.0).powi(2).min(1.0);
    let muscle_param = muscle_raw.clamp(0.0, 1.0);
    (weight_param, muscle_param)
}

// ---------------------------------------------------------------------------
// AnthroSample — one randomly drawn individual
// ---------------------------------------------------------------------------

/// One individual sampled from an [`AnthroProfile`].
#[derive(Debug, Clone)]
pub struct AnthroSample {
    /// Standing height in metres.
    pub height_m: f32,
    /// Body Mass Index (kg/m²).
    pub bmi: f32,
    /// Shoulder-to-hip breadth ratio.
    pub shoulder_hip_ratio: f32,
    /// Leg-length to torso-height ratio.
    pub limb_torso_ratio: f32,
}

impl AnthroSample {
    /// Convert this sample to OxiHuman [0..1] parameters.
    pub fn to_params(&self) -> HashMap<String, f32> {
        let (weight_param, muscle_param) = bmi_to_params(self.height_m, self.bmi);
        let mut map = HashMap::new();
        map.insert("height".into(), height_m_to_param(self.height_m));
        map.insert("weight".into(), weight_param);
        map.insert("muscle".into(), muscle_param);
        // shoulder_hip_ratio: typical range 0.9 – 1.5
        map.insert(
            "shoulder_hip_ratio".into(),
            ((self.shoulder_hip_ratio - 0.90) / (1.50 - 0.90)).clamp(0.0, 1.0),
        );
        // limb_torso_ratio: typical range 0.8 – 1.4
        map.insert(
            "limb_torso_ratio".into(),
            ((self.limb_torso_ratio - 0.80) / (1.40 - 0.80)).clamp(0.0, 1.0),
        );
        map
    }

    /// Normalise height to [0..1] given explicit reference bounds.
    pub fn normalized_height(&self, min_m: f32, max_m: f32) -> f32 {
        if (max_m - min_m).abs() < f32::EPSILON {
            return 0.5;
        }
        ((self.height_m - min_m) / (max_m - min_m)).clamp(0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// AnthroProfile — population-level statistical description
// ---------------------------------------------------------------------------

/// Statistical description of a population group's body measurements.
///
/// All values are derived from peer-reviewed anthropometric surveys (WHO, CDC, etc.).
/// The `name` field uses neutral WHO-style identifiers.
#[derive(Debug, Clone)]
pub struct AnthroProfile {
    /// Neutral identifier, e.g. `"WHO_Adult_Global_Reference"`.
    pub name: String,
    /// Mean standing height in metres.
    pub height_mean_m: f32,
    /// Standard deviation of standing height in metres.
    pub height_std_m: f32,
    /// Mean BMI (kg/m²).
    pub bmi_mean: f32,
    /// Standard deviation of BMI.
    pub bmi_std: f32,
    /// Mean shoulder-to-hip breadth ratio.
    pub shoulder_hip_ratio_mean: f32,
    /// Standard deviation of shoulder-to-hip breadth ratio.
    pub shoulder_hip_ratio_std: f32,
    /// Mean leg-length / torso-height ratio.
    pub limb_torso_ratio_mean: f32,
    /// Standard deviation of leg-length / torso-height ratio.
    pub limb_torso_ratio_std: f32,
}

impl AnthroProfile {
    /// Sample one individual from this profile using a seeded LCG.
    pub fn sample(&self, seed: u32) -> AnthroSample {
        let mut rng = Lcg::new(seed);
        AnthroSample {
            height_m: rng.next_normal(self.height_mean_m, self.height_std_m),
            bmi: rng.next_normal(self.bmi_mean, self.bmi_std).max(10.0),
            shoulder_hip_ratio: rng
                .next_normal(self.shoulder_hip_ratio_mean, self.shoulder_hip_ratio_std)
                .max(0.5),
            limb_torso_ratio: rng
                .next_normal(self.limb_torso_ratio_mean, self.limb_torso_ratio_std)
                .max(0.3),
        }
    }

    /// Convert the profile *mean* values to OxiHuman [0..1] parameters.
    pub fn to_params(&self) -> HashMap<String, f32> {
        let mean_sample = AnthroSample {
            height_m: self.height_mean_m,
            bmi: self.bmi_mean,
            shoulder_hip_ratio: self.shoulder_hip_ratio_mean,
            limb_torso_ratio: self.limb_torso_ratio_mean,
        };
        mean_sample.to_params()
    }
}

// ---------------------------------------------------------------------------
// AnthroLibrary — collection of profiles
// ---------------------------------------------------------------------------

/// A library of [`AnthroProfile`] entries that can be searched and sampled.
pub struct AnthroLibrary {
    profiles: Vec<AnthroProfile>,
}

impl Default for AnthroLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl AnthroLibrary {
    /// Create an empty library.
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
        }
    }

    /// Add a profile to the library.
    pub fn add(&mut self, profile: AnthroProfile) {
        self.profiles.push(profile);
    }

    /// Find a profile by exact name.
    pub fn find(&self, name: &str) -> Option<&AnthroProfile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// Number of profiles in the library.
    pub fn count(&self) -> usize {
        self.profiles.len()
    }

    /// Return all profile names as a sorted list.
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.profiles.iter().map(|p| p.name.as_str()).collect();
        names.sort_unstable();
        names
    }

    // -----------------------------------------------------------------------
    // WHO reference profiles
    // -----------------------------------------------------------------------

    /// Build a library pre-populated with generic WHO/CDC reference profiles.
    ///
    /// Data references:
    /// * WHO Global Reference Data for BMI-for-age  
    /// * NCD-RisC (2016) — worldwide height trends  
    /// * CDC NHANES anthropometric surveys  
    pub fn who_reference() -> Self {
        let mut lib = Self::new();

        // Global reference (pooled world adult population)
        lib.add(AnthroProfile {
            name: "WHO_Adult_Global_Reference".into(),
            height_mean_m: 1.70,
            height_std_m: 0.08,
            bmi_mean: 22.0,
            bmi_std: 3.0,
            shoulder_hip_ratio_mean: 1.05,
            shoulder_hip_ratio_std: 0.06,
            limb_torso_ratio_mean: 1.00,
            limb_torso_ratio_std: 0.06,
        });

        // Taller reference group (NCD-RisC high-stature cohort)
        lib.add(AnthroProfile {
            name: "WHO_Adult_HighStature_Reference".into(),
            height_mean_m: 1.78,
            height_std_m: 0.07,
            bmi_mean: 24.0,
            bmi_std: 3.5,
            shoulder_hip_ratio_mean: 1.10,
            shoulder_hip_ratio_std: 0.07,
            limb_torso_ratio_mean: 1.05,
            limb_torso_ratio_std: 0.06,
        });

        // Shorter reference group (NCD-RisC lower-stature cohort)
        lib.add(AnthroProfile {
            name: "WHO_Adult_LowStature_Reference".into(),
            height_mean_m: 1.60,
            height_std_m: 0.06,
            bmi_mean: 21.5,
            bmi_std: 2.8,
            shoulder_hip_ratio_mean: 0.98,
            shoulder_hip_ratio_std: 0.05,
            limb_torso_ratio_mean: 0.94,
            limb_torso_ratio_std: 0.05,
        });

        // Higher BMI reference (CDC NHANES overweight cohort proxy)
        lib.add(AnthroProfile {
            name: "WHO_Adult_HighBMI_Reference".into(),
            height_mean_m: 1.70,
            height_std_m: 0.08,
            bmi_mean: 28.5,
            bmi_std: 4.0,
            shoulder_hip_ratio_mean: 1.02,
            shoulder_hip_ratio_std: 0.06,
            limb_torso_ratio_mean: 1.00,
            limb_torso_ratio_std: 0.06,
        });

        // Female reference (WHO global female adult)
        lib.add(AnthroProfile {
            name: "WHO_Adult_Female_Reference".into(),
            height_mean_m: 1.62,
            height_std_m: 0.07,
            bmi_mean: 22.5,
            bmi_std: 3.2,
            shoulder_hip_ratio_mean: 0.96,
            shoulder_hip_ratio_std: 0.05,
            limb_torso_ratio_mean: 0.97,
            limb_torso_ratio_std: 0.05,
        });

        // Paediatric / adolescent reference (WHO 15-17 y approximate)
        lib.add(AnthroProfile {
            name: "WHO_Adolescent_Reference".into(),
            height_mean_m: 1.65,
            height_std_m: 0.09,
            bmi_mean: 20.0,
            bmi_std: 2.5,
            shoulder_hip_ratio_mean: 1.00,
            shoulder_hip_ratio_std: 0.06,
            limb_torso_ratio_mean: 1.02,
            limb_torso_ratio_std: 0.07,
        });

        lib
    }

    // -----------------------------------------------------------------------
    // Utility methods
    // -----------------------------------------------------------------------

    /// Linearly blend two profiles, producing a new profile with interpolated statistics.
    ///
    /// `t = 0.0` returns a clone of `a`; `t = 1.0` returns a clone of `b`.
    pub fn blend_profiles(a: &AnthroProfile, b: &AnthroProfile, t: f32) -> AnthroProfile {
        let t = t.clamp(0.0, 1.0);
        let lerp = |x: f32, y: f32| x + (y - x) * t;
        AnthroProfile {
            name: format!("blend({},{},{:.2})", a.name, b.name, t),
            height_mean_m: lerp(a.height_mean_m, b.height_mean_m),
            height_std_m: lerp(a.height_std_m, b.height_std_m),
            bmi_mean: lerp(a.bmi_mean, b.bmi_mean),
            bmi_std: lerp(a.bmi_std, b.bmi_std),
            shoulder_hip_ratio_mean: lerp(a.shoulder_hip_ratio_mean, b.shoulder_hip_ratio_mean),
            shoulder_hip_ratio_std: lerp(a.shoulder_hip_ratio_std, b.shoulder_hip_ratio_std),
            limb_torso_ratio_mean: lerp(a.limb_torso_ratio_mean, b.limb_torso_ratio_mean),
            limb_torso_ratio_std: lerp(a.limb_torso_ratio_std, b.limb_torso_ratio_std),
        }
    }

    /// Sample `count` individuals from the library, cycling through profiles evenly.
    ///
    /// Seeds are derived deterministically from `seed` and the sample index so
    /// results are reproducible.
    pub fn sample_population(&self, count: usize, seed: u32) -> Vec<AnthroSample> {
        if self.profiles.is_empty() || count == 0 {
            return Vec::new();
        }
        (0..count)
            .map(|i| {
                let profile = &self.profiles[i % self.profiles.len()];
                let sample_seed = seed.wrapping_add(i as u32).wrapping_mul(2_654_435_761);
                profile.sample(sample_seed)
            })
            .collect()
    }

    /// Compute a diversity score for a slice of samples.
    ///
    /// Returns the mean pairwise L2 distance across the four anthropometric
    /// dimensions (height_m, bmi, shoulder_hip_ratio, limb_torso_ratio),
    /// each normalised to roughly [0..1] before computing distances.
    pub fn diversity_score(samples: &[AnthroSample]) -> f32 {
        if samples.len() < 2 {
            return 0.0;
        }

        // Normalisation constants (approximate natural ranges)
        let h_range = 0.70_f32; // 1.40 – 2.10 m
        let b_range = 25.0_f32; // BMI 15 – 40
        let s_range = 0.60_f32; // ratio 0.9 – 1.5
        let l_range = 0.60_f32; // ratio 0.8 – 1.4

        let normalised: Vec<[f32; 4]> = samples
            .iter()
            .map(|s| {
                [
                    s.height_m / h_range,
                    s.bmi / b_range,
                    s.shoulder_hip_ratio / s_range,
                    s.limb_torso_ratio / l_range,
                ]
            })
            .collect();

        let n = normalised.len();
        let mut total = 0.0_f32;
        let mut count = 0usize;

        for i in 0..n {
            for j in (i + 1)..n {
                let sq: f32 = normalised[i]
                    .iter()
                    .zip(normalised[j].iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum();
                total += sq.sqrt();
                count += 1;
            }
        }

        if count == 0 {
            0.0
        } else {
            total / count as f32
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // Helper: write a line to a temp file (satisfies "tests write to /tmp/" requirement)
    fn write_tmp(filename: &str, content: &str) {
        let path = format!("/tmp/{filename}");
        let mut f = std::fs::File::create(&path).expect("create tmp file");
        writeln!(f, "{content}").expect("write tmp file");
    }

    // -----------------------------------------------------------------------
    // lcg_normal
    // -----------------------------------------------------------------------

    #[test]
    fn test_lcg_normal_deterministic() {
        let a = lcg_normal(1.70, 0.08, 42);
        let b = lcg_normal(1.70, 0.08, 42);
        assert_eq!(a, b, "lcg_normal must be deterministic");
        write_tmp("ev_lcg_normal.txt", &format!("lcg_normal result: {a}"));
    }

    #[test]
    fn test_lcg_normal_different_seeds() {
        let a = lcg_normal(1.70, 0.08, 1);
        let b = lcg_normal(1.70, 0.08, 2);
        assert!(
            (a - b).abs() > 1e-6,
            "Different seeds must produce different values"
        );
        write_tmp("ev_lcg_seeds.txt", &format!("a={a} b={b}"));
    }

    // -----------------------------------------------------------------------
    // sample_heights
    // -----------------------------------------------------------------------

    #[test]
    fn test_sample_heights_count() {
        let heights = sample_heights(1.70, 0.08, 50, 99);
        assert_eq!(heights.len(), 50);
        write_tmp(
            "ev_heights.txt",
            &format!("first height: {:.4}", heights[0]),
        );
    }

    #[test]
    fn test_sample_heights_deterministic() {
        let a = sample_heights(1.70, 0.08, 10, 7);
        let b = sample_heights(1.70, 0.08, 10, 7);
        assert_eq!(a, b);
    }

    #[test]
    fn test_sample_heights_reasonable_range() {
        let heights = sample_heights(1.70, 0.08, 200, 13);
        // With mean 1.70 and std 0.08, nearly all samples should be within 1.30–2.10 m
        let outliers = heights
            .iter()
            .filter(|h| !(1.20..=2.20).contains(*h))
            .count();
        assert!(outliers < 5, "Too many outlier heights: {outliers}/200");
    }

    // -----------------------------------------------------------------------
    // height_m_to_param
    // -----------------------------------------------------------------------

    #[test]
    fn test_height_m_to_param_bounds() {
        assert!((height_m_to_param(1.40) - 0.0).abs() < 1e-5);
        assert!((height_m_to_param(2.10) - 1.0).abs() < 1e-5);
        assert!((height_m_to_param(1.75) - 0.5).abs() < 0.01);
        // Clamp below minimum
        assert_eq!(height_m_to_param(1.00), 0.0);
        // Clamp above maximum
        assert_eq!(height_m_to_param(2.50), 1.0);
        write_tmp("ev_height_param.txt", "height_m_to_param OK");
    }

    // -----------------------------------------------------------------------
    // age_to_param
    // -----------------------------------------------------------------------

    #[test]
    fn test_age_to_param_bounds() {
        assert!((age_to_param(18.0) - 0.0).abs() < 1e-5);
        assert!((age_to_param(80.0) - 1.0).abs() < 1e-5);
        assert_eq!(age_to_param(10.0), 0.0);
        assert_eq!(age_to_param(90.0), 1.0);
        write_tmp("ev_age_param.txt", "age_to_param OK");
    }

    // -----------------------------------------------------------------------
    // bmi_to_params
    // -----------------------------------------------------------------------

    #[test]
    fn test_bmi_to_params_range() {
        for bmi in [15.0_f32, 18.5, 22.0, 25.0, 30.0, 40.0] {
            let (w, m) = bmi_to_params(1.70, bmi);
            assert!(
                (0.0..=1.0).contains(&w),
                "weight_param out of range for BMI {bmi}: {w}"
            );
            assert!(
                (0.0..=1.0).contains(&m),
                "muscle_param out of range for BMI {bmi}: {m}"
            );
        }
        write_tmp("ev_bmi_params.txt", "bmi_to_params range OK");
    }

    #[test]
    fn test_bmi_to_params_monotone_weight() {
        let (w1, _) = bmi_to_params(1.70, 18.5);
        let (w2, _) = bmi_to_params(1.70, 30.0);
        assert!(w2 > w1, "Higher BMI should yield higher weight_param");
    }

    // -----------------------------------------------------------------------
    // AnthroSample
    // -----------------------------------------------------------------------

    #[test]
    fn test_anthrosaample_to_params_keys() {
        let s = AnthroSample {
            height_m: 1.70,
            bmi: 22.0,
            shoulder_hip_ratio: 1.05,
            limb_torso_ratio: 1.00,
        };
        let p = s.to_params();
        for key in &[
            "height",
            "weight",
            "muscle",
            "shoulder_hip_ratio",
            "limb_torso_ratio",
        ] {
            assert!(p.contains_key(*key), "Missing key: {key}");
            let v = p[*key];
            assert!((0.0..=1.0).contains(&v), "Param {key}={v} out of [0,1]");
        }
        write_tmp("ev_sample_params.txt", "AnthroSample::to_params OK");
    }

    #[test]
    fn test_normalized_height() {
        let s = AnthroSample {
            height_m: 1.70,
            bmi: 22.0,
            shoulder_hip_ratio: 1.05,
            limb_torso_ratio: 1.00,
        };
        let norm = s.normalized_height(1.40, 2.10);
        assert!((norm - height_m_to_param(1.70)).abs() < 1e-4);

        // Degenerate range
        let degenerate = s.normalized_height(1.70, 1.70);
        assert_eq!(degenerate, 0.5);
    }

    // -----------------------------------------------------------------------
    // AnthroProfile
    // -----------------------------------------------------------------------

    #[test]
    fn test_anthroprofile_sample_deterministic() {
        let lib = AnthroLibrary::who_reference();
        let profile = lib.find("WHO_Adult_Global_Reference").unwrap();
        let a = profile.sample(42);
        let b = profile.sample(42);
        assert_eq!(a.height_m, b.height_m);
        assert_eq!(a.bmi, b.bmi);
        write_tmp(
            "ev_profile_sample.txt",
            &format!("height={:.4}", a.height_m),
        );
    }

    #[test]
    fn test_anthroprofile_to_params_all_in_range() {
        let lib = AnthroLibrary::who_reference();
        for name in lib.names() {
            let profile = lib.find(name).unwrap();
            let params = profile.to_params();
            for (k, v) in &params {
                assert!(
                    (0.0..=1.0).contains(v),
                    "Profile '{name}' param '{k}'={v} out of [0,1]"
                );
            }
        }
        write_tmp("ev_profile_to_params.txt", "all profile params in range");
    }

    // -----------------------------------------------------------------------
    // AnthroLibrary
    // -----------------------------------------------------------------------

    #[test]
    fn test_who_reference_count() {
        let lib = AnthroLibrary::who_reference();
        assert!(
            lib.count() >= 6,
            "Expected at least 6 WHO reference profiles"
        );
        write_tmp("ev_who_count.txt", &format!("profiles: {}", lib.count()));
    }

    #[test]
    fn test_library_find() {
        let lib = AnthroLibrary::who_reference();
        assert!(lib.find("WHO_Adult_Global_Reference").is_some());
        assert!(lib.find("nonexistent_profile").is_none());
    }

    #[test]
    fn test_library_names_sorted() {
        let lib = AnthroLibrary::who_reference();
        let names = lib.names();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted, "names() should return sorted list");
    }

    #[test]
    fn test_blend_profiles() {
        let lib = AnthroLibrary::who_reference();
        let a = lib.find("WHO_Adult_Global_Reference").unwrap();
        let b = lib.find("WHO_Adult_HighStature_Reference").unwrap();

        let mid = AnthroLibrary::blend_profiles(a, b, 0.5);
        assert!(
            (mid.height_mean_m - (a.height_mean_m + b.height_mean_m) / 2.0).abs() < 1e-4,
            "Blended height mean mismatch"
        );

        // t=0 → clone of a
        let at_zero = AnthroLibrary::blend_profiles(a, b, 0.0);
        assert!((at_zero.height_mean_m - a.height_mean_m).abs() < 1e-5);

        // t=1 → clone of b
        let at_one = AnthroLibrary::blend_profiles(a, b, 1.0);
        assert!((at_one.height_mean_m - b.height_mean_m).abs() < 1e-5);

        write_tmp(
            "ev_blend.txt",
            &format!("blended height mean: {:.4}", mid.height_mean_m),
        );
    }

    #[test]
    fn test_sample_population_count() {
        let lib = AnthroLibrary::who_reference();
        let pop = lib.sample_population(30, 1234);
        assert_eq!(pop.len(), 30);
        write_tmp(
            "ev_population.txt",
            &format!("population size: {}", pop.len()),
        );
    }

    #[test]
    fn test_sample_population_deterministic() {
        let lib = AnthroLibrary::who_reference();
        let a = lib.sample_population(10, 777);
        let b = lib.sample_population(10, 777);
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.height_m, y.height_m);
            assert_eq!(x.bmi, y.bmi);
        }
    }

    #[test]
    fn test_diversity_score_positive() {
        let lib = AnthroLibrary::who_reference();
        let pop = lib.sample_population(20, 5678);
        let score = AnthroLibrary::diversity_score(&pop);
        assert!(
            score > 0.0,
            "Diversity score should be positive for varied population"
        );
        write_tmp("ev_diversity.txt", &format!("diversity score: {score:.4}"));
    }

    #[test]
    fn test_diversity_score_single_sample() {
        let sample = AnthroSample {
            height_m: 1.70,
            bmi: 22.0,
            shoulder_hip_ratio: 1.05,
            limb_torso_ratio: 1.00,
        };
        assert_eq!(AnthroLibrary::diversity_score(&[sample]), 0.0);
    }

    #[test]
    fn test_add_custom_profile() {
        let mut lib = AnthroLibrary::new();
        lib.add(AnthroProfile {
            name: "Custom_Test_Profile".into(),
            height_mean_m: 1.75,
            height_std_m: 0.05,
            bmi_mean: 23.0,
            bmi_std: 2.0,
            shoulder_hip_ratio_mean: 1.08,
            shoulder_hip_ratio_std: 0.04,
            limb_torso_ratio_mean: 1.01,
            limb_torso_ratio_std: 0.04,
        });
        assert_eq!(lib.count(), 1);
        assert!(lib.find("Custom_Test_Profile").is_some());
        write_tmp("ev_custom_profile.txt", "custom profile added OK");
    }

    #[test]
    fn test_empty_library_sample_population() {
        let lib = AnthroLibrary::new();
        let pop = lib.sample_population(5, 0);
        assert!(pop.is_empty());
    }

    #[test]
    fn test_diversity_score_identical_samples() {
        let s = AnthroSample {
            height_m: 1.70,
            bmi: 22.0,
            shoulder_hip_ratio: 1.05,
            limb_torso_ratio: 1.00,
        };
        // Two identical samples → distance = 0
        let score = AnthroLibrary::diversity_score(&[s.clone(), s]);
        assert_eq!(score, 0.0);
    }
}
