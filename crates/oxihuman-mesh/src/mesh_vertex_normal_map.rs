#![allow(dead_code)]
//! Per-vertex normal map storage.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexNormalMap { normals: Vec<[f32; 3]> }

fn norm(a:[f32;3])->f32{(a[0]*a[0]+a[1]*a[1]+a[2]*a[2]).sqrt()}

#[allow(dead_code)]
pub fn build_normal_map(normals: &[[f32;3]]) -> VertexNormalMap {
    VertexNormalMap { normals: normals.to_vec() }
}

#[allow(dead_code)]
pub fn normal_at_vertex(nm: &VertexNormalMap, idx: usize) -> [f32;3] { nm.normals.get(idx).copied().unwrap_or([0.0;3]) }
#[allow(dead_code)]
pub fn normal_map_count(nm: &VertexNormalMap) -> usize { nm.normals.len() }

#[allow(dead_code)]
pub fn smooth_normal_map(nm: &mut VertexNormalMap, indices: &[u32]) {
    let n = nm.normals.len();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for tri in indices.chunks(3) {
        if tri.len() < 3 { continue; }
        let vs: Vec<usize> = tri.iter().map(|&v| v as usize).collect();
        for i in 0..3 { for j in 0..3 { if i != j && vs[i] < n && vs[j] < n && !adj[vs[i]].contains(&vs[j]) { adj[vs[i]].push(vs[j]); } } }
    }
    let old = nm.normals.clone();
    for i in 0..n {
        if adj[i].is_empty() { continue; }
        let mut s = old[i];
        for &j in &adj[i] { s[0] += old[j][0]; s[1] += old[j][1]; s[2] += old[j][2]; }
        let l = norm(s);
        if l > 1e-10 { nm.normals[i] = [s[0]/l, s[1]/l, s[2]/l]; }
    }
}

#[allow(dead_code)]
pub fn normal_map_to_bytes(nm: &VertexNormalMap) -> Vec<u8> {
    let mut b = Vec::with_capacity(nm.normals.len() * 12);
    for n in &nm.normals { for &v in n { b.extend_from_slice(&v.to_le_bytes()); } }
    b
}

#[allow(dead_code)]
pub fn normal_map_to_json(nm: &VertexNormalMap) -> String {
    let ns: Vec<String> = nm.normals.iter().map(|n| format!("[{:.6},{:.6},{:.6}]",n[0],n[1],n[2])).collect();
    format!("{{\"normals\":[{}]}}", ns.join(","))
}

#[allow(dead_code)]
pub fn clear_normal_map(nm: &mut VertexNormalMap) { nm.normals.clear(); }

#[allow(dead_code)]
pub fn validate_normal_map(nm: &VertexNormalMap) -> bool {
    nm.normals.iter().all(|n| { let l = norm(*n); (l - 1.0).abs() < 0.01 || l < 1e-10 })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn normals() -> Vec<[f32;3]> { vec![[0.0,0.0,1.0],[0.0,0.0,1.0],[0.0,0.0,1.0]] }
    #[test] fn test_build() { let nm = build_normal_map(&normals()); assert_eq!(normal_map_count(&nm), 3); }
    #[test] fn test_at() { let nm = build_normal_map(&normals()); assert!((normal_at_vertex(&nm, 0)[2] - 1.0).abs() < 1e-6); }
    #[test] fn test_count() { let nm = build_normal_map(&normals()); assert_eq!(normal_map_count(&nm), 3); }
    #[test] fn test_smooth() { let mut nm = build_normal_map(&normals()); smooth_normal_map(&mut nm, &[0,1,2]); assert_eq!(normal_map_count(&nm), 3); }
    #[test] fn test_bytes() { let nm = build_normal_map(&normals()); assert_eq!(normal_map_to_bytes(&nm).len(), 36); }
    #[test] fn test_json() { let nm = build_normal_map(&normals()); assert!(normal_map_to_json(&nm).contains("normals")); }
    #[test] fn test_clear() { let mut nm = build_normal_map(&normals()); clear_normal_map(&mut nm); assert_eq!(normal_map_count(&nm), 0); }
    #[test] fn test_validate() { let nm = build_normal_map(&normals()); assert!(validate_normal_map(&nm)); }
    #[test] fn test_oob() { let nm = build_normal_map(&[]); assert!((normal_at_vertex(&nm, 5)[0]).abs() < 1e-9); }
    #[test] fn test_empty_validate() { let nm = build_normal_map(&[]); assert!(validate_normal_map(&nm)); }
}
