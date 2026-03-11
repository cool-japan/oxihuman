// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Parallel constraint grouping via graph coloring.
//!
//! Partitions a set of constraints into independent groups (colors) so that
//! all constraints within a group can be solved in parallel without write
//! conflicts.  Provides both a greedy degree-ordered coloring and a simpler
//! independent-set coloring.

/// A constraint graph where each edge represents a constraint between two particles.
#[allow(dead_code)]
pub struct ConstraintGraph {
    pub n_particles: usize,
    /// Each entry is a pair of particle indices for one constraint.
    pub edges: Vec<(usize, usize)>,
}

/// A single color group containing indices into the original constraint list.
#[allow(dead_code)]
pub struct ColorGroup {
    pub color: u32,
    pub constraint_indices: Vec<usize>,
}

/// Result of a graph coloring operation.
#[allow(dead_code)]
pub struct ColoringResult {
    pub groups: Vec<ColorGroup>,
    pub n_colors: u32,
    pub max_group_size: usize,
    /// Average constraints per color divided by total constraint count.
    pub parallelism_ratio: f32,
}

/// Build a constraint graph from a list of particle-pair constraints.
pub fn build_constraint_graph(
    n_particles: usize,
    constraints: &[(usize, usize)],
) -> ConstraintGraph {
    ConstraintGraph {
        n_particles,
        edges: constraints.to_vec(),
    }
}

/// For each particle, count how many constraints reference it.
pub fn constraint_particle_degree(graph: &ConstraintGraph) -> Vec<usize> {
    let mut degree = vec![0usize; graph.n_particles];
    for &(a, b) in &graph.edges {
        if a < degree.len() {
            degree[a] += 1;
        }
        if b < degree.len() {
            degree[b] += 1;
        }
    }
    degree
}

/// Greedy graph coloring: order constraints by descending maximum-particle degree,
/// then assign the smallest color not used by any adjacent constraint.
///
/// Two constraints are adjacent if they share at least one particle.
pub fn greedy_graph_color(graph: &ConstraintGraph) -> ColoringResult {
    let n = graph.edges.len();
    if n == 0 {
        return ColoringResult {
            groups: Vec::new(),
            n_colors: 0,
            max_group_size: 0,
            parallelism_ratio: 0.0,
        };
    }

    let degrees = constraint_particle_degree(graph);

    // Sort constraint indices by the sum of degrees of their two particles (desc).
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&ci, &cj| {
        let da = constraint_degree_sum(&graph.edges[ci], &degrees);
        let db = constraint_degree_sum(&graph.edges[cj], &degrees);
        db.cmp(&da)
    });

    // Assign colors.
    let mut colors = vec![u32::MAX; n];
    for &ci in &order {
        let (pa, pb) = graph.edges[ci];
        // Collect colors used by neighbours (constraints sharing a particle).
        let used: std::collections::BTreeSet<u32> = graph
            .edges
            .iter()
            .enumerate()
            .filter(|&(cj, &(qa, qb))| {
                cj != ci && colors[cj] != u32::MAX && (qa == pa || qa == pb || qb == pa || qb == pb)
            })
            .map(|(cj, _)| colors[cj])
            .collect();
        // Find the smallest unused color.
        let mut c = 0u32;
        while used.contains(&c) {
            c += 1;
        }
        colors[ci] = c;
    }

    build_coloring_result(n, &colors)
}

/// Independent-set coloring: greedily mark each constraint and its neighbours,
/// forming one independent set per pass until all constraints are colored.
pub fn independent_set_coloring(graph: &ConstraintGraph) -> ColoringResult {
    let n = graph.edges.len();
    if n == 0 {
        return ColoringResult {
            groups: Vec::new(),
            n_colors: 0,
            max_group_size: 0,
            parallelism_ratio: 0.0,
        };
    }

    let mut colors = vec![u32::MAX; n];
    let mut current_color = 0u32;
    let mut remaining: Vec<usize> = (0..n).collect();

    while !remaining.is_empty() {
        let mut blocked = vec![false; n];
        let mut next_remaining = Vec::new();
        for &ci in &remaining {
            if blocked[ci] {
                next_remaining.push(ci);
                continue;
            }
            // Color this constraint and block its neighbours.
            colors[ci] = current_color;
            let (pa, pb) = graph.edges[ci];
            for &cj in &remaining {
                if cj == ci || blocked[cj] {
                    continue;
                }
                let (qa, qb) = graph.edges[cj];
                if qa == pa || qa == pb || qb == pa || qb == pb {
                    blocked[cj] = true;
                }
            }
        }
        remaining = next_remaining;
        current_color += 1;
    }

    build_coloring_result(n, &colors)
}

