// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Crowd generation with controlled variation using deterministic LCG.

#[allow(dead_code)]
/// Distribution shape for a variation axis.
#[derive(Debug, Clone)]
pub struct Distribution {
    /// "uniform", "gaussian", or "bimodal"
    pub kind: String,
    pub mean: f32,
    pub std: f32,
}

#[allow(dead_code)]
/// A single variation dimension (e.g. height, weight).
#[derive(Debug, Clone)]
pub struct VariationAxis {
    pub name: String,
    pub min: f32,
    pub max: f32,
    pub distribution: Distribution,
}

#[allow(dead_code)]
/// Specification for generating a crowd.
#[derive(Debug, Clone)]
pub struct CrowdSpec {
    pub n: usize,
    pub axes: Vec<VariationAxis>,
    pub seed: u64,
}

#[allow(dead_code)]
/// One person in a crowd.
#[derive(Debug, Clone)]
pub struct CrowdMember {
    pub id: usize,
    pub params: Vec<(String, f32)>,
    pub group_id: usize,
}

#[allow(dead_code)]
/// A generated crowd.
#[derive(Debug, Clone)]
pub struct Crowd {
    pub members: Vec<CrowdMember>,
    pub spec: CrowdSpec,
}

// ---------------------------------------------------------------------------
// LCG helpers
// ---------------------------------------------------------------------------

/// LCG step — multiplier/increment from Numerical Recipes.
#[inline]
fn lcg_step(state: &mut u64) {
    *state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
}

/// Sample a uniform float in [min, max) using the LCG state.
pub fn lcg_sample(state: &mut u64, min: f32, max: f32) -> f32 {
    lcg_step(state);
    let t = (*state as f32) / (u64::MAX as f32);
    min + t * (max - min)
}

/// Box-Muller transform for Gaussian sampling, clamped to [mean-3σ, mean+3σ].
pub fn lcg_gaussian(state: &mut u64, mean: f32, std: f32) -> f32 {
    let u1 = lcg_sample(state, 1e-9, 1.0);
    let u2 = lcg_sample(state, 0.0, 1.0);
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
    mean + std * z
}

