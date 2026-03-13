// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Averaged face normals for smooth shading groups.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceNormalAvgResult { pub normals: Vec<[f32;3]>, pub face_count: usize }

#[allow(dead_code)]
pub fn triangle_normal(a:[f32;3],b:[f32;3],c:[f32;3]) -> [f32;3] {
    let ab=[b[0]-a[0],b[1]-a[1],b[2]-a[2]]; let ac=[c[0]-a[0],c[1]-a[1],c[2]-a[2]];
    [ab[1]*ac[2]-ab[2]*ac[1], ab[2]*ac[0]-ab[0]*ac[2], ab[0]*ac[1]-ab[1]*ac[0]]
}

#[allow(dead_code)]
pub fn normalize3(v:[f32;3]) -> [f32;3] {
    let l=(v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt();
    if l<1e-12{[0.0,0.0,1.0]} else {[v[0]/l,v[1]/l,v[2]/l]}
}

#[allow(dead_code)]
pub fn compute_face_normals(positions:&[[f32;3]], indices:&[[u32;3]]) -> Vec<[f32;3]> {
    indices.iter().map(|tri| {
        normalize3(triangle_normal(positions[tri[0] as usize],positions[tri[1] as usize],positions[tri[2] as usize]))
    }).collect()
}

#[allow(dead_code)]
pub fn average_vertex_normals(positions:&[[f32;3]], indices:&[[u32;3]]) -> FaceNormalAvgResult {
    let face_normals = compute_face_normals(positions, indices);
    let mut vertex_normals = vec![[0.0f32;3]; positions.len()];
    for (fi, tri) in indices.iter().enumerate() {
        let n = face_normals[fi];
        for &vi in tri { vertex_normals[vi as usize][0]+=n[0]; vertex_normals[vi as usize][1]+=n[1]; vertex_normals[vi as usize][2]+=n[2]; }
    }
    for n in &mut vertex_normals { *n = normalize3(*n); }
    FaceNormalAvgResult { normals: vertex_normals, face_count: indices.len() }
}

#[allow(dead_code)]
pub fn normal_at_vertex(result:&FaceNormalAvgResult, vi:usize) -> [f32;3] {
    if vi<result.normals.len(){result.normals[vi]} else {[0.0,0.0,1.0]}
}

#[allow(dead_code)]
pub fn all_normals_unit(result:&FaceNormalAvgResult) -> bool {
    result.normals.iter().all(|n| { let l=n[0]*n[0]+n[1]*n[1]+n[2]*n[2]; (l-1.0).abs()<0.01 })
}

#[allow(dead_code)]
pub fn face_normal_avg_to_json(result:&FaceNormalAvgResult) -> String {
    format!("{{\"normals\":{},\"faces\":{}}}", result.normals.len(), result.face_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tri() -> (Vec<[f32;3]>,Vec<[u32;3]>) { (vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]], vec![[0,1,2]]) }
    #[test] fn test_triangle_normal() { let n=triangle_normal([0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]); assert!(n[2]>0.0); }
    #[test] fn test_normalize3() { let n=normalize3([3.0,4.0,0.0]); assert!((n[0]*n[0]+n[1]*n[1]+n[2]*n[2]-1.0).abs()<1e-6); }
    #[test] fn test_compute_face_normals() { let(p,i)=tri(); let ns=compute_face_normals(&p,&i); assert_eq!(ns.len(),1); }
    #[test] fn test_average_vertex_normals() { let(p,i)=tri(); let r=average_vertex_normals(&p,&i); assert_eq!(r.normals.len(),3); }
    #[test] fn test_normal_at_vertex() { let(p,i)=tri(); let r=average_vertex_normals(&p,&i); let n=normal_at_vertex(&r,0); assert!(n[2]>0.0); }
    #[test] fn test_all_normals_unit() { let(p,i)=tri(); let r=average_vertex_normals(&p,&i); assert!(all_normals_unit(&r)); }
    #[test] fn test_oob_vertex() { let r=FaceNormalAvgResult{normals:vec![],face_count:0}; let n=normal_at_vertex(&r,999); assert!((n[2]-1.0).abs()<1e-6); }
    #[test] fn test_to_json() { let(p,i)=tri(); let r=average_vertex_normals(&p,&i); assert!(face_normal_avg_to_json(&r).contains("normals")); }
    #[test] fn test_empty() { let r=average_vertex_normals(&[],&[]); assert!(r.normals.is_empty()); }
    #[test] fn test_face_count() { let(p,i)=tri(); let r=average_vertex_normals(&p,&i); assert_eq!(r.face_count,1); }
}
