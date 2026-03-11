#![allow(dead_code)]
//! Compute topology statistics for triangle meshes.

use std::collections::{HashMap, HashSet};

/// Topology statistics for a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TopologyStats {
    pub vertex_count: usize,
    pub face_count: usize,
    pub edge_count: usize,
    pub euler: i64,
    pub genus: i64,
    pub components: usize,
    pub boundary_components: usize,
    pub manifold: bool,
}

/// Compute topology stats from positions and triangle indices.
#[allow(dead_code)]
pub fn compute_topology_stats(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> TopologyStats {
    let v = positions.len();
    let f = indices.len();
    let mut edges: HashSet<(u32, u32)> = HashSet::new();
    for tri in indices {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let e = if a < b { (a, b) } else { (b, a) };
            edges.insert(e);
        }
    }
    let e = edges.len();
    let euler_val = v as i64 - e as i64 + f as i64;
    let comps = connected_components(positions, indices);
    let bc = boundary_component_count(indices, &edges);
    let g = genus_from_euler(euler_val, comps, bc);
    let m = is_manifold_mesh_inner(indices, &edges);

    TopologyStats {
        vertex_count: v,
        face_count: f,
        edge_count: e,
        euler: euler_val,
        genus: g,
        components: comps,
        boundary_components: bc,
        manifold: m,
    }
}

/// Return the Euler characteristic V - E + F.
#[allow(dead_code)]
pub fn euler_characteristic(stats: &TopologyStats) -> i64 {
    stats.euler
}

/// Compute the genus.
#[allow(dead_code)]
pub fn genus(stats: &TopologyStats) -> i64 {
    stats.genus
}

fn genus_from_euler(euler: i64, components: usize, boundaries: usize) -> i64 {
    // genus = (2*C - E - B) / 2 where C=components, E=euler, B=boundaries
    let val = 2 * components as i64 - euler - boundaries as i64;
    val / 2
}

/// Compute connected components using union-find.
#[allow(dead_code)]
pub fn connected_components(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> usize {
    if positions.is_empty() {
        return 0;
    }
    let n = positions.len();
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent[ra] = rb;
        }
    }

    for tri in indices {
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;
        if a < n && b < n { union(&mut parent, a, b); }
        if b < n && c < n { union(&mut parent, b, c); }
    }

    let mut roots = HashSet::new();
    // Only count vertices that appear in faces
    let mut used: HashSet<usize> = HashSet::new();
    for tri in indices {
        for &v in tri {
            if (v as usize) < n {
                used.insert(v as usize);
            }
        }
    }
    if used.is_empty() && !positions.is_empty() {
        return positions.len(); // isolated vertices
    }
    for &v in &used {
        roots.insert(find(&mut parent, v));
    }
    roots.len()
}

/// Check if the mesh is manifold.
#[allow(dead_code)]
pub fn is_manifold_mesh(indices: &[[u32; 3]]) -> bool {
    let mut edges: HashSet<(u32, u32)> = HashSet::new();
    for tri in indices {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let e = if a < b { (a, b) } else { (b, a) };
            edges.insert(e);
        }
    }
    is_manifold_mesh_inner(indices, &edges)
}

fn is_manifold_mesh_inner(indices: &[[u32; 3]], _edges: &HashSet<(u32, u32)>) -> bool {
    // An edge is non-manifold if it has more than 2 adjacent faces
    let mut edge_face_count: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in indices {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let e = if a < b { (a, b) } else { (b, a) };
            *edge_face_count.entry(e).or_insert(0) += 1;
        }
    }
    edge_face_count.values().all(|&c| (1..=2).contains(&c))
}

