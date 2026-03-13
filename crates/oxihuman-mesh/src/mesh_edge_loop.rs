// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeLoop {
    pub edges: Vec<(u32, u32)>,
    pub closed: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeLoopConfig {
    pub max_length: usize,
}

#[allow(dead_code)]
pub fn default_edge_loop_config() -> EdgeLoopConfig {
    EdgeLoopConfig { max_length: 256 }
}

#[allow(dead_code)]
pub fn new_edge_loop() -> EdgeLoop {
    EdgeLoop { edges: Vec::new(), closed: false }
}

#[allow(dead_code)]
pub fn el_add_edge(loop_: &mut EdgeLoop, a: u32, b: u32) {
    loop_.edges.push((a, b));
}

#[allow(dead_code)]
pub fn el_edge_count(loop_: &EdgeLoop) -> usize {
    loop_.edges.len()
}

#[allow(dead_code)]
pub fn el_is_closed(loop_: &EdgeLoop) -> bool {
    loop_.closed
}

#[allow(dead_code)]
pub fn el_close_loop(loop_: &mut EdgeLoop) {
    loop_.closed = true;
}

#[allow(dead_code)]
pub fn el_reverse(loop_: &mut EdgeLoop) {
    loop_.edges.reverse();
    for e in &mut loop_.edges {
        *e = (e.1, e.0);
    }
}

#[allow(dead_code)]
pub fn el_to_json(loop_: &EdgeLoop) -> String {
    format!(
        r#"{{"edge_count":{},"closed":{}}}"#,
        loop_.edges.len(),
        loop_.closed
    )
}

#[allow(dead_code)]
pub fn el_contains_vertex(loop_: &EdgeLoop, v: u32) -> bool {
    loop_.edges.iter().any(|&(a, b)| a == v || b == v)
}

#[allow(dead_code)]
pub fn el_vertex_count(loop_: &EdgeLoop) -> usize {
    let mut verts: Vec<u32> = Vec::new();
    for &(a, b) in &loop_.edges {
        if !verts.contains(&a) {
            verts.push(a);
        }
        if !verts.contains(&b) {
            verts.push(b);
        }
    }
    verts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_edge_loop_config();
        assert_eq!(cfg.max_length, 256);
    }

    #[test]
    fn test_new_loop_empty() {
        let lp = new_edge_loop();
        assert_eq!(el_edge_count(&lp), 0);
        assert!(!el_is_closed(&lp));
    }

    #[test]
    fn test_add_edge() {
        let mut lp = new_edge_loop();
        el_add_edge(&mut lp, 0, 1);
        el_add_edge(&mut lp, 1, 2);
        assert_eq!(el_edge_count(&lp), 2);
    }

    #[test]
    fn test_close_loop() {
        let mut lp = new_edge_loop();
        el_close_loop(&mut lp);
        assert!(el_is_closed(&lp));
    }

    #[test]
    fn test_contains_vertex() {
        let mut lp = new_edge_loop();
        el_add_edge(&mut lp, 3, 7);
        assert!(el_contains_vertex(&lp, 3));
        assert!(el_contains_vertex(&lp, 7));
        assert!(!el_contains_vertex(&lp, 99));
    }

    #[test]
    fn test_vertex_count() {
        let mut lp = new_edge_loop();
        el_add_edge(&mut lp, 0, 1);
        el_add_edge(&mut lp, 1, 2);
        assert_eq!(el_vertex_count(&lp), 3);
    }

    #[test]
    fn test_reverse() {
        let mut lp = new_edge_loop();
        el_add_edge(&mut lp, 0, 1);
        el_add_edge(&mut lp, 1, 2);
        el_reverse(&mut lp);
        assert_eq!(lp.edges[0], (2, 1));
    }

    #[test]
    fn test_to_json() {
        let mut lp = new_edge_loop();
        el_add_edge(&mut lp, 0, 1);
        let j = el_to_json(&lp);
        assert!(j.contains("edge_count"));
        assert!(j.contains("closed"));
    }
}
