#![allow(dead_code)]

//! Vertex lock/freeze for mesh editing.

use std::collections::HashSet;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexLock {
    pub locked: HashSet<usize>,
    pub vertex_count: usize,
}

#[allow(dead_code)]
pub fn lock_vertex(vl: &mut VertexLock, vertex: usize) {
    if vertex < vl.vertex_count { vl.locked.insert(vertex); }
}

#[allow(dead_code)]
pub fn unlock_vertex(vl: &mut VertexLock, vertex: usize) {
    vl.locked.remove(&vertex);
}

#[allow(dead_code)]
pub fn is_vertex_locked(vl: &VertexLock, vertex: usize) -> bool {
    vl.locked.contains(&vertex)
}

#[allow(dead_code)]
pub fn locked_count(vl: &VertexLock) -> usize {
    vl.locked.len()
}

#[allow(dead_code)]
pub fn lock_all(vl: &mut VertexLock) {
    for i in 0..vl.vertex_count { vl.locked.insert(i); }
}

#[allow(dead_code)]
pub fn unlock_all(vl: &mut VertexLock) {
    vl.locked.clear();
}

#[allow(dead_code)]
pub fn locked_to_vec(vl: &VertexLock) -> Vec<usize> {
    let mut v: Vec<usize> = vl.locked.iter().copied().collect();
    v.sort_unstable();
    v
}

#[allow(dead_code)]
pub fn lock_to_json(vl: &VertexLock) -> String {
    let locked: Vec<String> = locked_to_vec(vl).iter().map(|i| i.to_string()).collect();
    format!("{{\"locked_count\":{},\"total\":{},\"vertices\":[{}]}}", vl.locked.len(), vl.vertex_count, locked.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vl() -> VertexLock { VertexLock { locked: HashSet::new(), vertex_count: 5 } }

    #[test]
    fn test_lock() { let mut v = vl(); lock_vertex(&mut v, 0); assert!(is_vertex_locked(&v, 0)); }
    #[test]
    fn test_unlock() { let mut v = vl(); lock_vertex(&mut v, 0); unlock_vertex(&mut v, 0); assert!(!is_vertex_locked(&v, 0)); }
    #[test]
    fn test_not_locked() { let v = vl(); assert!(!is_vertex_locked(&v, 0)); }
    #[test]
    fn test_count() { let mut v = vl(); lock_vertex(&mut v, 0); lock_vertex(&mut v, 1); assert_eq!(locked_count(&v), 2); }
    #[test]
    fn test_lock_all() { let mut v = vl(); lock_all(&mut v); assert_eq!(locked_count(&v), 5); }
    #[test]
    fn test_unlock_all() { let mut v = vl(); lock_all(&mut v); unlock_all(&mut v); assert_eq!(locked_count(&v), 0); }
    #[test]
    fn test_to_vec() { let mut v = vl(); lock_vertex(&mut v, 2); lock_vertex(&mut v, 0); assert_eq!(locked_to_vec(&v), vec![0, 2]); }
    #[test]
    fn test_to_json() { let v = vl(); assert!(lock_to_json(&v).contains("\"locked_count\":0")); }
    #[test]
    fn test_out_of_bounds() { let mut v = vl(); lock_vertex(&mut v, 10); assert_eq!(locked_count(&v), 0); }
    #[test]
    fn test_double_lock() { let mut v = vl(); lock_vertex(&mut v, 0); lock_vertex(&mut v, 0); assert_eq!(locked_count(&v), 1); }
}
