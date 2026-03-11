// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Population-level validation comparing generated body measurements
//! against reference datasets (NHANES, ANSUR) using Kolmogorov-Smirnov tests.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Population-level validator comparing generated body measurements
/// against reference datasets (NHANES, ANSUR).
#[derive(Debug, Clone)]
pub struct PopulationValidator {
    reference_data: Vec<ReferenceDataset>,
}

/// A reference dataset with summary statistics for multiple measurements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceDataset {
    pub name: String,
    pub source: DataSource,
    pub measurements: Vec<PopulationMeasurement>,
}

/// Origin of the reference data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSource {
    /// US National Health and Nutrition Examination Survey
    Nhanes,
    /// US Army Anthropometric Survey
    Ansur,
    /// User-supplied data
    Custom,
}

/// Sex category used to separate reference statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
}

/// Summary statistics for one measurement within a reference population.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationMeasurement {
    pub name: String,
    pub sex: Sex,
    pub mean: f64,
    pub std_dev: f64,
    pub percentiles: Percentiles,
    pub sample_size: usize,
}

/// Selected percentile values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Percentiles {
    pub p1: f64,
    pub p5: f64,
    pub p10: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

impl Percentiles {
    /// Create from a normal distribution approximation using mean and std_dev.
    fn from_normal(mean: f64, std_dev: f64) -> Self {
        Self {
            p1: mean + std_dev * Z_001,
            p5: mean + std_dev * Z_005,
            p10: mean + std_dev * Z_010,
            p25: mean + std_dev * Z_025,
            p50: mean,
            p75: mean + std_dev * Z_075,
            p90: mean + std_dev * Z_090,
            p95: mean + std_dev * Z_095,
            p99: mean + std_dev * Z_099,
        }
    }
}

// Standard normal z-scores for common percentiles
const Z_001: f64 = -2.326;
const Z_005: f64 = -1.645;
const Z_010: f64 = -1.282;
const Z_025: f64 = -0.674;
const Z_075: f64 = 0.674;
const Z_090: f64 = 1.282;
const Z_095: f64 = 1.645;
const Z_099: f64 = 2.326;

// ---------------------------------------------------------------------------
// KS test result & validation report
// ---------------------------------------------------------------------------

/// Result of a Kolmogorov-Smirnov test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsTestResult {
    pub measurement_name: String,
    pub ks_statistic: f64,
    pub p_value: f64,
    /// Whether the null hypothesis is rejected at alpha = 0.05.
    pub reject_null: bool,
    pub sample_size: usize,
}

/// Full validation report for one reference dataset comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub dataset_name: String,
    pub ks_results: Vec<KsTestResult>,
    pub passing_measurements: usize,
    pub total_measurements: usize,
    pub overall_pass: bool,
    pub summary_statistics: Vec<SummaryStat>,
}

/// Per-measurement comparison of generated vs reference statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStat {
    pub name: String,
    pub generated_mean: f64,
    pub generated_std: f64,
    pub reference_mean: f64,
    pub reference_std: f64,
    pub mean_error_percent: f64,
    pub std_error_percent: f64,
}

// ---------------------------------------------------------------------------
// Normal CDF — Abramowitz & Stegun 26.2.17
// ---------------------------------------------------------------------------

/// Standard normal PDF: phi(x).
fn normal_pdf(x: f64) -> f64 {
    const INV_SQRT_2PI: f64 = 0.398_942_280_401_432_7; // 1/sqrt(2*pi)
    INV_SQRT_2PI * (-0.5 * x * x).exp()
}

/// Standard normal CDF: Phi(x).
///
/// Uses the Abramowitz & Stegun polynomial approximation (formula 26.2.17)
/// which has maximum error < 7.5e-8.
pub fn normal_cdf(x: f64) -> f64 {
    if x < -8.0 {
        return 0.0;
    }
    if x > 8.0 {
        return 1.0;
    }

    let abs_x = x.abs();
    let t = 1.0 / (1.0 + 0.231_641_9 * abs_x);

    const B1: f64 = 0.319_381_530;
    const B2: f64 = -0.356_563_782;
    const B3: f64 = 1.781_477_937;
    const B4: f64 = -1.821_255_978;
    const B5: f64 = 1.330_274_429;

    let poly = t * (B1 + t * (B2 + t * (B3 + t * (B4 + t * B5))));
    let cdf_positive = 1.0 - normal_pdf(abs_x) * poly;

    if x >= 0.0 {
        cdf_positive
    } else {
        1.0 - cdf_positive
    }
}

