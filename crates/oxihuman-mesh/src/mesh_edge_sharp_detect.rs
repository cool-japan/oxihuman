#![allow(dead_code)]
//! Sharp edge detection based on dihedral angle.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SharpEdgeDetect { edges: Vec<(u32, u32)>, threshold_deg: f32 }

fn cross3(a:[f32;3],b:[f32;3])->[f32;3]{[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]}
fn sub3(a:[f32;3],b:[f32;3])->[f32;3]{[a[0]-b[0],a[1]-b[1],a[2]-b[2]]}
fn dot3(a:[f32;3],b:[f32;3])->f32{a[0]*b[0]+a[1]*b[1]+a[2]*b[2]}
fn norm3(a:[f32;3])->f32{dot3(a,a).sqrt()}
fn normalize3(a:[f32;3])->[f32;3]{let l=norm3(a);if l<1e-10{[0.0;3]}else{[a[0]/l,a[1]/l,a[2]/l]}}

fn face_normal(p: &[[f32;3]], a: usize, b: usize, c: usize) -> [f32;3] {
    normalize3(cross3(sub3(p[b],p[a]), sub3(p[c],p[a])))
}

#[allow(dead_code)]
pub fn detect_sharp_edges_sed(positions: &[[f32;3]], indices: &[u32], threshold_deg: f32) -> SharpEdgeDetect {
    let fc = indices.len() / 3;
    let mut edge_faces: HashMap<(u32,u32), Vec<usize>> = HashMap::new();
    let mut normals = Vec::with_capacity(fc);
    for fi in 0..fc {
        let (a,b,c) = (indices[fi*3] as usize, indices[fi*3+1] as usize, indices[fi*3+2] as usize);
        normals.push(face_normal(positions, a, b, c));
        for &(e0,e1) in &[(indices[fi*3],indices[fi*3+1]),(indices[fi*3+1],indices[fi*3+2]),(indices[fi*3+2],indices[fi*3])] {
            let key = if e0<e1{(e0,e1)}else{(e1,e0)};
            edge_faces.entry(key).or_default().push(fi);
        }
    }
    let threshold_rad = threshold_deg * std::f32::consts::PI / 180.0;
    let mut edges = Vec::new();
    for (edge, faces) in &edge_faces {
        if faces.len() == 2 {
            let d = dot3(normals[faces[0]], normals[faces[1]]).clamp(-1.0, 1.0);
            if d.acos() > threshold_rad { edges.push(*edge); }
        }
    }
    SharpEdgeDetect { edges, threshold_deg }
}

#[allow(dead_code)]
pub fn sharp_edge_count(sed: &SharpEdgeDetect) -> usize { sed.edges.len() }
#[allow(dead_code)]
pub fn sharp_angle_threshold(sed: &SharpEdgeDetect) -> f32 { sed.threshold_deg }
#[allow(dead_code)]
pub fn is_sharp_edge(sed: &SharpEdgeDetect, e0: u32, e1: u32) -> bool {
    let key = if e0<e1{(e0,e1)}else{(e1,e0)};
    sed.edges.contains(&key)
}
#[allow(dead_code)]
pub fn sharp_edges_to_vec(sed: &SharpEdgeDetect) -> Vec<(u32,u32)> { sed.edges.clone() }
#[allow(dead_code)]
pub fn sharp_to_json(sed: &SharpEdgeDetect) -> String {
    let es: Vec<String> = sed.edges.iter().map(|(a,b)| format!("[{},{}]",a,b)).collect();
    format!("{{\"sharp_edges\":[{}],\"threshold\":{:.2}}}", es.join(","), sed.threshold_deg)
}
#[allow(dead_code)]
pub fn clear_sharp_edges(sed: &mut SharpEdgeDetect) { sed.edges.clear(); }
#[allow(dead_code)]
pub fn auto_detect_sharp(positions: &[[f32;3]], indices: &[u32]) -> SharpEdgeDetect {
    detect_sharp_edges_sed(positions, indices, 30.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn box_data() -> (Vec<[f32;3]>, Vec<u32>) {
        let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0],[1.0,0.0,1.0]];
        let i = vec![0,1,2, 0,2,3, 0,1,4, 1,5,4];
        (p, i)
    }
    #[test] fn test_detect() { let (p,i) = box_data(); let s = detect_sharp_edges_sed(&p,&i, 30.0); assert!(sharp_edge_count(&s) > 0); }
    #[test] fn test_count() { let (p,i) = box_data(); let s = auto_detect_sharp(&p,&i); let _ = sharp_edge_count(&s); }
    #[test] fn test_threshold() { let (p,i) = box_data(); let s = detect_sharp_edges_sed(&p,&i, 45.0); assert!((sharp_angle_threshold(&s) - 45.0).abs() < 1e-6); }
    #[test] fn test_is_sharp() { let (p,i) = box_data(); let s = auto_detect_sharp(&p,&i); let _ = is_sharp_edge(&s, 0, 1); }
    #[test] fn test_to_vec() { let (p,i) = box_data(); let s = auto_detect_sharp(&p,&i); let _ = sharp_edges_to_vec(&s); }
    #[test] fn test_json() { let (p,i) = box_data(); let s = auto_detect_sharp(&p,&i); assert!(sharp_to_json(&s).contains("sharp_edges")); }
    #[test] fn test_clear() { let (p,i) = box_data(); let mut s = auto_detect_sharp(&p,&i); clear_sharp_edges(&mut s); assert_eq!(sharp_edge_count(&s), 0); }
    #[test] fn test_auto() { let (p,i) = box_data(); let s = auto_detect_sharp(&p,&i); assert!((sharp_angle_threshold(&s) - 30.0).abs() < 1e-6); }
    #[test] fn test_empty() { let s = detect_sharp_edges_sed(&[],&[], 30.0); assert_eq!(sharp_edge_count(&s), 0); }
    #[test] fn test_flat() { let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[1.5,1.0,0.0]]; let s = detect_sharp_edges_sed(&p, &[0,1,2,1,3,2], 1.0); assert_eq!(sharp_edge_count(&s), 0); }
}
