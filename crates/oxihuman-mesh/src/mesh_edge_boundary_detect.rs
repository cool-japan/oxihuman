#![allow(dead_code)]
//! Boundary edge detection.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoundaryEdgeDetect { edges: Vec<(u32,u32)>, loops: Vec<Vec<(u32,u32)>> }

#[allow(dead_code)]
pub fn detect_boundary_edges_bd(indices: &[u32]) -> BoundaryEdgeDetect {
    let mut edge_count: HashMap<(u32,u32), u32> = HashMap::new();
    for tri in indices.chunks(3) {
        if tri.len() < 3 { continue; }
        for k in 0..3 {
            let (a,b) = (tri[k], tri[(k+1)%3]);
            let key = if a<b{(a,b)}else{(b,a)};
            *edge_count.entry(key).or_default() += 1;
        }
    }
    let edges: Vec<(u32,u32)> = edge_count.into_iter().filter(|(_,c)| *c == 1).map(|(e,_)| e).collect();
    let loops = vec![edges.clone()]; // simplified
    BoundaryEdgeDetect { edges, loops }
}

#[allow(dead_code)]
pub fn boundary_count_bd(bd: &BoundaryEdgeDetect) -> usize { bd.edges.len() }
#[allow(dead_code)]
pub fn is_boundary_edge_bd(bd: &BoundaryEdgeDetect, e0: u32, e1: u32) -> bool {
    let key = if e0<e1{(e0,e1)}else{(e1,e0)};
    bd.edges.contains(&key)
}
#[allow(dead_code)]
pub fn boundary_loops_bd(bd: &BoundaryEdgeDetect) -> &[Vec<(u32,u32)>] { &bd.loops }
#[allow(dead_code)]
pub fn boundary_to_json_bd(bd: &BoundaryEdgeDetect) -> String {
    let es: Vec<String> = bd.edges.iter().map(|(a,b)| format!("[{},{}]",a,b)).collect();
    format!("{{\"boundary_edges\":[{}],\"count\":{}}}", es.join(","), bd.edges.len())
}
#[allow(dead_code)]
pub fn boundary_length_bd(bd: &BoundaryEdgeDetect, positions: &[[f32;3]]) -> f32 {
    let mut total = 0.0f32;
    for &(a,b) in &bd.edges {
        let (ai, bi) = (a as usize, b as usize);
        if ai < positions.len() && bi < positions.len() {
            let d = [positions[ai][0]-positions[bi][0], positions[ai][1]-positions[bi][1], positions[ai][2]-positions[bi][2]];
            total += (d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt();
        }
    }
    total
}
#[allow(dead_code)]
pub fn clear_boundary_detect(bd: &mut BoundaryEdgeDetect) { bd.edges.clear(); bd.loops.clear(); }
#[allow(dead_code)]
pub fn auto_detect_boundary(indices: &[u32]) -> BoundaryEdgeDetect { detect_boundary_edges_bd(indices) }

#[cfg(test)]
mod tests {
    use super::*;
    fn tri() -> Vec<u32> { vec![0,1,2] }
    #[test] fn test_detect() { let bd = detect_boundary_edges_bd(&tri()); assert_eq!(boundary_count_bd(&bd), 3); }
    #[test] fn test_count() { let bd = detect_boundary_edges_bd(&tri()); assert_eq!(boundary_count_bd(&bd), 3); }
    #[test] fn test_is_boundary() { let bd = detect_boundary_edges_bd(&tri()); assert!(is_boundary_edge_bd(&bd, 0, 1)); }
    #[test] fn test_loops() { let bd = detect_boundary_edges_bd(&tri()); assert!(!boundary_loops_bd(&bd).is_empty()); }
    #[test] fn test_json() { let bd = detect_boundary_edges_bd(&tri()); assert!(boundary_to_json_bd(&bd).contains("boundary_edges")); }
    #[test] fn test_length() { let p = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]]; let bd = detect_boundary_edges_bd(&tri()); assert!(boundary_length_bd(&bd, &p) > 0.0); }
    #[test] fn test_clear() { let mut bd = detect_boundary_edges_bd(&tri()); clear_boundary_detect(&mut bd); assert_eq!(boundary_count_bd(&bd), 0); }
    #[test] fn test_auto() { let bd = auto_detect_boundary(&tri()); assert_eq!(boundary_count_bd(&bd), 3); }
    #[test] fn test_closed_mesh() { let idx = vec![0,1,2, 0,2,1]; let bd = detect_boundary_edges_bd(&idx); assert_eq!(boundary_count_bd(&bd), 0); }
    #[test] fn test_empty() { let bd = detect_boundary_edges_bd(&[]); assert_eq!(boundary_count_bd(&bd), 0); }
}
