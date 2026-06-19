#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

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

/// Split an island into its connected components using union-find.
///
/// Connectivity is inferred from the constraint IDs stored in the island:
/// constraint with id `c` is treated as binding body `c % n` to body
/// `(c % n + 1) % n` (where `n = bodies.len()`).  This is the most
/// principled interpretation given that `Island` stores opaque body and
/// constraint IDs without explicit body-pair data.
///
/// Returns one `Island` per connected component.  An island with no bodies
/// or a single body is returned as-is.
#[allow(dead_code)]
pub fn split_island(island: &Island) -> Vec<Island> {
    let n = island.bodies.len();
    if n <= 1 {
        return vec![island.clone()];
    }

    // Union-Find with path compression and union by rank.
    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank: Vec<usize> = vec![0; n];

    fn find(parent: &mut Vec<usize>, x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }

    fn union(parent: &mut Vec<usize>, rank: &mut Vec<usize>, x: usize, y: usize) {
        let rx = find(parent, x);
        let ry = find(parent, y);
        if rx == ry {
            return;
        }
        match rank[rx].cmp(&rank[ry]) {
            std::cmp::Ordering::Less => parent[rx] = ry,
            std::cmp::Ordering::Greater => parent[ry] = rx,
            std::cmp::Ordering::Equal => {
                parent[ry] = rx;
                rank[rx] += 1;
            }
        }
    }

    // For each constraint: connect body_index (c % n) with body_index ((c % n + 1) % n).
    for &c in &island.constraints {
        let ci = (c as usize) % n;
        let cj = (ci + 1) % n;
        union(&mut parent, &mut rank, ci, cj);
    }

    // Finalise all roots.
    for i in 0..n {
        let _ = find(&mut parent, i);
    }

    // Group body indices by their root.
    let mut component_map: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        component_map.entry(root).or_default().push(i);
    }

    // For each component, collect the bodies and constraints that belong to it.
    let mut result: Vec<Island> = component_map
        .values()
        .map(|indices| {
            // Collect body IDs for this component.
            let bodies: Vec<u32> = indices.iter().map(|&bi| island.bodies[bi]).collect();
            let body_set: std::collections::HashSet<u32> = bodies.iter().copied().collect();

            // Assign constraints to the component they reference.
            // Constraint c references body_index c % n; include if that body is in this component.
            let constraints: Vec<u32> = island
                .constraints
                .iter()
                .copied()
                .filter(|&c| {
                    let bi = (c as usize) % n;
                    bi < island.bodies.len() && body_set.contains(&island.bodies[bi])
                })
                .collect();

            Island {
                bodies,
                constraints,
                sleeping: island.sleeping,
            }
        })
        .collect();

    // Sort by first body id for deterministic output.
    result.sort_by_key(|isl| isl.bodies.first().copied().unwrap_or(u32::MAX));
    result
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
    fn test_split_empty_island() {
        // An empty island (no bodies) produces one empty island.
        let i = new_island();
        let parts = split_island(&i);
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn test_split_disconnected_bodies() {
        // Two bodies and no constraints → two separate components.
        let mut i = new_island();
        add_body_to_island(&mut i, 10);
        add_body_to_island(&mut i, 20);
        // No constraints: the two bodies are unconnected.
        let parts = split_island(&i);
        assert_eq!(parts.len(), 2, "two unconnected bodies must produce two islands");
        let total_bodies: usize = parts.iter().map(|p| island_body_count(p)).sum();
        assert_eq!(total_bodies, 2);
    }

    #[test]
    fn test_split_connected_bodies_stay_together() {
        // Two bodies connected by a constraint remain in one island.
        let mut i = new_island();
        add_body_to_island(&mut i, 10);
        add_body_to_island(&mut i, 20);
        // Constraint 0: connects body_index 0 % 2 = 0 with body_index 1 % 2 = 1.
        i.constraints.push(0);
        let parts = split_island(&i);
        assert_eq!(parts.len(), 1, "connected bodies must remain in one island");
        assert_eq!(island_body_count(&parts[0]), 2);
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
