// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Simulation island: connected group of bodies for sleeping/waking.

#![allow(dead_code)]

/// A simulation island: a connected group of bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimIsland {
    pub id: u32,
    pub body_ids: Vec<u32>,
    pub sleeping: bool,
    pub energy: f32,
}

/// A collection of simulation islands.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct IslandSet {
    pub islands: Vec<SimIsland>,
    next_id: u32,
}

/// Creates a new empty island set.
#[allow(dead_code)]
pub fn new_island_set() -> IslandSet {
    IslandSet {
        islands: Vec::new(),
        next_id: 1,
    }
}

/// Creates a new island with the given body ids and returns its id.
#[allow(dead_code)]
pub fn create_island(islands: &mut IslandSet, body_ids: Vec<u32>) -> u32 {
    let id = islands.next_id;
    islands.next_id += 1;
    islands.islands.push(SimIsland {
        id,
        body_ids,
        sleeping: false,
        energy: 0.0,
    });
    id
}

/// Merges two islands by id into one (keeping `id_a`, removing `id_b`).
#[allow(dead_code)]
pub fn merge_islands(islands: &mut IslandSet, id_a: u32, id_b: u32) {
    let bodies_b: Vec<u32> = islands
        .islands
        .iter()
        .find(|i| i.id == id_b)
        .map(|i| i.body_ids.clone())
        .unwrap_or_default();
    islands.islands.retain(|i| i.id != id_b);
    if let Some(a) = islands.islands.iter_mut().find(|i| i.id == id_a) {
        for bid in bodies_b {
            if !a.body_ids.contains(&bid) {
                a.body_ids.push(bid);
            }
        }
    }
}

/// Returns the number of islands.
#[allow(dead_code)]
pub fn island_count(islands: &IslandSet) -> usize {
    islands.islands.len()
}

/// Returns the total number of bodies across all islands.
#[allow(dead_code)]
pub fn total_bodies(islands: &IslandSet) -> usize {
    islands.islands.iter().map(|i| i.body_ids.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_island_set() {
        let s = new_island_set();
        assert_eq!(island_count(&s), 0);
        assert_eq!(total_bodies(&s), 0);
    }

    #[test]
    fn test_create_island() {
        let mut s = new_island_set();
        create_island(&mut s, vec![1, 2, 3]);
        assert_eq!(island_count(&s), 1);
        assert_eq!(total_bodies(&s), 3);
    }

    #[test]
    fn test_create_multiple_islands() {
        let mut s = new_island_set();
        create_island(&mut s, vec![1, 2]);
        create_island(&mut s, vec![3, 4]);
        assert_eq!(island_count(&s), 2);
        assert_eq!(total_bodies(&s), 4);
    }

    #[test]
    fn test_create_returns_unique_ids() {
        let mut s = new_island_set();
        let id1 = create_island(&mut s, vec![1]);
        let id2 = create_island(&mut s, vec![2]);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_merge_islands() {
        let mut s = new_island_set();
        let ia = create_island(&mut s, vec![1, 2]);
        let ib = create_island(&mut s, vec![3, 4]);
        merge_islands(&mut s, ia, ib);
        assert_eq!(island_count(&s), 1);
        assert_eq!(total_bodies(&s), 4);
    }

    #[test]
    fn test_merge_no_duplicates() {
        let mut s = new_island_set();
        let ia = create_island(&mut s, vec![1, 2]);
        let ib = create_island(&mut s, vec![2, 3]);
        merge_islands(&mut s, ia, ib);
        let merged = &s.islands[0];
        // body 2 should not be duplicated
        assert_eq!(merged.body_ids.iter().filter(|&&b| b == 2).count(), 1);
    }

    #[test]
    fn test_merge_nonexistent_id() {
        let mut s = new_island_set();
        let ia = create_island(&mut s, vec![1]);
        merge_islands(&mut s, ia, 9999);
        assert_eq!(island_count(&s), 1);
    }

    #[test]
    fn test_total_bodies_empty_island() {
        let mut s = new_island_set();
        create_island(&mut s, vec![]);
        assert_eq!(total_bodies(&s), 0);
    }
}