/// CDF of a normal distribution with given mean and std_dev.
fn normal_cdf_params(x: f64, mean: f64, std_dev: f64) -> f64 {
    if std_dev <= 0.0 {
        if x >= mean {
            1.0
        } else {
            0.0
        }
    } else {
        normal_cdf((x - mean) / std_dev)
    }
}

// ---------------------------------------------------------------------------
// Empirical CDF
// ---------------------------------------------------------------------------

/// Compute the empirical CDF value F_n(x) for sorted data.
fn empirical_cdf(sorted_data: &[f64], x: f64) -> f64 {
    if sorted_data.is_empty() {
        return 0.0;
    }
    // Binary search: number of elements <= x
    let count = match sorted_data.binary_search_by(|v| v.partial_cmp(&x).unwrap_or(core::cmp::Ordering::Equal)) {
        Ok(mut idx) => {
            // Move past all equal elements
            while idx + 1 < sorted_data.len()
                && sorted_data[idx + 1]
                    .partial_cmp(&x)
                    .unwrap_or(core::cmp::Ordering::Equal)
                    == core::cmp::Ordering::Equal
            {
                idx += 1;
            }
            idx + 1
        }
        Err(idx) => idx,
    };
    count as f64 / sorted_data.len() as f64
}

// ---------------------------------------------------------------------------
// KS distribution p-value approximation
// ---------------------------------------------------------------------------

/// Approximate the survival function P(D_n > d) for the KS statistic.
///
/// Uses the asymptotic formula:
///   P(D_n > d) ≈ 2 * sum_{k=1..inf} (-1)^{k+1} exp(-2 k^2 lambda^2)
/// where lambda = (sqrt(n) + 0.12 + 0.11/sqrt(n)) * d.
fn ks_survival(n: usize, d: f64) -> f64 {
    if n == 0 || d <= 0.0 {
        return 1.0;
    }
    if d >= 1.0 {
        return 0.0;
    }
    let sqrt_n = (n as f64).sqrt();
    let lambda = (sqrt_n + 0.12 + 0.11 / sqrt_n) * d;
    let lambda_sq = lambda * lambda;

    let mut sum = 0.0;
    for k in 1..=100 {
        let kf = k as f64;
        let term = (-2.0 * kf * kf * lambda_sq).exp();
        if term < 1e-15 {
            break;
        }
        if k % 2 == 1 {
            sum += term;
        } else {
            sum -= term;
        }
    }
    (2.0 * sum).clamp(0.0, 1.0)
}

/// Approximate KS survival for two-sample test with sizes n1, n2.
fn ks_survival_two_sample(n1: usize, n2: usize, d: f64) -> f64 {
    if n1 == 0 || n2 == 0 || d <= 0.0 {
        return 1.0;
    }
    if d >= 1.0 {
        return 0.0;
    }
    let effective_n = (n1 as f64 * n2 as f64) / (n1 as f64 + n2 as f64);
    let sqrt_n = effective_n.sqrt();
    let lambda = (sqrt_n + 0.12 + 0.11 / sqrt_n) * d;
    let lambda_sq = lambda * lambda;

    let mut sum = 0.0;
    for k in 1..=100 {
        let kf = k as f64;
        let term = (-2.0 * kf * kf * lambda_sq).exp();
        if term < 1e-15 {
            break;
        }
        if k % 2 == 1 {
            sum += term;
        } else {
            sum -= term;
        }
    }
    (2.0 * sum).clamp(0.0, 1.0)
}

/// KS critical value approximation for significance level `alpha`.
///
/// For moderate to large n, the critical value is approximately
///   c(alpha) / sqrt(n)
/// where c(alpha) comes from the asymptotic distribution.
pub fn ks_critical_value(n: usize, alpha: f64) -> f64 {
    if n == 0 {
        return f64::INFINITY;
    }
    // Common critical values for the asymptotic KS distribution
    let c = if alpha <= 0.001 {
        1.949
    } else if alpha <= 0.005 {
        1.731
    } else if alpha <= 0.01 {
        1.628
    } else if alpha <= 0.02 {
        1.517
    } else if alpha <= 0.05 {
        1.358
    } else if alpha <= 0.10 {
        1.224
    } else if alpha <= 0.20 {
        1.073
    } else {
        // For larger alpha, use a rough interpolation
        0.95
    };
    c / (n as f64).sqrt()
}

// ---------------------------------------------------------------------------
// Sorting helper
// ---------------------------------------------------------------------------

fn sorted_copy(data: &[f64]) -> Vec<f64> {
    let mut v = data.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    v
}

// ---------------------------------------------------------------------------
// Summary statistics helpers
// ---------------------------------------------------------------------------

fn compute_mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

