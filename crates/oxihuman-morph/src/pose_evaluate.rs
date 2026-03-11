#![allow(dead_code)]

/// Pose evaluation utilities.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseEvaluate {
    pub weights: Vec<f32>,
    pub names: Vec<String>,
}

#[allow(dead_code)]
pub fn evaluate_pose(weights: &[f32]) -> Vec<f32> { weights.to_vec() }

#[allow(dead_code)]
pub fn pose_param_count_pe(pe: &PoseEvaluate) -> usize { pe.weights.len() }

#[allow(dead_code)]
pub fn pose_weight_at(pe: &PoseEvaluate, idx: usize) -> f32 {
    pe.weights.get(idx).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn pose_is_neutral(pe: &PoseEvaluate) -> bool {
    pe.weights.iter().all(|w| w.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn pose_to_params(pe: &PoseEvaluate) -> Vec<(String, f32)> {
    pe.names.iter().zip(pe.weights.iter()).map(|(n, w)| (n.clone(), *w)).collect()
}

#[allow(dead_code)]
pub fn pose_blend_two(a: &PoseEvaluate, b: &PoseEvaluate, t: f32) -> Vec<f32> {
    let len = a.weights.len().max(b.weights.len());
    (0..len).map(|i| {
        let va = a.weights.get(i).copied().unwrap_or(0.0);
        let vb = b.weights.get(i).copied().unwrap_or(0.0);
        va + (vb - va) * t
    }).collect()
}

#[allow(dead_code)]
pub fn pose_to_json_pe(pe: &PoseEvaluate) -> String {
    let ws: Vec<String> = pe.weights.iter().map(|w| format!("{:.4}", w)).collect();
    format!("{{\"count\":{},\"weights\":[{}]}}", pe.weights.len(), ws.join(","))
}

#[allow(dead_code)]
pub fn pose_normalize(pe: &mut PoseEvaluate) {
    let sum: f32 = pe.weights.iter().map(|w| w.abs()).sum();
    if sum > 1e-9 {
        for w in pe.weights.iter_mut() { *w /= sum; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_pe(w: &[f32]) -> PoseEvaluate {
        PoseEvaluate {
            weights: w.to_vec(),
            names: (0..w.len()).map(|i| format!("p{}", i)).collect(),
        }
    }
    #[test] fn test_evaluate() { assert_eq!(evaluate_pose(&[1.0, 2.0]).len(), 2); }
    #[test] fn test_count() { assert_eq!(pose_param_count_pe(&make_pe(&[1.0, 2.0])), 2); }
    #[test] fn test_weight_at() { assert!((pose_weight_at(&make_pe(&[0.5]), 0) - 0.5).abs() < 1e-6); }
    #[test] fn test_weight_at_oob() { assert!((pose_weight_at(&make_pe(&[]), 0)).abs() < 1e-6); }
    #[test] fn test_is_neutral_true() { assert!(pose_is_neutral(&make_pe(&[0.0, 0.0]))); }
    #[test] fn test_is_neutral_false() { assert!(!pose_is_neutral(&make_pe(&[0.1]))); }
    #[test] fn test_to_params() {
        let pe = make_pe(&[0.5]);
        let p = pose_to_params(&pe);
        assert_eq!(p[0].0, "p0");
    }
    #[test] fn test_blend() {
        let a = make_pe(&[0.0]); let b = make_pe(&[1.0]);
        let r = pose_blend_two(&a, &b, 0.5);
        assert!((r[0] - 0.5).abs() < 1e-6);
    }
    #[test] fn test_to_json() { assert!(pose_to_json_pe(&make_pe(&[1.0])).contains("count")); }
    #[test] fn test_normalize() {
        let mut pe = make_pe(&[2.0, 2.0]);
        pose_normalize(&mut pe);
        assert!((pe.weights[0] - 0.5).abs() < 1e-6);
    }
}