/// Count boundary components.
#[allow(dead_code)]
pub fn boundary_component_count(indices: &[[u32; 3]], edges: &HashSet<(u32, u32)>) -> usize {
    let mut edge_face_count: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in indices {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let e = if a < b { (a, b) } else { (b, a) };
            *edge_face_count.entry(e).or_insert(0) += 1;
        }
    }
    let boundary_edges: Vec<(u32, u32)> = edges
        .iter()
        .filter(|e| edge_face_count.get(e).copied().unwrap_or(0) == 1)
        .copied()
        .collect();
    if boundary_edges.is_empty() {
        return 0;
    }
    // Count connected components of boundary edges
    let mut verts: HashSet<u32> = HashSet::new();
    for &(a, b) in &boundary_edges {
        verts.insert(a);
        verts.insert(b);
    }
    let vert_list: Vec<u32> = verts.iter().copied().collect();
    let mut idx_map: HashMap<u32, usize> = HashMap::new();
    for (i, &v) in vert_list.iter().enumerate() {
        idx_map.insert(v, i);
    }
    let n = vert_list.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], mut x: usize) -> usize {
        while p[x] != x { p[x] = p[p[x]]; x = p[x]; }
        x
    }
    for &(a, b) in &boundary_edges {
        let ia = idx_map[&a];
        let ib = idx_map[&b];
        let ra = find(&mut parent, ia);
        let rb = find(&mut parent, ib);
        if ra != rb { parent[ra] = rb; }
    }
    let mut roots: HashSet<usize> = HashSet::new();
    for i in 0..n {
        roots.insert(find(&mut parent, i));
    }
    roots.len()
}

/// Serialize topology stats to JSON.
#[allow(dead_code)]
pub fn topology_to_json(stats: &TopologyStats) -> String {
    format!(
        "{{\"vertices\":{},\"faces\":{},\"edges\":{},\"euler\":{},\"genus\":{},\"components\":{},\"manifold\":{}}}",
        stats.vertex_count, stats.face_count, stats.edge_count,
        stats.euler, stats.genus, stats.components, stats.manifold
    )
}

/// Compute vertex valence histogram.
#[allow(dead_code)]
pub fn vertex_valence_histogram(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Vec<(u32, u32)> {
    let n = positions.len();
    let mut valence = vec![0u32; n];
    let mut counted: HashSet<(u32, u32)> = HashSet::new();
    for tri in indices {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let e = if a < b { (a, b) } else { (b, a) };
            if counted.insert(e) {
                if (a as usize) < n { valence[a as usize] += 1; }
                if (b as usize) < n { valence[b as usize] += 1; }
            }
        }
    }
    let mut hist: HashMap<u32, u32> = HashMap::new();
    for &v in &valence {
        *hist.entry(v).or_insert(0) += 1;
    }
    let mut result: Vec<(u32, u32)> = hist.into_iter().collect();
    result.sort_by_key(|&(k, _)| k);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn test_compute_topology_stats() {
        let (p, i) = triangle();
        let s = compute_topology_stats(&p, &i);
        assert_eq!(s.vertex_count, 3);
        assert_eq!(s.face_count, 1);
        assert_eq!(s.edge_count, 3);
    }

    #[test]
    fn test_euler_characteristic() {
        let (p, i) = triangle();
        let s = compute_topology_stats(&p, &i);
        assert_eq!(euler_characteristic(&s), 1); // 3 - 3 + 1
    }

    #[test]
    fn test_connected_components() {
        let (p, i) = triangle();
        assert_eq!(connected_components(&p, &i), 1);
    }

    #[test]
    fn test_connected_components_empty() {
        assert_eq!(connected_components(&[], &[]), 0);
    }

    #[test]
    fn test_is_manifold() {
        let (_, i) = triangle();
        assert!(is_manifold_mesh(&i));
    }

    #[test]
    fn test_topology_to_json() {
        let (p, i) = triangle();
        let s = compute_topology_stats(&p, &i);
        let j = topology_to_json(&s);
        assert!(j.contains("\"vertices\":3"));
    }

    #[test]
    fn test_vertex_valence_histogram() {
        let (p, i) = triangle();
        let h = vertex_valence_histogram(&p, &i);
        assert!(!h.is_empty());
    }

    #[test]
    fn test_genus_closed() {
        let (p, i) = triangle();
        let s = compute_topology_stats(&p, &i);
        assert_eq!(genus(&s), 0);
    }

    #[test]
    fn test_two_components() {
        let p = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
            [5.0, 0.0, 0.0], [6.0, 0.0, 0.0], [5.0, 1.0, 0.0],
        ];
        let i = vec![[0, 1, 2], [3, 4, 5]];
        assert_eq!(connected_components(&p, &i), 2);
    }

    #[test]
    fn test_empty_stats() {
        let s = compute_topology_stats(&[], &[]);
        assert_eq!(s.vertex_count, 0);
        assert_eq!(s.face_count, 0);
    }
}