fn compute_std_dev(data: &[f64], mean: f64) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
    variance.sqrt()
}

fn percent_error(generated: f64, reference: f64) -> f64 {
    if reference.abs() < 1e-12 {
        if generated.abs() < 1e-12 {
            0.0
        } else {
            100.0
        }
    } else {
        ((generated - reference) / reference * 100.0).abs()
    }
}

// ---------------------------------------------------------------------------
// PopulationValidator
// ---------------------------------------------------------------------------

impl Default for PopulationValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl PopulationValidator {
    /// Create an empty validator with no reference datasets.
    pub fn new() -> Self {
        Self {
            reference_data: Vec::new(),
        }
    }

    /// Create a validator pre-loaded with NHANES reference data.
    pub fn with_nhanes_reference() -> Self {
        let mut v = Self::new();
        v.reference_data.push(nhanes_reference_dataset());
        v
    }

    /// Create a validator pre-loaded with ANSUR reference data.
    pub fn with_ansur_reference() -> Self {
        let mut v = Self::new();
        v.reference_data.push(ansur_reference_dataset());
        v
    }

    /// Create a validator with both NHANES and ANSUR reference data.
    pub fn with_all_references() -> Self {
        let mut v = Self::new();
        v.reference_data.push(nhanes_reference_dataset());
        v.reference_data.push(ansur_reference_dataset());
        v
    }

    /// Add a custom reference dataset.
    pub fn add_reference(&mut self, dataset: ReferenceDataset) {
        self.reference_data.push(dataset);
    }

    /// Return all loaded reference datasets.
    pub fn reference_datasets(&self) -> &[ReferenceDataset] {
        &self.reference_data
    }

    /// Validate a set of generated bodies against a named reference dataset.
    ///
    /// `generated_measurements` is a slice of bodies, each body being a list
    /// of (measurement_name, value) pairs.
    ///
    /// Returns a `ValidationReport` comparing generated samples against the
    /// reference distribution using KS tests.
    pub fn validate(
        &self,
        generated_measurements: &[Vec<(String, f64)>],
        dataset_name: &str,
    ) -> anyhow::Result<ValidationReport> {
        let dataset = self
            .reference_data
            .iter()
            .find(|d| d.name == dataset_name)
            .ok_or_else(|| anyhow::anyhow!("Reference dataset '{}' not found", dataset_name))?;

        let mut ks_results = Vec::new();
        let mut summary_statistics = Vec::new();

        for ref_meas in &dataset.measurements {
            // Collect matching generated values
            let values: Vec<f64> = generated_measurements
                .iter()
                .filter_map(|body| {
                    body.iter()
                        .find(|(n, _)| *n == ref_meas.name)
                        .map(|(_, v)| *v)
                })
                .collect();

            if values.is_empty() {
                continue;
            }

            // KS test against normal reference distribution
            let ks = Self::ks_test_normal(&values, ref_meas.mean, ref_meas.std_dev);
            let ks_result = KsTestResult {
                measurement_name: ref_meas.name.clone(),
                ks_statistic: ks.ks_statistic,
                p_value: ks.p_value,
                reject_null: ks.reject_null,
                sample_size: ks.sample_size,
            };

            // Summary statistics
            let gen_mean = compute_mean(&values);
            let gen_std = compute_std_dev(&values, gen_mean);
            summary_statistics.push(SummaryStat {
                name: ref_meas.name.clone(),
                generated_mean: gen_mean,
                generated_std: gen_std,
                reference_mean: ref_meas.mean,
                reference_std: ref_meas.std_dev,
                mean_error_percent: percent_error(gen_mean, ref_meas.mean),
                std_error_percent: percent_error(gen_std, ref_meas.std_dev),
            });

            ks_results.push(ks_result);
        }

        let passing = ks_results.iter().filter(|r| !r.reject_null).count();
        let total = ks_results.len();
        // Overall pass if at least 80% of measurements pass (to allow some noise)
        let overall_pass = if total == 0 {
            true
        } else {
            passing as f64 / total as f64 >= 0.80
        };

        Ok(ValidationReport {
            dataset_name: dataset_name.to_string(),
            ks_results,
            passing_measurements: passing,
            total_measurements: total,
            overall_pass,
            summary_statistics,
        })
    }

