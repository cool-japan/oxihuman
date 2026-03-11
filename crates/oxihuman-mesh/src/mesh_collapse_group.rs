// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Group-based edge collapse: collapse all edges within a vertex group.

/// One collapse group record.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollapseGroup {
    pub group_id: u32,
    pub vertices: Vec<usize>,
    pub target: [f32; 3],
}

/// Result of group collapse.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollapseGroupResult {
    pub groups: Vec<CollapseGroup>,
    pub remap: Vec<usize>,
    pub collapsed_count: usize,
}

/// Build collapse groups from per-vertex group IDs.
#[allow(dead_code)]
pub fn build_collapse_groups(positions: &[[f32; 3]], group_ids: &[u32]) -> Vec<CollapseGroup> {
    assert_eq!(positions.len(), group_ids.len());
    let mut map: std::collections::HashMap<u32, Vec<usize>> = std::collections::HashMap::new();
    for (i, &gid) in group_ids.iter().enumerate() {
        map.entry(gid).or_default().push(i);
    }
    let mut groups: Vec<CollapseGroup> = map
        .into_iter()
        .map(|(gid, verts)| {
            let n = verts.len() as f32;
            let mut sum = [0.0_f32; 3];
            for &v in &verts {
                sum[0] += positions[v][0];
                sum[1] += positions[v][1];
                sum[2] += positions[v][2];
            }
            let target = [sum[0] / n, sum[1] / n, sum[2] / n];
            CollapseGroup {
                group_id: gid,
                vertices: verts,
                target,
            }
        })
        .collect();
    groups.sort_by_key(|g| g.group_id);
    groups
}

/// Apply collapse groups to produce a vertex remap array.
#[allow(dead_code)]
pub fn apply_collapse_groups(positions: &[[f32; 3]], group_ids: &[u32]) -> CollapseGroupResult {
    let groups = build_collapse_groups(positions, group_ids);
    let mut remap: Vec<usize> = (0..positions.len()).collect();
    let mut collapsed = 0;
    for (new_idx, g) in groups.iter().enumerate() {
        for &old in &g.vertices {
            remap[old] = new_idx;
        }
        if g.vertices.len() > 1 {
            collapsed += g.vertices.len() - 1;
        }
    }
    CollapseGroupResult {
        groups,
        remap,
        collapsed_count: collapsed,
    }
}

/// Number of groups.
#[allow(dead_code)]
pub fn group_count(res: &CollapseGroupResult) -> usize {
    res.groups.len()
}

/// Total vertices collapsed.
#[allow(dead_code)]
pub fn total_collapsed(res: &CollapseGroupResult) -> usize {
    res.collapsed_count
}

/// Maximum group size.
#[allow(dead_code)]
pub fn max_group_size(res: &CollapseGroupResult) -> usize {
    res.groups
        .iter()
        .map(|g| g.vertices.len())
        .max()
        .unwrap_or(0)
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn collapse_group_to_json(res: &CollapseGroupResult) -> String {
    format!(
        "{{\"group_count\":{},\"collapsed_count\":{}}}",
        group_count(res),
        res.collapsed_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_group_collapse() {
        let pos = vec![[0.0f32; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let ids = vec![0u32, 0, 0];
        let res = apply_collapse_groups(&pos, &ids);
        assert_eq!(group_count(&res), 1);
        assert_eq!(total_collapsed(&res), 2);
    }

    #[test]
    fn two_groups_no_collapse() {
        let pos = vec![[0.0f32; 3], [1.0, 0.0, 0.0]];
        let ids = vec![0u32, 1];
        let res = apply_collapse_groups(&pos, &ids);
        assert_eq!(group_count(&res), 2);
        assert_eq!(total_collapsed(&res), 0);
    }

    #[test]
    fn remap_length_preserved() {
        let pos = vec![[0.0f32; 3]; 4];
        let ids = vec![0u32, 0, 1, 1];
        let res = apply_collapse_groups(&pos, &ids);
        assert_eq!(res.remap.len(), 4);
    }

    #[test]
    fn target_is_centroid() {
        let pos = vec![[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let ids = vec![5u32, 5];
        let groups = build_collapse_groups(&pos, &ids);
        assert!((groups[0].target[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn max_group_size_correct() {
        let pos = vec![[0.0f32; 3]; 3];
        let ids = vec![0u32, 0, 0];
        let res = apply_collapse_groups(&pos, &ids);
        assert_eq!(max_group_size(&res), 3);
    }

    #[test]
    fn json_contains_group_count() {
        let pos = vec![[0.0f32; 3]; 2];
        let ids = vec![0u32, 1];
        let res = apply_collapse_groups(&pos, &ids);
        let j = collapse_group_to_json(&res);
        assert!(j.contains("group_count"));
    }

    #[test]
    fn remap_values_in_range() {
        let pos = vec![[0.0f32; 3]; 4];
        let ids = vec![0u32, 0, 1, 1];
        let res = apply_collapse_groups(&pos, &ids);
        let max_remap = res.remap.iter().copied().max().unwrap_or(0);
        assert!(max_remap < group_count(&res));
    }

    #[test]
    fn groups_sorted_by_id() {
        let pos = vec![[0.0f32; 3]; 3];
        let ids = vec![2u32, 0, 1];
        let groups = build_collapse_groups(&pos, &ids);
        for i in 1..groups.len() {
            assert!(groups[i].group_id > groups[i - 1].group_id);
        }
    }

    #[test]
    fn empty_input() {
        let res = apply_collapse_groups(&[], &[]);
        assert_eq!(group_count(&res), 0);
    }

    #[test]
    fn contains_range_check() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