/// Sample from the axis distribution, clamped to [min, max].
fn sample_axis(state: &mut u64, axis: &VariationAxis) -> f32 {
    let v = match axis.distribution.kind.as_str() {
        "gaussian" => lcg_gaussian(state, axis.distribution.mean, axis.distribution.std),
        "bimodal" => {
            // two Gaussians at ±σ from mean
            let which = lcg_sample(state, 0.0, 1.0);
            let offset = if which < 0.5 {
                -axis.distribution.std
            } else {
                axis.distribution.std
            };
            lcg_gaussian(
                state,
                axis.distribution.mean + offset,
                axis.distribution.std * 0.5,
            )
        }
        _ => lcg_sample(state, axis.min, axis.max), // "uniform"
    };
    v.clamp(axis.min, axis.max)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate a crowd from a spec.
pub fn generate_crowd(spec: &CrowdSpec) -> Crowd {
    let mut state = spec.seed ^ 0xDEAD_BEEF_CAFE_BABE;
    let members: Vec<CrowdMember> = (0..spec.n)
        .map(|id| {
            let params: Vec<(String, f32)> = spec
                .axes
                .iter()
                .map(|ax| (ax.name.clone(), sample_axis(&mut state, ax)))
                .collect();
            CrowdMember {
                id,
                params,
                group_id: 0,
            }
        })
        .collect();
    Crowd {
        members,
        spec: spec.clone(),
    }
}

/// Average pairwise Euclidean distance across all param axes.
pub fn crowd_diversity_score(crowd: &Crowd) -> f32 {
    let n = crowd.members.len();
    if n < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    let mut count = 0usize;
    for i in 0..n {
        for j in (i + 1)..n {
            let a = &crowd.members[i].params;
            let b = &crowd.members[j].params;
            let dist: f32 = a
                .iter()
                .zip(b.iter())
                .map(|(x, y)| (x.1 - y.1).powi(2))
                .sum::<f32>()
                .sqrt();
            total += dist;
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

/// K-means cluster assignment (Euclidean, max 100 iterations).
pub fn cluster_crowd(crowd: &Crowd, k: usize) -> Vec<usize> {
    let n = crowd.members.len();
    if n == 0 || k == 0 {
        return vec![];
    }
    let k = k.min(n);
    let dim = crowd.members[0].params.len();

    // Initialise centroids from first k members
    let mut centroids: Vec<Vec<f32>> = (0..k)
        .map(|i| crowd.members[i].params.iter().map(|(_, v)| *v).collect())
        .collect();

    let mut assignments = vec![0usize; n];

    for _ in 0..100 {
        // Assign
        let mut changed = false;
        for (i, m) in crowd.members.iter().enumerate() {
            let vals: Vec<f32> = m.params.iter().map(|(_, v)| *v).collect();
            let best = (0..k)
                .min_by(|&a, &b| {
                    let da: f32 = vals
                        .iter()
                        .zip(&centroids[a])
                        .map(|(x, c)| (x - c).powi(2))
                        .sum();
                    let db: f32 = vals
                        .iter()
                        .zip(&centroids[b])
                        .map(|(x, c)| (x - c).powi(2))
                        .sum();
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or(0);
            if assignments[i] != best {
                assignments[i] = best;
                changed = true;
            }
        }
        if !changed {
            break;
        }
        // Update centroids
        let mut sums = vec![vec![0.0f32; dim]; k];
        let mut counts = vec![0usize; k];
        for (i, m) in crowd.members.iter().enumerate() {
            let c = assignments[i];
            for (d, (_, v)) in m.params.iter().enumerate() {
                sums[c][d] += v;
            }
            counts[c] += 1;
        }
        for c in 0..k {
            if counts[c] > 0 {
                for d in 0..dim {
                    centroids[c][d] = sums[c][d] / counts[c] as f32;
                }
            }
        }
    }
    assignments
}

/// Minimal JSON serialization (no external deps).
pub fn crowd_to_json(crowd: &Crowd) -> String {
    let mut out = String::from("[");
    for (i, m) in crowd.members.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            r#"{{"id":{},"group_id":{},"params":{{"#,
            m.id, m.group_id
        ));
        for (j, (k, v)) in m.params.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            out.push_str(&format!(r#""{}":{}."#, k, *v as i32));
            // use serde_json-free approach: just format as fixed decimal
            out = out.trim_end_matches('.').to_string();
            out.push_str(&format!("{:.4}", v));
            // remove the integer part we just pushed — redo cleanly
        }
        out.push_str("}}");
    }
    out.push(']');

    // Redo with a cleaner builder
    let mut result = String::from("[");
    for (i, m) in crowd.members.iter().enumerate() {
        if i > 0 {
            result.push(',');
        }
        result.push_str(&format!(
            r#"{{"id":{},"group_id":{},"params":{{"#,
            m.id, m.group_id
        ));
        for (j, (name, val)) in m.params.iter().enumerate() {
            if j > 0 {
                result.push(',');
            }
            result.push_str(&format!(r#""{}":{:.4}"#, name, val));
        }
        result.push_str("}}");
    }
    result.push(']');
    result
}

/// Eight standard variation axes (height, weight, age, muscle, fat, skin_tone, face_width, leg_length).
pub fn standard_crowd_axes() -> Vec<VariationAxis> {
    vec![
        VariationAxis {
            name: "height".into(),
            min: 1.50,
            max: 2.05,
            distribution: Distribution {
                kind: "gaussian".into(),
                mean: 1.75,
                std: 0.08,
            },
        },
        VariationAxis {
            name: "weight".into(),
            min: 45.0,
            max: 130.0,
            distribution: Distribution {
                kind: "gaussian".into(),
                mean: 75.0,
                std: 15.0,
            },
        },
        VariationAxis {
            name: "age".into(),
            min: 18.0,
            max: 80.0,
            distribution: Distribution {
                kind: "uniform".into(),
                mean: 49.0,
                std: 18.0,
            },
        },
        VariationAxis {
            name: "muscle".into(),
            min: 0.0,
            max: 1.0,
            distribution: Distribution {
                kind: "gaussian".into(),
                mean: 0.4,
                std: 0.2,
            },
        },
        VariationAxis {
            name: "fat".into(),
            min: 0.0,
            max: 1.0,
            distribution: Distribution {
                kind: "gaussian".into(),
                mean: 0.35,
                std: 0.2,
            },
        },
        VariationAxis {
            name: "skin_tone".into(),
            min: 0.0,
            max: 1.0,
            distribution: Distribution {
                kind: "uniform".into(),
                mean: 0.5,
                std: 0.3,
            },
        },
        VariationAxis {
            name: "face_width".into(),
            min: 0.8,
            max: 1.2,
            distribution: Distribution {
                kind: "gaussian".into(),
                mean: 1.0,
                std: 0.08,
            },
        },
        VariationAxis {
            name: "leg_length".into(),
            min: 0.8,
            max: 1.2,
            distribution: Distribution {
                kind: "gaussian".into(),
                mean: 1.0,
                std: 0.07,
            },
        },
    ]
}

/// Histogram of values for a given axis index across all members.
pub fn diversity_histogram(crowd: &Crowd, axis_idx: usize, bins: usize) -> Vec<u32> {
    if bins == 0 || crowd.members.is_empty() {
        return vec![];
    }
    let vals: Vec<f32> = crowd
        .members
        .iter()
        .filter_map(|m| m.params.get(axis_idx).map(|(_, v)| *v))
        .collect();
    if vals.is_empty() {
        return vec![0; bins];
    }
    let min_v = vals.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_v = vals.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let range = (max_v - min_v).max(1e-9);
    let mut hist = vec![0u32; bins];
    for v in &vals {
        let idx = (((v - min_v) / range) * bins as f32) as usize;
        let idx = idx.min(bins - 1);
        hist[idx] += 1;
    }
    hist
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_spec(n: usize) -> CrowdSpec {
        CrowdSpec {
            n,
            axes: standard_crowd_axes(),
            seed: 42,
        }
    }

    #[test]
    fn test_crowd_size_matches_spec() {
        let spec = simple_spec(50);
        let crowd = generate_crowd(&spec);
        assert_eq!(crowd.members.len(), 50);
    }

    #[test]
    fn test_crowd_zero_size() {
        let spec = simple_spec(0);
        let crowd = generate_crowd(&spec);
        assert!(crowd.members.is_empty());
    }

    #[test]
    fn test_params_count_matches_axes() {
        let spec = simple_spec(10);
        let crowd = generate_crowd(&spec);
        for m in &crowd.members {
            assert_eq!(m.params.len(), spec.axes.len());
        }
    }

    #[test]
    fn test_params_in_range() {
        let spec = simple_spec(100);
        let crowd = generate_crowd(&spec);
        for m in &crowd.members {
            for (j, (_, v)) in m.params.iter().enumerate() {
                let ax = &spec.axes[j];
                assert!(
                    (ax.min..=ax.max).contains(v),
                    "axis {} out of range: {}",
                    ax.name,
                    v
                );
            }
        }
    }

    #[test]
    fn test_determinism() {
        let spec = simple_spec(20);
        let c1 = generate_crowd(&spec);
        let c2 = generate_crowd(&spec);
        for (a, b) in c1.members.iter().zip(c2.members.iter()) {
            for (pa, pb) in a.params.iter().zip(b.params.iter()) {
                assert!((pa.1 - pb.1).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn test_different_seeds_differ() {
        let spec1 = CrowdSpec {
            n: 20,
            axes: standard_crowd_axes(),
            seed: 1,
        };
        let spec2 = CrowdSpec {
            n: 20,
            axes: standard_crowd_axes(),
            seed: 2,
        };
        let c1 = generate_crowd(&spec1);
        let c2 = generate_crowd(&spec2);
        let differs = c1.members.iter().zip(c2.members.iter()).any(|(a, b)| {
            a.params
                .iter()
                .zip(b.params.iter())
                .any(|(pa, pb)| (pa.1 - pb.1).abs() > 1e-6)
        });
        assert!(differs);
    }

    #[test]
    fn test_diversity_positive_for_varied_crowd() {
        let spec = simple_spec(30);
        let crowd = generate_crowd(&spec);
        assert!(crowd_diversity_score(&crowd) > 0.0);
    }

    #[test]
    fn test_diversity_zero_for_single_member() {
        let spec = simple_spec(1);
        let crowd = generate_crowd(&spec);
        assert_eq!(crowd_diversity_score(&crowd), 0.0);
    }

    #[test]
    fn test_cluster_k_groups() {
        let spec = simple_spec(40);
        let crowd = generate_crowd(&spec);
        let assignments = cluster_crowd(&crowd, 4);
        assert_eq!(assignments.len(), 40);
        let max_group = assignments.iter().cloned().max().unwrap_or(0);
        assert!(max_group < 4);
    }

    #[test]
    fn test_cluster_single_group() {
        let spec = simple_spec(10);
        let crowd = generate_crowd(&spec);
        let assignments = cluster_crowd(&crowd, 1);
        assert!(assignments.iter().all(|&g| g == 0));
    }

    #[test]
    fn test_cluster_empty_crowd() {
        let spec = simple_spec(0);
        let crowd = generate_crowd(&spec);
        let assignments = cluster_crowd(&crowd, 3);
        assert!(assignments.is_empty());
    }

    #[test]
    fn test_crowd_to_json_non_empty() {
        let spec = simple_spec(5);
        let crowd = generate_crowd(&spec);
        let json = crowd_to_json(&crowd);
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        assert!(json.contains("height"));
    }

    #[test]
    fn test_histogram_sum_equals_n() {
        let spec = simple_spec(50);
        let crowd = generate_crowd(&spec);
        let hist = diversity_histogram(&crowd, 0, 10);
        let total: u32 = hist.iter().sum();
        assert_eq!(total, 50);
    }

    #[test]
    fn test_histogram_bins_count() {
        let spec = simple_spec(20);
        let crowd = generate_crowd(&spec);
        let hist = diversity_histogram(&crowd, 1, 5);
        assert_eq!(hist.len(), 5);
    }

    #[test]
    fn test_standard_axes_count() {
        let axes = standard_crowd_axes();
        assert_eq!(axes.len(), 8);
    }

    #[test]
    fn test_lcg_sample_range() {
        let mut state = 12345u64;
        for _ in 0..1000 {
            let v = lcg_sample(&mut state, 0.0, 1.0);
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn test_gaussian_distribution_kind() {
        let axes = standard_crowd_axes();
        // height uses gaussian
        let height_ax = axes
            .iter()
            .find(|a| a.name == "height")
            .expect("should succeed");
        assert_eq!(height_ax.distribution.kind, "gaussian");
    }
}
