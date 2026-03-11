#![allow(dead_code)]
//! Vertex importance scoring for mesh simplification.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexImportance { scores: Vec<f32> }

#[allow(dead_code)]
pub fn compute_importance(positions: &[[f32; 3]], indices: &[u32]) -> VertexImportance {
    let n = positions.len();
    let mut valence = vec![0u32; n];
    for &idx in indices {
        let i = idx as usize;
        if i < n { valence[i] += 1; }
    }
    let scores = valence.iter().map(|&v| v as f32).collect();
    VertexImportance { scores }
}

#[allow(dead_code)]
pub fn importance_at(vi: &VertexImportance, idx: usize) -> f32 {
    vi.scores.get(idx).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn max_importance(vi: &VertexImportance) -> f32 {
    vi.scores.iter().copied().fold(f32::NEG_INFINITY, f32::max)
}

#[allow(dead_code)]
pub fn min_importance(vi: &VertexImportance) -> f32 {
    vi.scores.iter().copied().fold(f32::INFINITY, f32::min)
}

#[allow(dead_code)]
pub fn importance_threshold(vi: &VertexImportance, threshold: f32) -> Vec<usize> {
    vi.scores.iter().enumerate().filter(|(_, &s)| s >= threshold).map(|(i, _)| i).collect()
}

#[allow(dead_code)]
pub fn important_vertices(vi: &VertexImportance, top_n: usize) -> Vec<usize> {
    let mut indexed: Vec<(usize, f32)> = vi.scores.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    indexed.into_iter().take(top_n).map(|(i, _)| i).collect()
}

#[allow(dead_code)]
pub fn importance_to_json(vi: &VertexImportance) -> String {
    let vals: Vec<String> = vi.scores.iter().map(|s| format!("{:.4}", s)).collect();
    format!("{{\"importance\":[{}]}}", vals.join(","))
}

#[allow(dead_code)]
pub fn importance_count(vi: &VertexImportance) -> usize { vi.scores.len() }

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> (Vec<[f32; 3]>, Vec<u32>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[0.5,0.0,1.0]], vec![0,1,2,0,2,3,0,1,3])
    }
    #[test] fn test_compute() { let (p,i) = data(); let vi = compute_importance(&p,&i); assert_eq!(importance_count(&vi), 4); }
    #[test] fn test_at() { let (p,i) = data(); let vi = compute_importance(&p,&i); assert!(importance_at(&vi, 0) > 0.0); }
    #[test] fn test_max() { let (p,i) = data(); let vi = compute_importance(&p,&i); assert!(max_importance(&vi) > 0.0); }
    #[test] fn test_min() { let (p,i) = data(); let vi = compute_importance(&p,&i); assert!(min_importance(&vi) > 0.0); }
    #[test] fn test_threshold() { let (p,i) = data(); let vi = compute_importance(&p,&i); let r = importance_threshold(&vi, 1.0); assert!(!r.is_empty()); }
    #[test] fn test_important() { let (p,i) = data(); let vi = compute_importance(&p,&i); let r = important_vertices(&vi, 2); assert_eq!(r.len(), 2); }
    #[test] fn test_json() { let (p,i) = data(); let vi = compute_importance(&p,&i); assert!(importance_to_json(&vi).contains("importance")); }
    #[test] fn test_count() { let (p,i) = data(); let vi = compute_importance(&p,&i); assert_eq!(importance_count(&vi), 4); }
    #[test] fn test_empty() { let vi = compute_importance(&[],&[]); assert_eq!(importance_count(&vi), 0); }
    #[test] fn test_at_oob() { let vi = compute_importance(&[],&[]); assert!((importance_at(&vi, 99) - 0.0).abs() < 1e-9); }
}
