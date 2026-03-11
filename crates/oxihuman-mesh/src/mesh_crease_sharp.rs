#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Mark edges as crease/sharp (independent from existing crease_angle).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CreaseSharpEdge {
    pub a: u32,
    pub b: u32,
    pub crease: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CreaseSharpMap {
    pub edges: Vec<CreaseSharpEdge>,
}

#[allow(dead_code)]
pub fn new_crease_sharp_map() -> CreaseSharpMap {
    CreaseSharpMap { edges: Vec::new() }
}

#[allow(dead_code)]
pub fn set_crease_sharp(map: &mut CreaseSharpMap, a: u32, b: u32, crease: f32) {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    for e in map.edges.iter_mut() {
        let (ea, eb) = if e.a <= e.b { (e.a, e.b) } else { (e.b, e.a) };
        if ea == lo && eb == hi {
            e.crease = crease;
            return;
        }
    }
    map.edges.push(CreaseSharpEdge { a: lo, b: hi, crease });
}

#[allow(dead_code)]
pub fn get_crease_sharp(map: &CreaseSharpMap, a: u32, b: u32) -> f32 {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    for e in &map.edges {
        let (ea, eb) = if e.a <= e.b { (e.a, e.b) } else { (e.b, e.a) };
        if ea == lo && eb == hi {
            return e.crease;
        }
    }
    0.0
}

#[allow(dead_code)]
pub fn remove_crease_sharp(map: &mut CreaseSharpMap, a: u32, b: u32) {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    map.edges.retain(|e| {
        let (ea, eb) = if e.a <= e.b { (e.a, e.b) } else { (e.b, e.a) };
        !(ea == lo && eb == hi)
    });
}

#[allow(dead_code)]
pub fn sharp_crease_edges(map: &CreaseSharpMap, threshold: f32) -> Vec<(u32, u32)> {
    map.edges
        .iter()
        .filter(|e| e.crease >= threshold)
        .map(|e| (e.a, e.b))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map_is_empty() {
        let m = new_crease_sharp_map();
        assert!(m.edges.is_empty());
    }

    #[test]
    fn set_and_get_crease() {
        let mut m = new_crease_sharp_map();
        set_crease_sharp(&mut m, 0, 1, 0.8);
        assert!((get_crease_sharp(&m, 0, 1) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn get_missing_returns_zero() {
        let m = new_crease_sharp_map();
        assert_eq!(get_crease_sharp(&m, 5, 6), 0.0);
    }

    #[test]
    fn order_independent_get() {
        let mut m = new_crease_sharp_map();
        set_crease_sharp(&mut m, 3, 1, 0.5);
        assert!((get_crease_sharp(&m, 1, 3) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn remove_crease_works() {
        let mut m = new_crease_sharp_map();
        set_crease_sharp(&mut m, 0, 2, 1.0);
        remove_crease_sharp(&mut m, 0, 2);
        assert_eq!(get_crease_sharp(&m, 0, 2), 0.0);
        assert!(m.edges.is_empty());
    }

    #[test]
    fn sharp_edges_above_threshold() {
        let mut m = new_crease_sharp_map();
        set_crease_sharp(&mut m, 0, 1, 0.9);
        set_crease_sharp(&mut m, 1, 2, 0.3);
        let sharp = sharp_crease_edges(&m, 0.5);
        assert_eq!(sharp.len(), 1);
        assert_eq!(sharp[0], (0, 1));
    }

    #[test]
    fn overwrite_crease_value() {
        let mut m = new_crease_sharp_map();
        set_crease_sharp(&mut m, 4, 5, 0.2);
        set_crease_sharp(&mut m, 4, 5, 0.9);
        assert_eq!(m.edges.len(), 1);
        assert!((get_crease_sharp(&m, 4, 5) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn multiple_edges_stored() {
        let mut m = new_crease_sharp_map();
        set_crease_sharp(&mut m, 0, 1, 1.0);
        set_crease_sharp(&mut m, 2, 3, 1.0);
        set_crease_sharp(&mut m, 4, 5, 1.0);
        assert_eq!(m.edges.len(), 3);
    }

    #[test]
    fn sharp_all_threshold_zero() {
        let mut m = new_crease_sharp_map();
        set_crease_sharp(&mut m, 0, 1, 0.1);
        set_crease_sharp(&mut m, 2, 3, 0.2);
        let sharp = sharp_crease_edges(&m, 0.0);
        assert_eq!(sharp.len(), 2);
    }

    #[test]
    fn remove_nonexistent_is_noop() {
        let mut m = new_crease_sharp_map();
        set_crease_sharp(&mut m, 0, 1, 0.5);
        remove_crease_sharp(&mut m, 9, 10);
        assert_eq!(m.edges.len(), 1);
    }
}
