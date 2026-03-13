#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Sort-and-sweep broadphase (1D SAP).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SapEntry {
    pub id: u32,
    pub min_x: f32,
    pub max_x: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct BroadPhaseSap1d {
    pub entries: Vec<SapEntry>,
}

#[allow(dead_code)]
pub fn new_broad_phase_sap1d() -> BroadPhaseSap1d {
    BroadPhaseSap1d { entries: Vec::new() }
}

#[allow(dead_code)]
pub fn sap_insert(sap: &mut BroadPhaseSap1d, id: u32, min_x: f32, max_x: f32) {
    sap.entries.push(SapEntry { id, min_x, max_x });
}

#[allow(dead_code)]
pub fn sap_find_pairs(sap: &BroadPhaseSap1d) -> Vec<(u32, u32)> {
    let mut sorted = sap.entries.clone();
    sorted.sort_by(|a, b| a.min_x.partial_cmp(&b.min_x).unwrap_or(std::cmp::Ordering::Equal));
    let mut pairs = Vec::new();
    for i in 0..sorted.len() {
        for j in (i + 1)..sorted.len() {
            if sorted[j].min_x > sorted[i].max_x {
                break;
            }
            pairs.push((sorted[i].id, sorted[j].id));
        }
    }
    pairs
}

#[allow(dead_code)]
pub fn sap_remove(sap: &mut BroadPhaseSap1d, id: u32) {
    sap.entries.retain(|e| e.id != id);
}

#[allow(dead_code)]
pub fn sap_count(sap: &BroadPhaseSap1d) -> usize {
    sap.entries.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let sap = new_broad_phase_sap1d();
        assert_eq!(sap_count(&sap), 0);
    }

    #[test]
    fn test_insert() {
        let mut sap = new_broad_phase_sap1d();
        sap_insert(&mut sap, 1, 0.0, 1.0);
        assert_eq!(sap_count(&sap), 1);
    }

    #[test]
    fn test_overlapping_pair() {
        let mut sap = new_broad_phase_sap1d();
        sap_insert(&mut sap, 1, 0.0, 2.0);
        sap_insert(&mut sap, 2, 1.0, 3.0);
        let pairs = sap_find_pairs(&sap);
        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn test_no_overlap() {
        let mut sap = new_broad_phase_sap1d();
        sap_insert(&mut sap, 1, 0.0, 1.0);
        sap_insert(&mut sap, 2, 2.0, 3.0);
        let pairs = sap_find_pairs(&sap);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_remove() {
        let mut sap = new_broad_phase_sap1d();
        sap_insert(&mut sap, 1, 0.0, 1.0);
        sap_insert(&mut sap, 2, 2.0, 3.0);
        sap_remove(&mut sap, 1);
        assert_eq!(sap_count(&sap), 1);
    }

    #[test]
    fn test_three_overlap() {
        let mut sap = new_broad_phase_sap1d();
        sap_insert(&mut sap, 1, 0.0, 3.0);
        sap_insert(&mut sap, 2, 1.0, 4.0);
        sap_insert(&mut sap, 3, 2.0, 5.0);
        let pairs = sap_find_pairs(&sap);
        assert!(pairs.len() >= 2);
    }

    #[test]
    fn test_pair_ids() {
        let mut sap = new_broad_phase_sap1d();
        sap_insert(&mut sap, 10, 0.0, 2.0);
        sap_insert(&mut sap, 20, 1.0, 3.0);
        let pairs = sap_find_pairs(&sap);
        assert!(pairs.contains(&(10, 20)));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut sap = new_broad_phase_sap1d();
        sap_insert(&mut sap, 1, 0.0, 1.0);
        sap_remove(&mut sap, 99);
        assert_eq!(sap_count(&sap), 1);
    }

    #[test]
    fn test_empty_pairs() {
        let sap = new_broad_phase_sap1d();
        let pairs = sap_find_pairs(&sap);
        assert!(pairs.is_empty());
    }
}
