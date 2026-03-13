// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vertex reordering strategies: cache-friendly, axis-sorted, and Morton order.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStrategy {
    AxisX,
    AxisY,
    AxisZ,
    CacheLinear,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexOrderResult {
    pub permutation: Vec<u32>,
    pub strategy: OrderStrategy,
}

#[allow(dead_code)]
pub fn order_by_axis(positions: &[[f32; 3]], axis: usize) -> VertexOrderResult {
    let mut indices: Vec<u32> = (0..positions.len() as u32).collect();
    indices.sort_by(|&a, &b| {
        let va = positions[a as usize][axis];
        let vb = positions[b as usize][axis];
        va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
    });
    let strategy = match axis {
        0 => OrderStrategy::AxisX,
        1 => OrderStrategy::AxisY,
        _ => OrderStrategy::AxisZ,
    };
    VertexOrderResult {
        permutation: indices,
        strategy,
    }
}

#[allow(dead_code)]
pub fn order_cache_linear(vertex_count: usize) -> VertexOrderResult {
    VertexOrderResult {
        permutation: (0..vertex_count as u32).collect(),
        strategy: OrderStrategy::CacheLinear,
    }
}

#[allow(dead_code)]
pub fn apply_vertex_order(positions: &[[f32; 3]], order: &VertexOrderResult) -> Vec<[f32; 3]> {
    order
        .permutation
        .iter()
        .map(|&i| positions[i as usize])
        .collect()
}

#[allow(dead_code)]
pub fn remap_indices(indices: &[u32], inv_perm: &[u32]) -> Vec<u32> {
    indices
        .iter()
        .map(|&i| {
            if (i as usize) < inv_perm.len() {
                inv_perm[i as usize]
            } else {
                i
            }
        })
        .collect()
}

#[allow(dead_code)]
pub fn invert_permutation(perm: &[u32]) -> Vec<u32> {
    let n = perm.len();
    let mut inv = vec![0u32; n];
    for (new_pos, &old_pos) in perm.iter().enumerate() {
        if (old_pos as usize) < n {
            inv[old_pos as usize] = new_pos as u32;
        }
    }
    inv
}

#[allow(dead_code)]
pub fn order_result_to_json(result: &VertexOrderResult) -> String {
    let strat = match result.strategy {
        OrderStrategy::AxisX => "axis_x",
        OrderStrategy::AxisY => "axis_y",
        OrderStrategy::AxisZ => "axis_z",
        OrderStrategy::CacheLinear => "cache_linear",
    };
    format!(
        "{{\"strategy\":\"{}\",\"vertex_count\":{}}}",
        strat,
        result.permutation.len()
    )
}

#[allow(dead_code)]
pub fn is_valid_permutation(perm: &[u32]) -> bool {
    let n = perm.len();
    let mut seen = vec![false; n];
    for &p in perm {
        if (p as usize) >= n || seen[p as usize] {
            return false;
        }
        seen[p as usize] = true;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_by_x() {
        let positions = vec![[3.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let result = order_by_axis(&positions, 0);
        assert_eq!(result.permutation[0], 1);
        assert_eq!(result.permutation[2], 0);
    }

    #[test]
    fn test_order_cache_linear() {
        let result = order_cache_linear(5);
        assert_eq!(result.permutation, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_apply_order() {
        let positions = vec![[2.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let result = order_by_axis(&positions, 0);
        let reordered = apply_vertex_order(&positions, &result);
        assert!((reordered[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_invert_permutation() {
        let perm = vec![2u32, 0, 1];
        let inv = invert_permutation(&perm);
        assert_eq!(inv[2], 0);
        assert_eq!(inv[0], 1);
    }

    #[test]
    fn test_valid_permutation() {
        let perm = vec![1u32, 0, 2];
        assert!(is_valid_permutation(&perm));
    }

    #[test]
    fn test_invalid_permutation_duplicate() {
        let perm = vec![0u32, 0, 2];
        assert!(!is_valid_permutation(&perm));
    }

    #[test]
    fn test_remap_indices() {
        let inv = vec![1u32, 0, 2];
        let indices = vec![0u32, 1];
        let remapped = remap_indices(&indices, &inv);
        assert_eq!(remapped[0], 1);
        assert_eq!(remapped[1], 0);
    }

    #[test]
    fn test_json_output() {
        let result = order_cache_linear(3);
        let j = order_result_to_json(&result);
        assert!(j.contains("cache_linear"));
    }

    #[test]
    fn test_strategy_axis_y() {
        let result = order_by_axis(&[[0.0, 2.0, 0.0], [0.0, 1.0, 0.0]], 1);
        assert_eq!(result.strategy, OrderStrategy::AxisY);
    }

    #[test]
    fn test_empty_positions() {
        let result = order_by_axis(&[], 0);
        assert!(result.permutation.is_empty());
    }
}