/// Verify that no two adjacent constraints (sharing a particle) share the same color.
pub fn validate_coloring(graph: &ConstraintGraph, result: &ColoringResult) -> bool {
    // Build a flat color lookup: constraint_index → color.
    let mut color_of = vec![u32::MAX; graph.edges.len()];
    for group in &result.groups {
        for &ci in &group.constraint_indices {
            if ci < color_of.len() {
                color_of[ci] = group.color;
            }
        }
    }

    for ci in 0..graph.edges.len() {
        let (pa, pb) = graph.edges[ci];
        for cj in (ci + 1)..graph.edges.len() {
            let (qa, qb) = graph.edges[cj];
            if (qa == pa || qa == pb || qb == pa || qb == pb) && color_of[ci] == color_of[cj] {
                return false;
            }
        }
    }
    true
}

/// Return a human-readable summary of a coloring result.
pub fn coloring_stats(result: &ColoringResult) -> String {
    format!(
        "colors={}, max_group={}, parallelism={:.3}",
        result.n_colors, result.max_group_size, result.parallelism_ratio
    )
}

/// Return the constraint groups in order, each as a `Vec<usize>` of constraint indices.
///
/// Suitable for sequential group-by-group solving where each group can be
/// processed in parallel internally.
pub fn optimal_substep_order(result: &ColoringResult) -> Vec<Vec<usize>> {
    let mut sorted = result.groups.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|g| g.color);
    sorted
        .into_iter()
        .map(|g| g.constraint_indices.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn constraint_degree_sum(edge: &(usize, usize), degrees: &[usize]) -> usize {
    let da = if edge.0 < degrees.len() {
        degrees[edge.0]
    } else {
        0
    };
    let db = if edge.1 < degrees.len() {
        degrees[edge.1]
    } else {
        0
    };
    da + db
}

fn build_coloring_result(n: usize, colors: &[u32]) -> ColoringResult {
    let max_c = colors
        .iter()
        .filter(|&&c| c != u32::MAX)
        .copied()
        .max()
        .unwrap_or(0);
    let n_colors = max_c + 1;

    let mut groups: Vec<ColorGroup> = (0..n_colors)
        .map(|c| ColorGroup {
            color: c,
            constraint_indices: Vec::new(),
        })
        .collect();

    for (ci, &c) in colors.iter().enumerate().take(n) {
        if c != u32::MAX {
            groups[c as usize].constraint_indices.push(ci);
        }
    }

    let max_group_size = groups
        .iter()
        .map(|g| g.constraint_indices.len())
        .max()
        .unwrap_or(0);
    let avg = if n_colors > 0 {
        n as f32 / n_colors as f32
    } else {
        0.0
    };
    let parallelism_ratio = if n > 0 { avg / n as f32 } else { 0.0 };

    ColoringResult {
        groups,
        n_colors,
        max_group_size,
        parallelism_ratio,
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_constraint_graph_edges_correct() {
        let edges = vec![(0, 1), (1, 2), (2, 3)];
        let g = build_constraint_graph(4, &edges);
        assert_eq!(g.n_particles, 4);
        assert_eq!(g.edges, edges);
    }

    #[test]
    fn constraint_particle_degree_correct() {
        let edges = vec![(0, 1), (1, 2), (0, 2)];
        let g = build_constraint_graph(3, &edges);
        let deg = constraint_particle_degree(&g);
        // particle 0: edges 0,2 → degree 2
        // particle 1: edges 0,1 → degree 2
        // particle 2: edges 1,2 → degree 2
        assert_eq!(deg[0], 2);
        assert_eq!(deg[1], 2);
        assert_eq!(deg[2], 2);
    }

    #[test]
    fn greedy_coloring_is_valid() {
        let edges = vec![(0, 1), (1, 2), (2, 3), (3, 0)];
        let g = build_constraint_graph(4, &edges);
        let result = greedy_graph_color(&g);
        assert!(
            validate_coloring(&g, &result),
            "greedy coloring must be valid"
        );
    }

    #[test]
    fn greedy_no_shared_colors_for_adjacent() {
        let edges = vec![(0, 1), (1, 2)]; // c0 adj c1
        let g = build_constraint_graph(3, &edges);
        let result = greedy_graph_color(&g);
        assert!(validate_coloring(&g, &result));
        // The two constraints share particle 1 so must have different colors.
        assert!(result.n_colors >= 2, "adjacent constraints need ≥2 colors");
    }

    #[test]
    fn independent_set_coloring_valid() {
        let edges = vec![(0, 1), (1, 2), (2, 3), (0, 3)];
        let g = build_constraint_graph(4, &edges);
        let result = independent_set_coloring(&g);
        assert!(
            validate_coloring(&g, &result),
            "independent-set coloring must be valid"
        );
    }

    #[test]
    fn empty_graph_produces_zero_colors() {
        let g = build_constraint_graph(0, &[]);
        let greedy = greedy_graph_color(&g);
        let indep = independent_set_coloring(&g);
        assert_eq!(greedy.n_colors, 0);
        assert_eq!(indep.n_colors, 0);
    }

    #[test]
    fn single_constraint_gets_one_color() {
        let edges = vec![(0, 1)];
        let g = build_constraint_graph(2, &edges);
        let result = greedy_graph_color(&g);
        assert_eq!(result.n_colors, 1);
    }

    #[test]
    fn chain_of_constraints_needs_two_colors() {
        // A chain: c0=(0,1), c1=(1,2), c2=(2,3) — alternating independent sets.
        let edges = vec![(0, 1), (1, 2), (2, 3)];
        let g = build_constraint_graph(4, &edges);
        let result = greedy_graph_color(&g);
        assert!(
            result.n_colors >= 2,
            "chain needs ≥2 colors, got {}",
            result.n_colors
        );
        assert!(validate_coloring(&g, &result));
    }

    #[test]
    fn coloring_stats_non_empty_string() {
        let edges = vec![(0, 1), (2, 3)];
        let g = build_constraint_graph(4, &edges);
        let result = greedy_graph_color(&g);
        let s = coloring_stats(&result);
        assert!(
            !s.is_empty(),
            "coloring_stats must return a non-empty string"
        );
    }

    #[test]
    fn greedy_colors_le_independent_set_colors() {
        // Greedy with degree ordering should use ≤ colors than naïve independent sets.
        let edges = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)];
        let g = build_constraint_graph(5, &edges);
        let greedy = greedy_graph_color(&g);
        let indep = independent_set_coloring(&g);
        assert!(
            greedy.n_colors <= indep.n_colors,
            "greedy ({}) should use ≤ colors than independent_set ({})",
            greedy.n_colors,
            indep.n_colors
        );
    }

    #[test]
    fn optimal_substep_order_length_equals_n_colors() {
        let edges = vec![(0, 1), (1, 2), (2, 3)];
        let g = build_constraint_graph(4, &edges);
        let result = greedy_graph_color(&g);
        let order = optimal_substep_order(&result);
        assert_eq!(order.len(), result.n_colors as usize);
    }

    #[test]
    fn optimal_substep_order_covers_all_constraints() {
        let edges = vec![(0, 1), (1, 2), (2, 3), (3, 4)];
        let g = build_constraint_graph(5, &edges);
        let result = greedy_graph_color(&g);
        let order = optimal_substep_order(&result);
        let total: usize = order.iter().map(|v| v.len()).sum();
        assert_eq!(
            total,
            edges.len(),
            "all constraints must appear in the order"
        );
    }

    #[test]
    fn validate_coloring_detects_bad_coloring() {
        // Manually create an invalid coloring: two adjacent constraints same color.
        let edges = vec![(0, 1), (1, 2)];
        let g = build_constraint_graph(3, &edges);
        let bad_result = ColoringResult {
            groups: vec![ColorGroup {
                color: 0,
                constraint_indices: vec![0, 1], // both adjacent – invalid!
            }],
            n_colors: 1,
            max_group_size: 2,
            parallelism_ratio: 1.0,
        };
        assert!(
            !validate_coloring(&g, &bad_result),
            "should detect invalid coloring"
        );
    }
}
