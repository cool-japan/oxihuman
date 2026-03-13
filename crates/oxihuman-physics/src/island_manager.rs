// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Simulation island manager for grouping connected bodies.

use std::collections::{HashMap, HashSet};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Island {
    pub id: usize,
    pub body_ids: Vec<usize>,
    pub sleeping: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct IslandManager {
    adjacency: HashMap<usize, HashSet<usize>>,
    islands: Vec<Island>,
}

#[allow(dead_code)]
impl IslandManager {
    pub fn new() -> Self {
        Self { adjacency: HashMap::new(), islands: Vec::new() }
    }

    pub fn add_body(&mut self, body_id: usize) {
        self.adjacency.entry(body_id).or_default();
    }

    pub fn add_contact(&mut self, a: usize, b: usize) {
        self.adjacency.entry(a).or_default().insert(b);
        self.adjacency.entry(b).or_default().insert(a);
    }

    pub fn remove_contact(&mut self, a: usize, b: usize) {
        if let Some(set) = self.adjacency.get_mut(&a) { set.remove(&b); }
        if let Some(set) = self.adjacency.get_mut(&b) { set.remove(&a); }
    }

    pub fn build_islands(&mut self) {
        self.islands.clear();
        let mut visited = HashSet::new();
        let body_ids: Vec<usize> = self.adjacency.keys().copied().collect();

        for body_id in body_ids {
            if visited.contains(&body_id) { continue; }
            let mut island_bodies = Vec::new();
            let mut stack = vec![body_id];
            while let Some(current) = stack.pop() {
                if !visited.insert(current) { continue; }
                island_bodies.push(current);
                if let Some(neighbors) = self.adjacency.get(&current) {
                    for &n in neighbors {
                        if !visited.contains(&n) {
                            stack.push(n);
                        }
                    }
                }
            }
            island_bodies.sort();
            let id = self.islands.len();
            self.islands.push(Island { id, body_ids: island_bodies, sleeping: false });
        }
    }

    pub fn island_count(&self) -> usize {
        self.islands.len()
    }

    pub fn islands(&self) -> &[Island] {
        &self.islands
    }

    pub fn island_of(&self, body_id: usize) -> Option<usize> {
        self.islands.iter().find(|i| i.body_ids.contains(&body_id)).map(|i| i.id)
    }

    pub fn set_sleeping(&mut self, island_id: usize, sleeping: bool) {
        if let Some(island) = self.islands.get_mut(island_id) {
            island.sleeping = sleeping;
        }
    }

    pub fn awake_island_count(&self) -> usize {
        self.islands.iter().filter(|i| !i.sleeping).count()
    }

    pub fn body_count(&self) -> usize {
        self.adjacency.len()
    }

    pub fn clear(&mut self) {
        self.adjacency.clear();
        self.islands.clear();
    }
}

impl Default for IslandManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_body() {
        let mut mgr = IslandManager::new();
        mgr.add_body(0);
        mgr.build_islands();
        assert_eq!(mgr.island_count(), 1);
    }

    #[test]
    fn test_two_separate_bodies() {
        let mut mgr = IslandManager::new();
        mgr.add_body(0);
        mgr.add_body(1);
        mgr.build_islands();
        assert_eq!(mgr.island_count(), 2);
    }

    #[test]
    fn test_connected_bodies() {
        let mut mgr = IslandManager::new();
        mgr.add_body(0);
        mgr.add_body(1);
        mgr.add_contact(0, 1);
        mgr.build_islands();
        assert_eq!(mgr.island_count(), 1);
        assert_eq!(mgr.islands()[0].body_ids.len(), 2);
    }

    #[test]
    fn test_island_of() {
        let mut mgr = IslandManager::new();
        mgr.add_body(0);
        mgr.add_body(1);
        mgr.add_contact(0, 1);
        mgr.build_islands();
        assert_eq!(mgr.island_of(0), mgr.island_of(1));
    }

    #[test]
    fn test_remove_contact() {
        let mut mgr = IslandManager::new();
        mgr.add_body(0);
        mgr.add_body(1);
        mgr.add_contact(0, 1);
        mgr.remove_contact(0, 1);
        mgr.build_islands();
        assert_eq!(mgr.island_count(), 2);
    }

    #[test]
    fn test_sleeping() {
        let mut mgr = IslandManager::new();
        mgr.add_body(0);
        mgr.build_islands();
        mgr.set_sleeping(0, true);
        assert_eq!(mgr.awake_island_count(), 0);
    }

    #[test]
    fn test_body_count() {
        let mut mgr = IslandManager::new();
        mgr.add_body(0);
        mgr.add_body(1);
        mgr.add_body(2);
        assert_eq!(mgr.body_count(), 3);
    }

    #[test]
    fn test_clear() {
        let mut mgr = IslandManager::new();
        mgr.add_body(0);
        mgr.build_islands();
        mgr.clear();
        assert_eq!(mgr.island_count(), 0);
        assert_eq!(mgr.body_count(), 0);
    }

    #[test]
    fn test_chain_of_three() {
        let mut mgr = IslandManager::new();
        mgr.add_body(0);
        mgr.add_body(1);
        mgr.add_body(2);
        mgr.add_contact(0, 1);
        mgr.add_contact(1, 2);
        mgr.build_islands();
        assert_eq!(mgr.island_count(), 1);
    }

    #[test]
    fn test_two_groups() {
        let mut mgr = IslandManager::new();
        mgr.add_body(0);
        mgr.add_body(1);
        mgr.add_contact(0, 1);
        mgr.add_body(2);
        mgr.add_body(3);
        mgr.add_contact(2, 3);
        mgr.build_islands();
        assert_eq!(mgr.island_count(), 2);
    }
}
