#![allow(dead_code)]

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphDiffApply {
    diffs: HashMap<String, f32>,
}

#[allow(dead_code)]
pub fn compute_morph_diff(before: &[(String, f32)], after: &[(String, f32)]) -> MorphDiffApply {
    let mut diffs = HashMap::new();
    let before_map: HashMap<&str, f32> = before.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    for (k, v) in after {
        let prev = before_map.get(k.as_str()).copied().unwrap_or(0.0);
        let d = v - prev;
        if d.abs() > 1e-7 { diffs.insert(k.clone(), d); }
    }
    MorphDiffApply { diffs }
}

#[allow(dead_code)]
pub fn apply_morph_diff(diff: &MorphDiffApply, params: &mut Vec<(String, f32)>) {
    for (k, &d) in &diff.diffs {
        if let Some(entry) = params.iter_mut().find(|(n, _)| n == k) {
            entry.1 += d;
        } else {
            params.push((k.clone(), d));
        }
    }
}

#[allow(dead_code)]
pub fn diff_param_count_mda(diff: &MorphDiffApply) -> usize { diff.diffs.len() }

#[allow(dead_code)]
pub fn diff_magnitude(diff: &MorphDiffApply) -> f32 {
    diff.diffs.values().map(|v| v * v).sum::<f32>().sqrt()
}

#[allow(dead_code)]
pub fn diff_to_json_mda(diff: &MorphDiffApply) -> String {
    format!("{{\"count\":{},\"magnitude\":{:.4}}}", diff.diffs.len(), diff_magnitude(diff))
}

#[allow(dead_code)]
pub fn diff_is_zero_mda(diff: &MorphDiffApply) -> bool { diff.diffs.is_empty() }

#[allow(dead_code)]
pub fn diff_invert(diff: &MorphDiffApply) -> MorphDiffApply {
    MorphDiffApply { diffs: diff.diffs.iter().map(|(k, v)| (k.clone(), -v)).collect() }
}

#[allow(dead_code)]
pub fn diff_combine(a: &MorphDiffApply, b: &MorphDiffApply) -> MorphDiffApply {
    let mut diffs = a.diffs.clone();
    for (k, v) in &b.diffs {
        *diffs.entry(k.clone()).or_insert(0.0) += v;
    }
    diffs.retain(|_, v| v.abs() > 1e-7);
    MorphDiffApply { diffs }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_compute_empty() { let d = compute_morph_diff(&[], &[]); assert!(diff_is_zero_mda(&d)); }
    #[test] fn test_compute() { let b = vec![("a".into(), 0.0)]; let a = vec![("a".into(), 1.0)]; let d = compute_morph_diff(&b, &a); assert_eq!(diff_param_count_mda(&d), 1); }
    #[test] fn test_apply() { let b = vec![("x".into(), 0.0)]; let a = vec![("x".into(), 0.5)]; let d = compute_morph_diff(&b, &a); let mut p = vec![("x".into(), 0.0)]; apply_morph_diff(&d, &mut p); assert!((p[0].1 - 0.5).abs() < 1e-6); }
    #[test] fn test_magnitude() { let b = vec![("a".into(), 0.0)]; let a = vec![("a".into(), 3.0)]; let d = compute_morph_diff(&b, &a); assert!((diff_magnitude(&d) - 3.0).abs() < 1e-4); }
    #[test] fn test_json() { let d = compute_morph_diff(&[], &[]); assert!(diff_to_json_mda(&d).contains("count")); }
    #[test] fn test_is_zero() { let d = compute_morph_diff(&[], &[]); assert!(diff_is_zero_mda(&d)); }
    #[test] fn test_invert() { let b = vec![("a".into(), 0.0)]; let a = vec![("a".into(), 1.0)]; let d = compute_morph_diff(&b, &a); let inv = diff_invert(&d); assert!((inv.diffs["a"] - (-1.0)).abs() < 1e-6); }
    #[test] fn test_combine() { let b = vec![("a".into(), 0.0)]; let a1 = vec![("a".into(), 1.0)]; let a2 = vec![("a".into(), 2.0)]; let d1 = compute_morph_diff(&b, &a1); let d2 = compute_morph_diff(&b, &a2); let c = diff_combine(&d1, &d2); assert!((c.diffs["a"] - 3.0).abs() < 1e-6); }
    #[test] fn test_apply_new_param() { let d = MorphDiffApply { diffs: [("new".into(), 0.5)].into() }; let mut p: Vec<(String, f32)> = Vec::new(); apply_morph_diff(&d, &mut p); assert_eq!(p.len(), 1); }
    #[test] fn test_count() { let d = compute_morph_diff(&[("a".into(), 0.0)], &[("a".into(), 1.0)]); assert_eq!(diff_param_count_mda(&d), 1); }
}
