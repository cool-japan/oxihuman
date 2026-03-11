#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Vertex ordering for cache-friendly mesh traversal.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexOrder {
    pub order: Vec<usize>,
    pub method: String,
}

#[allow(dead_code)]
pub fn optimize_vertex_order(positions: &[[f32; 3]], indices: &[u32]) -> VertexOrder {
    let n = positions.len();
    let mut used = vec![false; n];
    let mut order = Vec::with_capacity(n);
    for &idx in indices {
        let i = idx as usize;
        if i < n && !used[i] {
            used[i] = true;
            order.push(i);
        }
    }
    for (i, &is_used) in used.iter().enumerate() {
        if !is_used {
            order.push(i);
        }
    }
    VertexOrder { order, method: "index_order".to_string() }
}

#[allow(dead_code)]
pub fn order_by_cache(indices: &[u32], vertex_count: usize) -> VertexOrder {
    let mut used = vec![false; vertex_count];
    let mut order = Vec::with_capacity(vertex_count);
    for &idx in indices {
        let i = idx as usize;
        if i < vertex_count && !used[i] {
            used[i] = true;
            order.push(i);
        }
    }
    for (i, &is_used) in used.iter().enumerate() {
        if !is_used {
            order.push(i);
        }
    }
    VertexOrder { order, method: "cache".to_string() }
}

#[allow(dead_code)]
pub fn order_by_position(positions: &[[f32; 3]]) -> VertexOrder {
    let mut indices: Vec<usize> = (0..positions.len()).collect();
    indices.sort_by(|&a, &b| {
        let pa = positions[a];
        let pb = positions[b];
        pa[0].partial_cmp(&pb[0]).unwrap_or(std::cmp::Ordering::Equal)
            .then(pa[1].partial_cmp(&pb[1]).unwrap_or(std::cmp::Ordering::Equal))
            .then(pa[2].partial_cmp(&pb[2]).unwrap_or(std::cmp::Ordering::Equal))
    });
    VertexOrder { order: indices, method: "position".to_string() }
}

#[allow(dead_code)]
pub fn order_by_strip(indices: &[u32], vertex_count: usize) -> VertexOrder {
    order_by_cache(indices, vertex_count)
}

#[allow(dead_code)]
pub fn vertex_reorder_map(order: &VertexOrder) -> Vec<usize> {
    let mut map = vec![0usize; order.order.len()];
    for (new_idx, &old_idx) in order.order.iter().enumerate() {
        if old_idx < map.len() {
            map[old_idx] = new_idx;
        }
    }
    map
}

#[allow(dead_code)]
pub fn apply_vertex_order(positions: &[[f32; 3]], order: &VertexOrder) -> Vec<[f32; 3]> {
    order.order.iter().map(|&i| {
        if i < positions.len() { positions[i] } else { [0.0; 3] }
    }).collect()
}

#[allow(dead_code)]
pub fn order_quality(order: &VertexOrder, indices: &[u32]) -> f32 {
    if indices.is_empty() || order.order.is_empty() {
        return 0.0;
    }
    let remap = vertex_reorder_map(order);
    let mut total_dist = 0u64;
    for tri in indices.chunks(3) {
        if tri.len() == 3 {
            let a = remap.get(tri[0] as usize).copied().unwrap_or(0);
            let b = remap.get(tri[1] as usize).copied().unwrap_or(0);
            let c = remap.get(tri[2] as usize).copied().unwrap_or(0);
            total_dist += a.abs_diff(b) as u64 + b.abs_diff(c) as u64;
        }
    }
    let n = (indices.len() / 3).max(1) as f32;
    1.0 / (1.0 + total_dist as f32 / n)
}

#[allow(dead_code)]
pub fn order_to_json(order: &VertexOrder) -> String {
    let indices_str: Vec<String> = order.order.iter().map(|i| i.to_string()).collect();
    format!(
        "{{\"method\":\"{}\",\"count\":{},\"order\":[{}]}}",
        order.method,
        order.order.len(),
        indices_str.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_vertex_order() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![2, 1, 0];
        let vo = optimize_vertex_order(&pos, &idx);
        assert_eq!(vo.order.len(), 3);
        assert_eq!(vo.order[0], 2);
    }

    #[test]
    fn test_order_by_cache() {
        let vo = order_by_cache(&[0, 1, 2], 3);
        assert_eq!(vo.order, vec![0, 1, 2]);
    }

    #[test]
    fn test_order_by_position() {
        let pos = vec![[2.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let vo = order_by_position(&pos);
        assert_eq!(vo.order[0], 1);
    }

    #[test]
    fn test_order_by_strip() {
        let vo = order_by_strip(&[1, 0, 2], 3);
        assert_eq!(vo.order[0], 1);
    }

    #[test]
    fn test_vertex_reorder_map() {
        let vo = VertexOrder { order: vec![2, 0, 1], method: "test".into() };
        let map = vertex_reorder_map(&vo);
        assert_eq!(map[2], 0);
        assert_eq!(map[0], 1);
    }

    #[test]
    fn test_apply_vertex_order() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let vo = VertexOrder { order: vec![2, 0, 1], method: "test".into() };
        let reordered = apply_vertex_order(&pos, &vo);
        assert!((reordered[0][0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_order_quality() {
        let vo = VertexOrder { order: vec![0, 1, 2], method: "test".into() };
        let q = order_quality(&vo, &[0, 1, 2]);
        assert!(q > 0.0);
    }

    #[test]
    fn test_order_quality_empty() {
        let vo = VertexOrder { order: vec![], method: "test".into() };
        assert!((order_quality(&vo, &[])).abs() < 1e-6);
    }

    #[test]
    fn test_order_to_json() {
        let vo = VertexOrder { order: vec![0, 1], method: "cache".into() };
        let json = order_to_json(&vo);
        assert!(json.contains("\"method\":\"cache\""));
        assert!(json.contains("\"count\":2"));
    }

    #[test]
    fn test_optimize_handles_duplicates() {
        let pos = vec![[0.0; 3]; 3];
        let idx = vec![0, 0, 1, 1, 2, 2];
        let vo = optimize_vertex_order(&pos, &idx);
        assert_eq!(vo.order.len(), 3);
    }
}