    /// Two-sample Kolmogorov-Smirnov test.
    ///
    /// Tests whether `sample_a` and `sample_b` are drawn from the same
    /// distribution. Returns the KS statistic, p-value, and whether the null
    /// hypothesis is rejected at alpha = 0.05.
    pub fn ks_test_two_sample(sample_a: &[f64], sample_b: &[f64]) -> KsTestResult {
        let sorted_a = sorted_copy(sample_a);
        let sorted_b = sorted_copy(sample_b);
        let n1 = sorted_a.len();
        let n2 = sorted_b.len();

        if n1 == 0 || n2 == 0 {
            return KsTestResult {
                measurement_name: String::new(),
                ks_statistic: 0.0,
                p_value: 1.0,
                reject_null: false,
                sample_size: 0,
            };
        }

        // Merge all unique data points as evaluation points
        let mut all_points = Vec::with_capacity(n1 + n2);
        all_points.extend_from_slice(&sorted_a);
        all_points.extend_from_slice(&sorted_b);
        all_points.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        all_points.dedup();

        let mut max_diff: f64 = 0.0;
        for &x in &all_points {
            let f1 = empirical_cdf(&sorted_a, x);
            let f2 = empirical_cdf(&sorted_b, x);
            let diff = (f1 - f2).abs();
            if diff > max_diff {
                max_diff = diff;
            }
        }

        let p_value = ks_survival_two_sample(n1, n2, max_diff);

        KsTestResult {
            measurement_name: String::new(),
            ks_statistic: max_diff,
            p_value,
            reject_null: p_value < 0.05,
            sample_size: n1.min(n2),
        }
    }

    /// One-sample KS test against a normal distribution with given parameters.
    pub fn ks_test_normal(sample: &[f64], mean: f64, std_dev: f64) -> KsTestResult {
        let sorted = sorted_copy(sample);
        let n = sorted.len();

        if n == 0 {
            return KsTestResult {
                measurement_name: String::new(),
                ks_statistic: 0.0,
                p_value: 1.0,
                reject_null: false,
                sample_size: 0,
            };
        }

        // Compute the KS statistic: max |F_n(x) - F(x)|
        // We check at each data point x_i:
        //   D+ = max_i { i/n - F(x_i) }
        //   D- = max_i { F(x_i) - (i-1)/n }
        //   D  = max(D+, D-)
        let mut d_plus: f64 = 0.0;
        let mut d_minus: f64 = 0.0;

        for (i, &x) in sorted.iter().enumerate() {
            let f_x = normal_cdf_params(x, mean, std_dev);
            let ecdf_at = (i + 1) as f64 / n as f64;
            let ecdf_below = i as f64 / n as f64;

            let dp = ecdf_at - f_x;
            let dm = f_x - ecdf_below;

            if dp > d_plus {
                d_plus = dp;
            }
            if dm > d_minus {
                d_minus = dm;
            }
        }

        let ks_stat = d_plus.max(d_minus);
        let p_value = ks_survival(n, ks_stat);

        KsTestResult {
            measurement_name: String::new(),
            ks_statistic: ks_stat,
            p_value,
            reject_null: p_value < 0.05,
            sample_size: n,
        }
    }

    /// Generate synthetic samples from a reference dataset's normal distributions.
    ///
    /// Useful for testing: generates `count` bodies using Box-Muller transform
    /// from the reference means and standard deviations.
    pub fn generate_from_reference(
        &self,
        dataset_name: &str,
        count: usize,
        seed: u64,
    ) -> anyhow::Result<Vec<Vec<(String, f64)>>> {
        let dataset = self
            .reference_data
            .iter()
            .find(|d| d.name == dataset_name)
            .ok_or_else(|| anyhow::anyhow!("Reference dataset '{}' not found", dataset_name))?;

        let mut state = seed.wrapping_add(1);
        let mut next_u64 = || -> u64 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            state
        };
        let mut next_f64 = || -> f64 { (next_u64() >> 11) as f64 / (1u64 << 53) as f64 };

        // Box-Muller
        let mut next_normal = |mean: f64, std_dev: f64| -> f64 {
            let u1 = next_f64().max(1e-15);
            let u2 = next_f64();
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * core::f64::consts::PI * u2).cos();
            mean + z * std_dev
        };

        let mut bodies = Vec::with_capacity(count);
        for _ in 0..count {
            let mut measurements = Vec::with_capacity(dataset.measurements.len());
            for m in &dataset.measurements {
                let value = next_normal(m.mean, m.std_dev);
                measurements.push((m.name.clone(), value));
            }
            bodies.push(measurements);
        }

        Ok(bodies)
    }
}

// ---------------------------------------------------------------------------
// NHANES reference data (approximate published values, adults 20+)
// ---------------------------------------------------------------------------

fn nhanes_measurement(name: &str, sex: Sex, mean: f64, std_dev: f64, sample_size: usize) -> PopulationMeasurement {
    PopulationMeasurement {
        name: name.to_string(),
        sex,
        mean,
        std_dev,
        percentiles: Percentiles::from_normal(mean, std_dev),
        sample_size,
    }
}

