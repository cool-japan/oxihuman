#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Store and deduplicate broadphase collision pairs.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BpPair {
    pub a: u32,
    pub b: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct BroadPhasePairs {
    pub pairs: Vec<BpPair>,
}

#[allow(dead_code)]
pub fn new_broad_phase_pairs() -> BroadPhasePairs {
    BroadPhasePairs { pairs: Vec::new() }
}

/// Normalise pair so a <= b for deduplication.
fn normalise(a: u32, b: u32) -> (u32, u32) {
    if a <= b { (a, b) } else { (b, a) }
}

#[allow(dead_code)]
pub fn bp_add(pairs: &mut BroadPhasePairs, a: u32, b: u32) {
    if a == b {
        return;
    }
    let (na, nb) = normalise(a, b);
    if !bp_contains(pairs, na, nb) {
        pairs.pairs.push(BpPair { a: na, b: nb });
    }
}

#[allow(dead_code)]
pub fn bp_contains(pairs: &BroadPhasePairs, a: u32, b: u32) -> bool {
    let (na, nb) = normalise(a, b);
    pairs.pairs.iter().any(|p| p.a == na && p.b == nb)
}

#[allow(dead_code)]
pub fn bp_clear(pairs: &mut BroadPhasePairs) {
    pairs.pairs.clear();
}

#[allow(dead_code)]
pub fn bp_count(pairs: &BroadPhasePairs) -> usize {
    pairs.pairs.len()
}

#[allow(dead_code)]
pub fn bp_for_body(pairs: &BroadPhasePairs, id: u32) -> Vec<u32> {
    pairs
        .pairs
        .iter()
        .filter_map(|p| {
            if p.a == id {
                Some(p.b)
            } else if p.b == id {
                Some(p.a)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let bp = new_broad_phase_pairs();
        assert_eq!(bp_count(&bp), 0);
    }

    #[test]
    fn add_pair() {
        let mut bp = new_broad_phase_pairs();
        bp_add(&mut bp, 1, 2);
        assert_eq!(bp_count(&bp), 1);
    }

    #[test]
    fn contains_forward() {
        let mut bp = new_broad_phase_pairs();
        bp_add(&mut bp, 1, 2);
        assert!(bp_contains(&bp, 1, 2));
    }

    #[test]
    fn contains_reversed() {
        let mut bp = new_broad_phase_pairs();
        bp_add(&mut bp, 1, 2);
        assert!(bp_contains(&bp, 2, 1));
    }

    #[test]
    fn no_duplicate() {
        let mut bp = new_broad_phase_pairs();
        bp_add(&mut bp, 1, 2);
        bp_add(&mut bp, 2, 1);
        assert_eq!(bp_count(&bp), 1);
    }

    #[test]
    fn self_pair_ignored() {
        let mut bp = new_broad_phase_pairs();
        bp_add(&mut bp, 5, 5);
        assert_eq!(bp_count(&bp), 0);
    }

    #[test]
    fn clear_empties() {
        let mut bp = new_broad_phase_pairs();
        bp_add(&mut bp, 1, 2);
        bp_add(&mut bp, 3, 4);
        bp_clear(&mut bp);
        assert_eq!(bp_count(&bp), 0);
    }

    #[test]
    fn for_body_returns_partners() {
        let mut bp = new_broad_phase_pairs();
        bp_add(&mut bp, 1, 2);
        bp_add(&mut bp, 1, 3);
        bp_add(&mut bp, 4, 5);
        let partners = bp_for_body(&bp, 1);
        assert!(partners.contains(&2));
        assert!(partners.contains(&3));
        assert!(!partners.contains(&5));
    }

    #[test]
    fn for_body_no_matches() {
        let mut bp = new_broad_phase_pairs();
        bp_add(&mut bp, 1, 2);
        let partners = bp_for_body(&bp, 99);
        assert!(partners.is_empty());
    }

    #[test]
    fn multiple_pairs() {
        let mut bp = new_broad_phase_pairs();
        for i in 0u32..5 {
            for j in (i + 1)..5 {
                bp_add(&mut bp, i, j);
            }
        }
        assert_eq!(bp_count(&bp), 10);
    }
}
