#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Constraint island solver for grouping connected bodies.

/// A group of connected bodies and constraints.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Island {
    bodies: Vec<u32>,
    constraints: Vec<u32>,
    sleeping: bool,
}

/// Solver that manages islands.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct IslandSolver {
    islands: Vec<Island>,
}

#[allow(dead_code)]
pub fn new_island() -> Island {
    Island {
        bodies: Vec::new(),
        constraints: Vec::new(),
        sleeping: false,
    }
}

#[allow(dead_code)]
pub fn add_body_to_island(island: &mut Island, body_id: u32) {
    if !island.bodies.contains(&body_id) {
        island.bodies.push(body_id);
    }
}

#[allow(dead_code)]
pub fn island_body_count(island: &Island) -> usize {
    island.bodies.len()
}

#[allow(dead_code)]
pub fn island_constraint_count(island: &Island) -> usize {
    island.constraints.len()
}

#[allow(dead_code)]
pub fn solve_island(island: &mut Island, iterations: u32) -> f32 {
    // Stub: return a residual that decreases with iterations.
    let base = island.constraints.len() as f32;
    if iterations == 0 {
        return base;
    }
    base / iterations as f32
}

#[allow(dead_code)]
pub fn merge_islands(a: &Island, b: &Island) -> Island {
    let mut merged = Island {
        bodies: a.bodies.clone(),
        constraints: a.constraints.clone(),
        sleeping: a.sleeping && b.sleeping,
    };
    for &body in &b.bodies {
        if !merged.bodies.contains(&body) {
            merged.bodies.push(body);
        }
    }
    for &c in &b.constraints {
        if !merged.constraints.contains(&c) {
            merged.constraints.push(c);
        }
    }
    merged
}

#[allow(dead_code)]
pub fn island_is_sleeping(island: &Island) -> bool {
    island.sleeping
}

#[allow(dead_code)]
pub fn split_island(island: &Island) -> Vec<Island> {
    // Stub: no actual graph analysis, just return the island as-is.
    vec![island.clone()]
}

impl IslandSolver {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn add_island(&mut self, island: Island) {
        self.islands.push(island);
    }

    #[allow(dead_code)]
    pub fn island_count(&self) -> usize {
        self.islands.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_island() {
        let i = new_island();
        assert_eq!(island_body_count(&i), 0);
        assert_eq!(island_constraint_count(&i), 0);
    }

    #[test]
    fn test_add_body() {
        let mut i = new_island();
        add_body_to_island(&mut i, 1);
        add_body_to_island(&mut i, 2);
        assert_eq!(island_body_count(&i), 2);
    }

    #[test]
    fn test_add_duplicate_body() {
        let mut i = new_island();
        add_body_to_island(&mut i, 1);
        add_body_to_island(&mut i, 1);
        assert_eq!(island_body_count(&i), 1);
    }

    #[test]
    fn test_solve() {
        let mut i = new_island();
        i.constraints.push(0);
        i.constraints.push(1);
        let residual = solve_island(&mut i, 4);
        assert!((residual - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_solve_zero_iter() {
        let mut i = new_island();
        i.constraints.push(0);
        assert_eq!(solve_island(&mut i, 0), 1.0);
    }

    #[test]
    fn test_merge() {
        let mut a = new_island();
        add_body_to_island(&mut a, 1);
        let mut b = new_island();
        add_body_to_island(&mut b, 2);
        let merged = merge_islands(&a, &b);
        assert_eq!(island_body_count(&merged), 2);
    }

    #[test]
    fn test_sleeping() {
        let i = new_island();
        assert!(!island_is_sleeping(&i));
    }

    #[test]
    fn test_split() {
        let i = new_island();
        let parts = split_island(&i);
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn test_solver() {
        let mut solver = IslandSolver::new();
        solver.add_island(new_island());
        assert_eq!(solver.island_count(), 1);
    }

    #[test]
    fn test_merge_overlapping() {
        let mut a = new_island();
        add_body_to_island(&mut a, 1);
        let mut b = new_island();
        add_body_to_island(&mut b, 1);
        add_body_to_island(&mut b, 2);
        let merged = merge_islands(&a, &b);
        assert_eq!(island_body_count(&merged), 2);
    }
}
