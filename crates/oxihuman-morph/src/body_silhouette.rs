#![allow(dead_code)]

/// Computed body silhouette from vertices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodySilhouette {
    pub vertices: Vec<[f32; 2]>,
    pub edges: Vec<[usize; 2]>,
}

#[allow(dead_code)]
pub fn compute_silhouette(verts_2d: &[[f32; 2]], edges: &[[usize; 2]]) -> BodySilhouette {
    BodySilhouette { vertices: verts_2d.to_vec(), edges: edges.to_vec() }
}

#[allow(dead_code)]
pub fn silhouette_vertex_count(s: &BodySilhouette) -> usize { s.vertices.len() }

#[allow(dead_code)]
pub fn silhouette_edge_count(s: &BodySilhouette) -> usize { s.edges.len() }

#[allow(dead_code)]
pub fn silhouette_bounds(s: &BodySilhouette) -> ([f32; 2], [f32; 2]) {
    if s.vertices.is_empty() { return ([0.0; 2], [0.0; 2]); }
    let mut lo = [f32::MAX; 2];
    let mut hi = [f32::MIN; 2];
    for v in &s.vertices {
        for i in 0..2 {
            if v[i] < lo[i] { lo[i] = v[i]; }
            if v[i] > hi[i] { hi[i] = v[i]; }
        }
    }
    (lo, hi)
}

#[allow(dead_code)]
pub fn silhouette_area(s: &BodySilhouette) -> f32 {
    let (lo, hi) = silhouette_bounds(s);
    (hi[0] - lo[0]) * (hi[1] - lo[1])
}

#[allow(dead_code)]
pub fn silhouette_to_json(s: &BodySilhouette) -> String {
    format!("{{\"vertices\":{},\"edges\":{}}}", s.vertices.len(), s.edges.len())
}

#[allow(dead_code)]
pub fn silhouette_from_params(width: f32, height: f32) -> BodySilhouette {
    let verts = vec![[0.0, 0.0], [width, 0.0], [width, height], [0.0, height]];
    let edges = vec![[0, 1], [1, 2], [2, 3], [3, 0]];
    BodySilhouette { vertices: verts, edges }
}

#[allow(dead_code)]
pub fn silhouette_compare(a: &BodySilhouette, b: &BodySilhouette) -> f32 {
    let aa = silhouette_area(a);
    let ab = silhouette_area(b);
    (aa - ab).abs()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_compute() {
        let s = compute_silhouette(&[[0.0, 0.0], [1.0, 1.0]], &[[0, 1]]);
        assert_eq!(silhouette_vertex_count(&s), 2);
    }
    #[test] fn test_edge_count() {
        let s = compute_silhouette(&[], &[[0, 1], [1, 2]]);
        assert_eq!(silhouette_edge_count(&s), 2);
    }
    #[test] fn test_bounds() {
        let s = compute_silhouette(&[[1.0, 2.0], [3.0, 4.0]], &[]);
        let (lo, hi) = silhouette_bounds(&s);
        assert!((lo[0] - 1.0).abs() < 1e-6);
        assert!((hi[1] - 4.0).abs() < 1e-6);
    }
    #[test] fn test_bounds_empty() {
        let s = compute_silhouette(&[], &[]);
        let (lo, _) = silhouette_bounds(&s);
        assert!((lo[0]).abs() < 1e-6);
    }
    #[test] fn test_area() {
        let s = silhouette_from_params(2.0, 3.0);
        assert!((silhouette_area(&s) - 6.0).abs() < 1e-6);
    }
    #[test] fn test_from_params() {
        let s = silhouette_from_params(1.0, 1.0);
        assert_eq!(silhouette_vertex_count(&s), 4);
        assert_eq!(silhouette_edge_count(&s), 4);
    }
    #[test] fn test_to_json() {
        let s = compute_silhouette(&[[0.0, 0.0]], &[]);
        assert!(silhouette_to_json(&s).contains("vertices"));
    }
    #[test] fn test_compare_same() {
        let a = silhouette_from_params(1.0, 1.0);
        let b = silhouette_from_params(1.0, 1.0);
        assert!((silhouette_compare(&a, &b)).abs() < 1e-6);
    }
    #[test] fn test_compare_diff() {
        let a = silhouette_from_params(1.0, 1.0);
        let b = silhouette_from_params(2.0, 2.0);
        assert!(silhouette_compare(&a, &b) > 0.0);
    }
    #[test] fn test_empty_silhouette() {
        let s = compute_silhouette(&[], &[]);
        assert_eq!(silhouette_vertex_count(&s), 0);
    }
}