fn nhanes_reference_dataset() -> ReferenceDataset {
    // Values approximated from NHANES 2015-2018 published tables (adults 20+)
    let measurements = vec![
        // --- Stature (cm) ---
        nhanes_measurement("stature_cm", Sex::Male, 175.7, 7.5, 5000),
        nhanes_measurement("stature_cm", Sex::Female, 162.1, 6.6, 5200),
        // --- Weight (kg) ---
        nhanes_measurement("weight_kg", Sex::Male, 88.8, 19.4, 5000),
        nhanes_measurement("weight_kg", Sex::Female, 77.4, 20.7, 5200),
        // --- BMI (kg/m^2) ---
        nhanes_measurement("bmi", Sex::Male, 28.8, 5.9, 5000),
        nhanes_measurement("bmi", Sex::Female, 29.1, 7.1, 5200),
        // --- Waist circumference (cm) ---
        nhanes_measurement("waist_circumference_cm", Sex::Male, 99.1, 15.1, 4800),
        nhanes_measurement("waist_circumference_cm", Sex::Female, 94.4, 16.0, 5000),
        // --- Hip circumference (cm) ---
        nhanes_measurement("hip_circumference_cm", Sex::Male, 105.3, 10.2, 4800),
        nhanes_measurement("hip_circumference_cm", Sex::Female, 108.5, 13.8, 5000),
        // --- Upper arm length (cm) ---
        nhanes_measurement("upper_arm_length_cm", Sex::Male, 37.0, 2.1, 4500),
        nhanes_measurement("upper_arm_length_cm", Sex::Female, 33.9, 2.0, 4700),
        // --- Upper arm circumference (cm) ---
        nhanes_measurement("upper_arm_circumference_cm", Sex::Male, 33.6, 4.2, 4500),
        nhanes_measurement("upper_arm_circumference_cm", Sex::Female, 31.3, 5.0, 4700),
        // --- Thigh circumference (cm) ---
        nhanes_measurement("thigh_circumference_cm", Sex::Male, 53.2, 6.1, 4500),
        nhanes_measurement("thigh_circumference_cm", Sex::Female, 54.8, 7.8, 4700),
        // --- Calf circumference (cm) ---
        nhanes_measurement("calf_circumference_cm", Sex::Male, 38.5, 3.7, 4500),
        nhanes_measurement("calf_circumference_cm", Sex::Female, 37.2, 4.3, 4700),
        // --- Waist-to-hip ratio ---
        nhanes_measurement("waist_hip_ratio", Sex::Male, 0.94, 0.07, 4800),
        nhanes_measurement("waist_hip_ratio", Sex::Female, 0.87, 0.08, 5000),
        // --- Head circumference (cm) ---
        nhanes_measurement("head_circumference_cm", Sex::Male, 57.0, 1.7, 3000),
        nhanes_measurement("head_circumference_cm", Sex::Female, 55.0, 1.6, 3100),
    ];

    ReferenceDataset {
        name: "NHANES".to_string(),
        source: DataSource::Nhanes,
        measurements,
    }
}

// ---------------------------------------------------------------------------
// ANSUR reference data (approximate published values, US Army personnel)
// ---------------------------------------------------------------------------

