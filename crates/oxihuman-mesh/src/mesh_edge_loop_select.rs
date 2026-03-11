#![allow(dead_code)]

//! Edge loop selection utilities.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeLoopSelect {
    pub edges: Vec<(u32, u32)>,
    pub closed: bool,
}

fn canonical(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

#[allow(dead_code)]
pub fn select_edge_loop(indices: &[u32], start_a: u32, start_b: u32) -> EdgeLoopSelect {
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for tri in indices.chunks(3) {
        if tri.len() == 3 {
            for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
                adj.entry(a).or_default().push(b);
                adj.entry(b).or_default().push(a);
            }
        }
    }
    let mut edges = vec![canonical(start_a, start_b)];
    let mut current = start_b;
    let mut prev = start_a;
    for _ in 0..indices.len() {
        if let Some(neighbors) = adj.get(&current) {
            let next = neighbors.iter().find(|&&n| n != prev && n != current);
            if let Some(&n) = next {
                let e = canonical(current, n);
                if edges.contains(&e) {
                    break;
                }
                edges.push(e);
                prev = current;
                current = n;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    let closed = edges.len() > 2 && current == start_a;
    EdgeLoopSelect { edges, closed }
}

#[allow(dead_code)]
pub fn loop_edge_count(sel: &EdgeLoopSelect) -> usize {
    sel.edges.len()
}

#[allow(dead_code)]
pub fn loop_is_closed_els(sel: &EdgeLoopSelect) -> bool {
    sel.closed
}

#[allow(dead_code)]
pub fn loop_vertices_els(sel: &EdgeLoopSelect) -> Vec<u32> {
    let mut verts = Vec::new();
    for &(a, b) in &sel.edges {
        if !verts.contains(&a) {
            verts.push(a);
        }
        if !verts.contains(&b) {
            verts.push(b);
        }
    }
    verts
}

#[allow(dead_code)]
pub fn grow_loop_selection(sel: &mut EdgeLoopSelect, indices: &[u32]) {
    let verts = loop_vertices_els(sel);
    for tri in indices.chunks(3) {
        if tri.len() == 3 {
            for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
                if verts.contains(&a) || verts.contains(&b) {
                    let e = canonical(a, b);
                    if !sel.edges.contains(&e) {
                        sel.edges.push(e);
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn shrink_loop_selection(sel: &mut EdgeLoopSelect) {
    if sel.edges.len() > 1 {
        sel.edges.pop();
    }
}

#[allow(dead_code)]
pub fn loop_to_json(sel: &EdgeLoopSelect) -> String {
    let es: Vec<String> = sel
        .edges
        .iter()
        .map(|(a, b)| format!("[{},{}]", a, b))
        .collect();
    format!(
        "{{\"edge_count\":{},\"closed\":{},\"edges\":[{}]}}",
        sel.edges.len(),
        sel.closed,
        es.join(",")
    )
}

#[allow(dead_code)]
pub fn clear_loop_selection(sel: &mut EdgeLoopSelect) {
    sel.edges.clear();
    sel.closed = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select() {
        let s = select_edge_loop(&[0, 1, 2], 0, 1);
        assert!(!s.edges.is_empty());
    }
    #[test]
    fn test_edge_count() {
        let s = select_edge_loop(&[0, 1, 2], 0, 1);
        assert!(loop_edge_count(&s) >= 1);
    }
    #[test]
    fn test_vertices() {
        let s = select_edge_loop(&[0, 1, 2], 0, 1);
        let v = loop_vertices_els(&s);
        assert!(v.contains(&0));
    }
    #[test]
    fn test_grow() {
        let mut s = EdgeLoopSelect {
            edges: vec![(0, 1)],
            closed: false,
        };
        grow_loop_selection(&mut s, &[0, 1, 2]);
        assert!(s.edges.len() > 1);
    }
    #[test]
    fn test_shrink() {
        let mut s = EdgeLoopSelect {
            edges: vec![(0, 1), (1, 2)],
            closed: false,
        };
        shrink_loop_selection(&mut s);
        assert_eq!(s.edges.len(), 1);
    }
    #[test]
    fn test_to_json() {
        let s = EdgeLoopSelect {
            edges: vec![(0, 1)],
            closed: false,
        };
        assert!(loop_to_json(&s).contains("\"edge_count\":1"));
    }
    #[test]
    fn test_clear() {
        let mut s = EdgeLoopSelect {
            edges: vec![(0, 1)],
            closed: true,
        };
        clear_loop_selection(&mut s);
        assert!(s.edges.is_empty());
        assert!(!s.closed);
    }
    #[test]
    fn test_closed_flag() {
        let s = EdgeLoopSelect {
            edges: vec![],
            closed: true,
        };
        assert!(loop_is_closed_els(&s));
    }
    #[test]
    fn test_empty_select() {
        let s = select_edge_loop(&[], 0, 1);
        assert_eq!(loop_edge_count(&s), 1);
    }
    #[test]
    fn test_shrink_min() {
        let mut s = EdgeLoopSelect {
            edges: vec![(0, 1)],
            closed: false,
        };
        shrink_loop_selection(&mut s);
        assert_eq!(s.edges.len(), 1);
    }
}
