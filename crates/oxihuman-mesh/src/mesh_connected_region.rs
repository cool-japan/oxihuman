// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Find connected regions in a triangle mesh via union-find.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConnectedRegions {
    pub labels: Vec<u32>,
    pub region_count: u32,
}

#[allow(dead_code)]
fn find(parent: &mut [u32], x: u32) -> u32 {
    let mut r = x;
    while parent[r as usize] != r { r = parent[r as usize]; }
    let mut c = x;
    while parent[c as usize] != r { let n = parent[c as usize]; parent[c as usize] = r; c = n; }
    r
}

#[allow(dead_code)]
fn union(parent: &mut [u32], rank: &mut [u32], a: u32, b: u32) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra == rb { return; }
    if rank[ra as usize] < rank[rb as usize] { parent[ra as usize] = rb; }
    else if rank[ra as usize] > rank[rb as usize] { parent[rb as usize] = ra; }
    else { parent[rb as usize] = ra; rank[ra as usize] += 1; }
}

#[allow(dead_code)]
pub fn find_connected_regions(vertex_count: usize, triangles: &[[u32; 3]]) -> ConnectedRegions {
    if vertex_count == 0 { return ConnectedRegions { labels: Vec::new(), region_count: 0 }; }
    let mut parent: Vec<u32> = (0..vertex_count as u32).collect();
    let mut rank = vec![0u32; vertex_count];
    for tri in triangles {
        union(&mut parent, &mut rank, tri[0], tri[1]);
        union(&mut parent, &mut rank, tri[1], tri[2]);
    }
    let labels: Vec<u32> = (0..vertex_count as u32).map(|i| find(&mut parent, i)).collect();
    let mut roots = labels.clone();
    roots.sort_unstable();
    roots.dedup();
    let region_count = roots.len() as u32;
    ConnectedRegions { labels, region_count }
}

#[allow(dead_code)]
pub fn region_vertex_counts(cr: &ConnectedRegions) -> Vec<usize> {
    use std::collections::HashMap;
    let mut counts: HashMap<u32, usize> = HashMap::new();
    for &l in &cr.labels { *counts.entry(l).or_insert(0) += 1; }
    let mut v: Vec<usize> = counts.into_values().collect();
    v.sort_unstable();
    v.reverse();
    v
}

#[allow(dead_code)]
pub fn largest_region_size(cr: &ConnectedRegions) -> usize {
    region_vertex_counts(cr).first().copied().unwrap_or(0)
}

#[allow(dead_code)]
pub fn connected_to_json(cr: &ConnectedRegions) -> String {
    format!("{{\"regions\":{},\"vertices\":{}}}", cr.region_count, cr.labels.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_single_tri() { let r = find_connected_regions(3, &[[0,1,2]]); assert_eq!(r.region_count, 1); }
    #[test] fn test_two_components() { let r = find_connected_regions(6, &[[0,1,2],[3,4,5]]); assert_eq!(r.region_count, 2); }
    #[test] fn test_empty() { let r = find_connected_regions(0, &[]); assert_eq!(r.region_count, 0); }
    #[test] fn test_isolated_vertices() { let r = find_connected_regions(4, &[[0,1,2]]); assert_eq!(r.region_count, 2); }
    #[test] fn test_vertex_counts() { let r = find_connected_regions(6, &[[0,1,2],[3,4,5]]); let c = region_vertex_counts(&r); assert_eq!(c.len(), 2); }
    #[test] fn test_largest() { let r = find_connected_regions(3, &[[0,1,2]]); assert_eq!(largest_region_size(&r), 3); }
    #[test] fn test_labels_len() { let r = find_connected_regions(3, &[[0,1,2]]); assert_eq!(r.labels.len(), 3); }
    #[test] fn test_to_json() { let r = find_connected_regions(3, &[[0,1,2]]); assert!(connected_to_json(&r).contains("regions")); }
    #[test] fn test_chain() { let r = find_connected_regions(4, &[[0,1,2],[1,2,3]]); assert_eq!(r.region_count, 1); }
    #[test] fn test_three_components() { let r = find_connected_regions(9, &[[0,1,2],[3,4,5],[6,7,8]]); assert_eq!(r.region_count, 3); }
}