fn ansur_reference_dataset() -> ReferenceDataset {
    // Values approximated from ANSUR II (2012) published tables
    let measurements = vec![
        // --- Stature (cm) ---
        nhanes_measurement("stature_cm", Sex::Male, 175.6, 6.7, 4082),
        nhanes_measurement("stature_cm", Sex::Female, 163.0, 6.3, 1986),
        // --- Weight (kg) ---
        nhanes_measurement("weight_kg", Sex::Male, 84.3, 14.7, 4082),
        nhanes_measurement("weight_kg", Sex::Female, 68.4, 12.0, 1986),
        // --- Chest circumference (cm) ---
        nhanes_measurement("chest_circumference_cm", Sex::Male, 102.1, 8.4, 4082),
        nhanes_measurement("chest_circumference_cm", Sex::Female, 93.7, 8.0, 1986),
        // --- Shoulder breadth (biacromial, cm) ---
        nhanes_measurement("shoulder_breadth_cm", Sex::Male, 46.1, 2.6, 4082),
        nhanes_measurement("shoulder_breadth_cm", Sex::Female, 40.4, 2.3, 1986),
        // --- Hip breadth, sitting (cm) ---
        nhanes_measurement("hip_breadth_sitting_cm", Sex::Male, 37.3, 2.8, 4082),
        nhanes_measurement("hip_breadth_sitting_cm", Sex::Female, 39.3, 3.4, 1986),
        // --- Waist circumference (cm) ---
        nhanes_measurement("waist_circumference_cm", Sex::Male, 87.4, 10.7, 4082),
        nhanes_measurement("waist_circumference_cm", Sex::Female, 76.8, 9.5, 1986),
        // --- Arm length (acromion-radiale, cm) ---
        nhanes_measurement("arm_length_cm", Sex::Male, 33.6, 1.8, 4082),
        nhanes_measurement("arm_length_cm", Sex::Female, 30.5, 1.7, 1986),
        // --- Forearm length (radiale-stylion, cm) ---
        nhanes_measurement("forearm_length_cm", Sex::Male, 27.1, 1.5, 4082),
        nhanes_measurement("forearm_length_cm", Sex::Female, 24.2, 1.3, 1986),
        // --- Total arm length (cm) ---
        nhanes_measurement("total_arm_length_cm", Sex::Male, 78.5, 3.7, 4082),
        nhanes_measurement("total_arm_length_cm", Sex::Female, 70.3, 3.3, 1986),
        // --- Inseam / crotch height (cm) ---
        nhanes_measurement("inseam_cm", Sex::Male, 84.1, 4.6, 4082),
        nhanes_measurement("inseam_cm", Sex::Female, 77.2, 4.2, 1986),
        // --- Sitting height (cm) ---
        nhanes_measurement("sitting_height_cm", Sex::Male, 91.5, 3.5, 4082),
        nhanes_measurement("sitting_height_cm", Sex::Female, 85.8, 3.2, 1986),
        // --- Head circumference (cm) ---
        nhanes_measurement("head_circumference_cm", Sex::Male, 57.0, 1.5, 4082),
        nhanes_measurement("head_circumference_cm", Sex::Female, 55.2, 1.4, 1986),
        // --- Neck circumference (cm) ---
        nhanes_measurement("neck_circumference_cm", Sex::Male, 39.5, 2.3, 4082),
        nhanes_measurement("neck_circumference_cm", Sex::Female, 33.8, 1.8, 1986),
        // --- Thigh circumference (cm) ---
        nhanes_measurement("thigh_circumference_cm", Sex::Male, 57.8, 5.0, 4082),
        nhanes_measurement("thigh_circumference_cm", Sex::Female, 56.0, 5.4, 1986),
        // --- Calf circumference (cm) ---
        nhanes_measurement("calf_circumference_cm", Sex::Male, 38.7, 2.8, 4082),
        nhanes_measurement("calf_circumference_cm", Sex::Female, 36.1, 2.8, 1986),
        // --- Foot length (cm) ---
        nhanes_measurement("foot_length_cm", Sex::Male, 27.2, 1.3, 4082),
        nhanes_measurement("foot_length_cm", Sex::Female, 24.5, 1.2, 1986),
        // --- Hand length (cm) ---
        nhanes_measurement("hand_length_cm", Sex::Male, 19.4, 1.0, 4082),
        nhanes_measurement("hand_length_cm", Sex::Female, 17.7, 0.9, 1986),
        // --- Bicep circumference flexed (cm) ---
        nhanes_measurement("bicep_circumference_flexed_cm", Sex::Male, 34.6, 3.4, 4082),
        nhanes_measurement("bicep_circumference_flexed_cm", Sex::Female, 29.0, 3.4, 1986),
        // --- Knee height (cm) ---
        nhanes_measurement("knee_height_cm", Sex::Male, 55.5, 2.8, 4082),
        nhanes_measurement("knee_height_cm", Sex::Female, 50.7, 2.5, 1986),
    ];

    ReferenceDataset {
        name: "ANSUR".to_string(),
        source: DataSource::Ansur,
        measurements,
    }
}

// ---------------------------------------------------------------------------
// Display implementations
// ---------------------------------------------------------------------------

impl core::fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "=== Validation Report: {} ===", self.dataset_name)?;
        writeln!(
            f,
            "Overall: {} ({}/{} measurements passing KS test at alpha=0.05)",
            if self.overall_pass { "PASS" } else { "FAIL" },
            self.passing_measurements,
            self.total_measurements,
        )?;
        writeln!(f)?;

        for ks in &self.ks_results {
            writeln!(
                f,
                "  {}: D={:.4}, p={:.4} [{}] (n={})",
                ks.measurement_name,
                ks.ks_statistic,
                ks.p_value,
                if ks.reject_null { "FAIL" } else { "PASS" },
                ks.sample_size,
            )?;
        }
        writeln!(f)?;

        if !self.summary_statistics.is_empty() {
            writeln!(f, "Summary statistics:")?;
            for s in &self.summary_statistics {
                writeln!(
                    f,
                    "  {}: gen_mean={:.2} ref_mean={:.2} (err={:.1}%), gen_std={:.2} ref_std={:.2} (err={:.1}%)",
                    s.name,
                    s.generated_mean,
                    s.reference_mean,
                    s.mean_error_percent,
                    s.generated_std,
                    s.reference_std,
                    s.std_error_percent,
                )?;
            }
        }

        Ok(())
    }
}

