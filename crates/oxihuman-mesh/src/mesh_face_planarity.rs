#![allow(dead_code)]
//! Face planarity analysis for polygon meshes.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FacePlanarity { errors: Vec<f32> }

fn cross(a: [f32;3], b: [f32;3]) -> [f32;3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}

fn sub(a: [f32;3], b: [f32;3]) -> [f32;3] {
    [a[0]-b[0], a[1]-b[1], a[2]-b[2]]
}

fn dot(a: [f32;3], b: [f32;3]) -> f32 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }

fn length(a: [f32;3]) -> f32 { dot(a,a).sqrt() }

#[allow(dead_code)]
pub fn compute_planarity(positions: &[[f32;3]], indices: &[u32]) -> FacePlanarity {
    let mut errors = Vec::new();
    for tri in indices.chunks(3) {
        if tri.len() < 3 { continue; }
        let (a,b,c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a >= positions.len() || b >= positions.len() || c >= positions.len() { errors.push(0.0); continue; }
        let n = cross(sub(positions[b],positions[a]), sub(positions[c],positions[a]));
        let area = length(n) * 0.5;
        errors.push(if area < 1e-10 { 1.0 } else { 0.0 });
    }
    FacePlanarity { errors }
}

#[allow(dead_code)]
pub fn planarity_at(fp: &FacePlanarity, idx: usize) -> f32 { fp.errors.get(idx).copied().unwrap_or(0.0) }

#[allow(dead_code)]
pub fn max_planarity_error(fp: &FacePlanarity) -> f32 { fp.errors.iter().copied().fold(0.0f32, f32::max) }

#[allow(dead_code)]
pub fn avg_planarity(fp: &FacePlanarity) -> f32 {
    if fp.errors.is_empty() { return 0.0; }
    fp.errors.iter().sum::<f32>() / fp.errors.len() as f32
}

#[allow(dead_code)]
pub fn non_planar_faces(fp: &FacePlanarity, threshold: f32) -> Vec<usize> {
    fp.errors.iter().enumerate().filter(|(_, &e)| e > threshold).map(|(i, _)| i).collect()
}

#[allow(dead_code)]
pub fn planarity_threshold(fp: &FacePlanarity, threshold: f32) -> usize {
    fp.errors.iter().filter(|&&e| e > threshold).count()
}

#[allow(dead_code)]
pub fn planarity_to_json(fp: &FacePlanarity) -> String {
    let vals: Vec<String> = fp.errors.iter().map(|e| format!("{:.6}", e)).collect();
    format!("{{\"planarity\":[{}]}}", vals.join(","))
}

#[allow(dead_code)]
pub fn planarity_count(fp: &FacePlanarity) -> usize { fp.errors.len() }

#[cfg(test)]
mod tests {
    use super::*;
    fn data() -> (Vec<[f32;3]>, Vec<u32>) {
        (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0]], vec![0,1,2])
    }
    #[test] fn test_compute() { let (p,i) = data(); let fp = compute_planarity(&p,&i); assert_eq!(planarity_count(&fp), 1); }
    #[test] fn test_at() { let (p,i) = data(); let fp = compute_planarity(&p,&i); assert!((planarity_at(&fp, 0) - 0.0).abs() < 1e-6); }
    #[test] fn test_max() { let (p,i) = data(); let fp = compute_planarity(&p,&i); assert!(max_planarity_error(&fp) < 1e-6); }
    #[test] fn test_avg() { let (p,i) = data(); let fp = compute_planarity(&p,&i); assert!(avg_planarity(&fp) < 1e-6); }
    #[test] fn test_non_planar() { let (p,i) = data(); let fp = compute_planarity(&p,&i); assert!(non_planar_faces(&fp, 0.5).is_empty()); }
    #[test] fn test_threshold() { let (p,i) = data(); let fp = compute_planarity(&p,&i); assert_eq!(planarity_threshold(&fp, 0.5), 0); }
    #[test] fn test_json() { let (p,i) = data(); let fp = compute_planarity(&p,&i); assert!(planarity_to_json(&fp).contains("planarity")); }
    #[test] fn test_count() { let (p,i) = data(); let fp = compute_planarity(&p,&i); assert_eq!(planarity_count(&fp), 1); }
    #[test] fn test_degenerate() {
        let p = vec![[0.0,0.0,0.0],[0.0,0.0,0.0],[0.0,0.0,0.0]];
        let fp = compute_planarity(&p, &[0,1,2]);
        assert!((planarity_at(&fp, 0) - 1.0).abs() < 1e-6);
    }
    #[test] fn test_empty() { let fp = compute_planarity(&[],&[]); assert_eq!(planarity_count(&fp), 0); }
}
