#![allow(dead_code)]
//! Edge ring selection operations.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeRingSelect { edges: Vec<(u32, u32)>, closed: bool }

#[allow(dead_code)]
pub fn select_edge_ring_ers(indices: &[u32], start_edge: (u32, u32)) -> EdgeRingSelect {
    let fc = indices.len() / 3;
    let mut edge_faces: HashMap<(u32,u32), Vec<usize>> = HashMap::new();
    for fi in 0..fc {
        for k in 0..3 {
            let (a,b) = (indices[fi*3+k], indices[fi*3+(k+1)%3]);
            let key = if a<b{(a,b)}else{(b,a)};
            edge_faces.entry(key).or_default().push(fi);
        }
    }
    let mut ring = vec![if start_edge.0<start_edge.1{start_edge}else{(start_edge.1,start_edge.0)}];
    let mut visited = std::collections::HashSet::new();
    visited.insert(ring[0]);
    let mut changed = true;
    while changed {
        changed = false;
        if let Some(&last) = ring.last() {
            if let Some(faces) = edge_faces.get(&last) {
                for &fi in faces {
                    for k in 0..3 {
                        let (a,b) = (indices[fi*3+k], indices[fi*3+(k+1)%3]);
                        let key = if a<b{(a,b)}else{(b,a)};
                        if key != last && !visited.contains(&key) {
                            let shares = key.0 == last.0 || key.0 == last.1 || key.1 == last.0 || key.1 == last.1;
                            if !shares { ring.push(key); visited.insert(key); changed = true; break; }
                        }
                    }
                    if changed { break; }
                }
            }
        }
    }
    let closed = ring.len() > 2 && {
        let first = ring[0];
        let last = ring[ring.len()-1];
        first.0 == last.0 || first.0 == last.1 || first.1 == last.0 || first.1 == last.1
    };
    EdgeRingSelect { edges: ring, closed }
}

#[allow(dead_code)]
pub fn ring_edge_count_ers(ers: &EdgeRingSelect) -> usize { ers.edges.len() }
#[allow(dead_code)]
pub fn ring_is_closed_ers(ers: &EdgeRingSelect) -> bool { ers.closed }
#[allow(dead_code)]
pub fn ring_vertices_ers(ers: &EdgeRingSelect) -> Vec<u32> {
    let mut vs: Vec<u32> = ers.edges.iter().flat_map(|e| vec![e.0, e.1]).collect();
    vs.sort(); vs.dedup(); vs
}
#[allow(dead_code)]
pub fn ring_to_json_ers(ers: &EdgeRingSelect) -> String {
    let es: Vec<String> = ers.edges.iter().map(|(a,b)| format!("[{},{}]",a,b)).collect();
    format!("{{\"ring\":[{}],\"closed\":{}}}", es.join(","), ers.closed)
}
#[allow(dead_code)]
pub fn clear_ring_selection(ers: &mut EdgeRingSelect) { ers.edges.clear(); ers.closed = false; }
#[allow(dead_code)]
pub fn grow_ring(ers: &mut EdgeRingSelect, edge: (u32, u32)) {
    let key = if edge.0<edge.1{edge}else{(edge.1,edge.0)};
    if !ers.edges.contains(&key) { ers.edges.push(key); }
}
#[allow(dead_code)]
pub fn shrink_ring(ers: &mut EdgeRingSelect) { ers.edges.pop(); }

#[cfg(test)]
mod tests {
    use super::*;
    fn quad() -> Vec<u32> { vec![0,1,2, 0,2,3] }
    #[test] fn test_select() { let r = select_edge_ring_ers(&quad(), (0,1)); assert!(ring_edge_count_ers(&r) >= 1); }
    #[test] fn test_count() { let r = select_edge_ring_ers(&quad(), (0,1)); let _ = ring_edge_count_ers(&r); }
    #[test] fn test_closed() { let r = select_edge_ring_ers(&quad(), (0,1)); let _ = ring_is_closed_ers(&r); }
    #[test] fn test_vertices() { let r = select_edge_ring_ers(&quad(), (0,1)); let vs = ring_vertices_ers(&r); assert!(!vs.is_empty()); }
    #[test] fn test_json() { let r = select_edge_ring_ers(&quad(), (0,1)); assert!(ring_to_json_ers(&r).contains("ring")); }
    #[test] fn test_clear() { let mut r = select_edge_ring_ers(&quad(), (0,1)); clear_ring_selection(&mut r); assert_eq!(ring_edge_count_ers(&r), 0); }
    #[test] fn test_grow() { let mut r = select_edge_ring_ers(&quad(), (0,1)); grow_ring(&mut r, (5,6)); assert!(ring_edge_count_ers(&r) >= 2); }
    #[test] fn test_shrink() { let mut r = select_edge_ring_ers(&quad(), (0,1)); let before = ring_edge_count_ers(&r); shrink_ring(&mut r); assert!(ring_edge_count_ers(&r) < before || before == 0); }
    #[test] fn test_empty() { let r = select_edge_ring_ers(&[], (0,1)); assert_eq!(ring_edge_count_ers(&r), 1); }
    #[test] fn test_no_dup_grow() { let mut r = select_edge_ring_ers(&quad(), (0,1)); grow_ring(&mut r, (0,1)); let c = ring_edge_count_ers(&r); grow_ring(&mut r, (0,1)); assert_eq!(ring_edge_count_ers(&r), c); }
}