impl core::fmt::Display for KsTestResult {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "KS({}, n={}): D={:.4}, p={:.4} [{}]",
            self.measurement_name,
            self.sample_size,
            self.ks_statistic,
            self.p_value,
            if self.reject_null {
                "reject H0"
            } else {
                "fail to reject H0"
            },
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_cdf_symmetry() {
        let mid = normal_cdf(0.0);
        assert!((mid - 0.5).abs() < 1e-6, "Phi(0) should be 0.5, got {mid}");

        let low = normal_cdf(-3.0);
        let high = normal_cdf(3.0);
        assert!(
            (low + high - 1.0).abs() < 1e-6,
            "Phi(-3) + Phi(3) should be 1.0"
        );
    }

    #[test]
    fn test_normal_cdf_known_values() {
        // Phi(1.96) ≈ 0.975
        let v = normal_cdf(1.96);
        assert!(
            (v - 0.975).abs() < 0.001,
            "Phi(1.96) should be ~0.975, got {v}"
        );

        // Phi(-1.96) ≈ 0.025
        let v2 = normal_cdf(-1.96);
        assert!(
            (v2 - 0.025).abs() < 0.001,
            "Phi(-1.96) should be ~0.025, got {v2}"
        );
    }

    #[test]
    fn test_empirical_cdf() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((empirical_cdf(&data, 0.0) - 0.0).abs() < 1e-10);
        assert!((empirical_cdf(&data, 1.0) - 0.2).abs() < 1e-10);
        assert!((empirical_cdf(&data, 3.0) - 0.6).abs() < 1e-10);
        assert!((empirical_cdf(&data, 5.0) - 1.0).abs() < 1e-10);
        assert!((empirical_cdf(&data, 6.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ks_test_normal_pass() {
        // Generate data from N(0, 1) — should not reject
        let n = 500;
        let mut data = Vec::with_capacity(n);
        let mut state: u64 = 42_u64.wrapping_add(1);
        for _ in 0..n {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let u1 = ((state >> 11) as f64 / (1u64 << 53) as f64).max(1e-15);
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let u2 = (state >> 11) as f64 / (1u64 << 53) as f64;
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * core::f64::consts::PI * u2).cos();
            data.push(z);
        }

        let result = PopulationValidator::ks_test_normal(&data, 0.0, 1.0);
        assert!(
            !result.reject_null,
            "Should not reject N(0,1) data vs N(0,1): D={}, p={}",
            result.ks_statistic,
            result.p_value,
        );
    }

    #[test]
    fn test_ks_test_normal_reject() {
        // Uniform data should be rejected against N(0, 1)
        let n = 200;
        let mut data = Vec::with_capacity(n);
        for i in 0..n {
            data.push(-3.0 + 6.0 * (i as f64) / (n as f64));
        }

        let result = PopulationValidator::ks_test_normal(&data, 0.0, 1.0);
        assert!(
            result.reject_null,
            "Uniform data should be rejected vs N(0,1): D={}, p={}",
            result.ks_statistic,
            result.p_value,
        );
    }

    #[test]
    fn test_ks_test_two_sample_same() {
        let a: Vec<f64> = (0..100).map(|i| i as f64 * 0.01).collect();
        let b: Vec<f64> = (0..100).map(|i| i as f64 * 0.01 + 0.005).collect();

        let result = PopulationValidator::ks_test_two_sample(&a, &b);
        assert!(
            result.ks_statistic < 0.2,
            "Similar samples should have small D: {}",
            result.ks_statistic,
        );
    }

    #[test]
    fn test_ks_test_two_sample_different() {
        let a: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let b: Vec<f64> = (0..100).map(|i| (i as f64) + 200.0).collect();

        let result = PopulationValidator::ks_test_two_sample(&a, &b);
        assert!(
            result.reject_null,
            "Very different samples should be rejected: D={}, p={}",
            result.ks_statistic,
            result.p_value,
        );
    }

    #[test]
    fn test_ks_critical_value() {
        let cv = ks_critical_value(100, 0.05);
        // For n=100, alpha=0.05: c ≈ 1.358/sqrt(100) = 0.1358
        assert!(
            (cv - 0.1358).abs() < 0.01,
            "Critical value for n=100, alpha=0.05 should be ~0.1358, got {cv}"
        );
    }

    #[test]
    fn test_nhanes_reference_loaded() {
        let v = PopulationValidator::with_nhanes_reference();
        let datasets = v.reference_datasets();
        assert_eq!(datasets.len(), 1);
        assert_eq!(datasets[0].name, "NHANES");
        assert!(!datasets[0].measurements.is_empty());
    }

    #[test]
    fn test_ansur_reference_loaded() {
        let v = PopulationValidator::with_ansur_reference();
        let datasets = v.reference_datasets();
        assert_eq!(datasets.len(), 1);
        assert_eq!(datasets[0].name, "ANSUR");
        assert!(!datasets[0].measurements.is_empty());
    }

    #[test]
    fn test_validate_synthetic_nhanes() {
        let v = PopulationValidator::with_nhanes_reference();
        let bodies = v
            .generate_from_reference("NHANES", 1000, 123)
            .expect("generate should succeed");
        let report = v.validate(&bodies, "NHANES").expect("validate should succeed");

        assert!(
            report.overall_pass,
            "Synthetic data from same distribution should pass: {}/{} passing",
            report.passing_measurements,
            report.total_measurements,
        );
    }

    #[test]
    fn test_validate_synthetic_ansur() {
        let v = PopulationValidator::with_ansur_reference();
        let bodies = v
            .generate_from_reference("ANSUR", 1000, 456)
            .expect("generate should succeed");
        let report = v.validate(&bodies, "ANSUR").expect("validate should succeed");

        assert!(
            report.overall_pass,
            "Synthetic ANSUR data should pass: {}/{} passing",
            report.passing_measurements,
            report.total_measurements,
        );
    }

    #[test]
    fn test_validate_wrong_dataset_name() {
        let v = PopulationValidator::new();
        let bodies: Vec<Vec<(String, f64)>> = vec![vec![("stature_cm".to_string(), 175.0)]];
        let result = v.validate(&bodies, "NONEXISTENT");
        assert!(result.is_err());
    }

    #[test]
    fn test_percentiles_from_normal() {
        let p = Percentiles::from_normal(100.0, 10.0);
        // p50 should be the mean
        assert!((p.p50 - 100.0).abs() < 1e-10);
        // p5 < p25 < p50 < p75 < p95
        assert!(p.p5 < p.p25);
        assert!(p.p25 < p.p50);
        assert!(p.p50 < p.p75);
        assert!(p.p75 < p.p95);
    }

    #[test]
    fn test_percent_error() {
        assert!((percent_error(105.0, 100.0) - 5.0).abs() < 1e-10);
        assert!((percent_error(95.0, 100.0) - 5.0).abs() < 1e-10);
        assert!((percent_error(0.0, 0.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_validation_report_display() {
        let report = ValidationReport {
            dataset_name: "Test".to_string(),
            ks_results: vec![KsTestResult {
                measurement_name: "stature_cm".to_string(),
                ks_statistic: 0.03,
                p_value: 0.85,
                reject_null: false,
                sample_size: 100,
            }],
            passing_measurements: 1,
            total_measurements: 1,
            overall_pass: true,
            summary_statistics: vec![SummaryStat {
                name: "stature_cm".to_string(),
                generated_mean: 175.5,
                generated_std: 7.4,
                reference_mean: 175.7,
                reference_std: 7.5,
                mean_error_percent: 0.11,
                std_error_percent: 1.33,
            }],
        };
        let s = format!("{report}");
        assert!(s.contains("PASS"));
        assert!(s.contains("stature_cm"));
    }

    #[test]
    fn test_empty_samples() {
        let result = PopulationValidator::ks_test_normal(&[], 0.0, 1.0);
        assert!(!result.reject_null);
        assert_eq!(result.sample_size, 0);

        let result2 = PopulationValidator::ks_test_two_sample(&[], &[1.0, 2.0]);
        assert!(!result2.reject_null);
        assert_eq!(result2.sample_size, 0);
    }

    #[test]
    fn test_custom_reference() {
        let mut v = PopulationValidator::new();
        v.add_reference(ReferenceDataset {
            name: "Custom".to_string(),
            source: DataSource::Custom,
            measurements: vec![nhanes_measurement("height_cm", Sex::Male, 170.0, 7.0, 1000)],
        });
        assert_eq!(v.reference_datasets().len(), 1);
        assert_eq!(v.reference_datasets()[0].source, DataSource::Custom);
    }

    #[test]
    fn test_all_references() {
        let v = PopulationValidator::with_all_references();
        assert_eq!(v.reference_datasets().len(), 2);
    }
}
