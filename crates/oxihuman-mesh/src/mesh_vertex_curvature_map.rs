#![allow(dead_code)]
//! Per-vertex curvature map for mesh visualization.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurvatureMap { values: Vec<f32> }

fn sub3(a: [f32;3], b: [f32;3]) -> [f32;3] { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
fn dot3(a: [f32;3], b: [f32;3]) -> f32 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }
fn len3(a: [f32;3]) -> f32 { dot3(a,a).sqrt() }

#[allow(dead_code)]
pub fn compute_curvature_map(positions: &[[f32;3]], indices: &[u32]) -> CurvatureMap {
    let n = positions.len();
    let mut angles = vec![0.0f32; n];
    let mut count = vec![0u32; n];
    for tri in indices.chunks(3) {
        if tri.len() < 3 { continue; }
        let verts: Vec<usize> = tri.iter().map(|&v| v as usize).collect();
        if verts.iter().any(|&v| v >= n) { continue; }
        for k in 0..3 {
            let (i, j, l) = (verts[k], verts[(k+1)%3], verts[(k+2)%3]);
            let a = sub3(positions[j], positions[i]);
            let b = sub3(positions[l], positions[i]);
            let la = len3(a);
            let lb = len3(b);
            if la > 1e-10 && lb > 1e-10 {
                let cos_a = (dot3(a,b) / (la * lb)).clamp(-1.0, 1.0);
                angles[i] += cos_a.acos();
                count[i] += 1;
            }
        }
    }
    let values = (0..n).map(|i| {
        if count[i] == 0 { 0.0 } else { std::f32::consts::TAU - angles[i] }
    }).collect();
    CurvatureMap { values }
}

#[allow(dead_code)]
pub fn curvature_at_cm(cm: &CurvatureMap, idx: usize) -> f32 { cm.values.get(idx).copied().unwrap_or(0.0) }
#[allow(dead_code)]
pub fn max_curvature(cm: &CurvatureMap) -> f32 { cm.values.iter().copied().fold(f32::NEG_INFINITY, f32::max) }
#[allow(dead_code)]
pub fn min_curvature(cm: &CurvatureMap) -> f32 { cm.values.iter().copied().fold(f32::INFINITY, f32::min) }
#[allow(dead_code)]
pub fn avg_curvature(cm: &CurvatureMap) -> f32 { if cm.values.is_empty() { 0.0 } else { cm.values.iter().sum::<f32>() / cm.values.len() as f32 } }

#[allow(dead_code)]
pub fn curvature_to_color(value: f32, lo: f32, hi: f32) -> [f32; 3] {
    let range = hi - lo;
    if range.abs() < 1e-10 { return [0.5, 0.5, 0.5]; }
    let t = ((value - lo) / range).clamp(0.0, 1.0);
    [t, 0.0, 1.0 - t]
}

#[allow(dead_code)]
pub fn curvature_to_json(cm: &CurvatureMap) -> String {
    let vals: Vec<String> = cm.values.iter().map(|v| format!("{:.6}", v)).collect();
    format!("{{\"curvature\":[{}]}}", vals.join(","))
}

#[allow(dead_code)]
pub fn curvature_count(cm: &CurvatureMap) -> usize { cm.values.len() }

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> (Vec<[f32;3]>, Vec<u32>) { (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]], vec![0,1,2]) }
    #[test] fn test_compute() { let (p,i) = data(); let cm = compute_curvature_map(&p,&i); assert_eq!(curvature_count(&cm), 3); }
    #[test] fn test_at() { let (p,i) = data(); let cm = compute_curvature_map(&p,&i); let _ = curvature_at_cm(&cm, 0); }
    #[test] fn test_max() { let (p,i) = data(); let cm = compute_curvature_map(&p,&i); let _ = max_curvature(&cm); }
    #[test] fn test_min() { let (p,i) = data(); let cm = compute_curvature_map(&p,&i); let _ = min_curvature(&cm); }
    #[test] fn test_avg() { let (p,i) = data(); let cm = compute_curvature_map(&p,&i); let _ = avg_curvature(&cm); }
    #[test] fn test_color() { let c = curvature_to_color(0.5, 0.0, 1.0); assert!((c[0] - 0.5).abs() < 1e-6); }
    #[test] fn test_json() { let (p,i) = data(); let cm = compute_curvature_map(&p,&i); assert!(curvature_to_json(&cm).contains("curvature")); }
    #[test] fn test_count() { let (p,i) = data(); let cm = compute_curvature_map(&p,&i); assert_eq!(curvature_count(&cm), 3); }
    #[test] fn test_empty() { let cm = compute_curvature_map(&[],&[]); assert_eq!(curvature_count(&cm), 0); }
    #[test] fn test_at_oob() { let cm = compute_curvature_map(&[],&[]); assert!((curvature_at_cm(&cm, 5) - 0.0).abs() < 1e-9); }
}
