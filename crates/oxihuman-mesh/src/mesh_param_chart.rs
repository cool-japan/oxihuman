// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A parameterization chart mapping 3D faces to a 2D UV region.
#[allow(dead_code)]
pub struct ParamChart {
    pub face_indices: Vec<usize>,
    pub uvs: Vec<[f32; 2]>,
    pub chart_id: u32,
}

#[allow(dead_code)]
pub struct ParamChartSet {
    pub charts: Vec<ParamChart>,
}

/// Build charts by connected components (simple flood-fill by face connectivity).
#[allow(dead_code)]
pub fn build_param_charts(positions: &[[f32; 3]], indices: &[u32]) -> ParamChartSet {
    let tri_count = indices.len() / 3;
    if tri_count == 0 {
        return ParamChartSet { charts: vec![] };
    }

    // Build edge -> face adjacency
    let mut edge_to_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for t in 0..tri_count {
        let a = indices[t * 3];
        let b = indices[t * 3 + 1];
        let c = indices[t * 3 + 2];
        for (p, q) in [(a, b), (b, c), (c, a)] {
            let key = if p < q { (p, q) } else { (q, p) };
            edge_to_faces.entry(key).or_default().push(t);
        }
    }

    // Build face adjacency
    let mut face_adj: Vec<Vec<usize>> = vec![vec![]; tri_count];
    for faces in edge_to_faces.values() {
        if faces.len() == 2 {
            face_adj[faces[0]].push(faces[1]);
            face_adj[faces[1]].push(faces[0]);
        }
    }

    // Flood fill into charts
    let mut visited = vec![false; tri_count];
    let mut charts = Vec::new();

    for start in 0..tri_count {
        if visited[start] {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![start];
        while let Some(f) = stack.pop() {
            if visited[f] {
                continue;
            }
            visited[f] = true;
            component.push(f);
            for &nb in &face_adj[f] {
                if !visited[nb] {
                    stack.push(nb);
                }
            }
        }
        let chart_id = charts.len() as u32;
        // Simple planar UV: project face centroids to XZ plane
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(component.len() * 3);
        for &fi in &component {
            for vi in 0..3 {
                let idx = indices[fi * 3 + vi] as usize;
                let p = if idx < positions.len() {
                    positions[idx]
                } else {
                    [0.0; 3]
                };
                uvs.push([p[0], p[2]]);
            }
        }
        // Normalize uvs to [0,1]
        if !uvs.is_empty() {
            let mut mn = uvs[0];
            let mut mx = uvs[0];
            for &uv in &uvs {
                if uv[0] < mn[0] {
                    mn[0] = uv[0];
                }
                if uv[1] < mn[1] {
                    mn[1] = uv[1];
                }
                if uv[0] > mx[0] {
                    mx[0] = uv[0];
                }
                if uv[1] > mx[1] {
                    mx[1] = uv[1];
                }
            }
            let rw = (mx[0] - mn[0]).max(1e-10);
            let rh = (mx[1] - mn[1]).max(1e-10);
            for uv in &mut uvs {
                uv[0] = (uv[0] - mn[0]) / rw;
                uv[1] = (uv[1] - mn[1]) / rh;
            }
        }
        charts.push(ParamChart {
            face_indices: component,
            uvs,
            chart_id,
        });
    }
    ParamChartSet { charts }
}

#[allow(dead_code)]
pub fn chart_count(cs: &ParamChartSet) -> usize {
    cs.charts.len()
}

#[allow(dead_code)]
pub fn total_chart_faces(cs: &ParamChartSet) -> usize {
    cs.charts.iter().map(|c| c.face_indices.len()).sum()
}

#[allow(dead_code)]
pub fn largest_chart(cs: &ParamChartSet) -> Option<&ParamChart> {
    cs.charts.iter().max_by_key(|c| c.face_indices.len())
}

#[allow(dead_code)]
pub fn chart_set_to_json(cs: &ParamChartSet) -> String {
    format!(
        "{{\"chart_count\":{},\"total_faces\":{}}}",
        cs.charts.len(),
        total_chart_faces(cs)
    )
}

#[allow(dead_code)]
pub fn uv_utilization(cs: &ParamChartSet) -> f32 {
    if cs.charts.is_empty() {
        return 0.0;
    }
    // Simplified: ratio of charts with >0 faces
    let filled = cs
        .charts
        .iter()
        .filter(|c| !c.face_indices.is_empty())
        .count();
    filled as f32 / cs.charts.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_triangles() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [2.5, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 3, 4, 5];
        (pos, idx)
    }

    fn connected_quad() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn test_disconnected_gives_two_charts() {
        let (pos, idx) = two_triangles();
        let cs = build_param_charts(&pos, &idx);
        assert_eq!(chart_count(&cs), 2);
    }

    #[test]
    fn test_connected_gives_one_chart() {
        let (pos, idx) = connected_quad();
        let cs = build_param_charts(&pos, &idx);
        assert_eq!(chart_count(&cs), 1);
    }

    #[test]
    fn test_total_faces() {
        let (pos, idx) = two_triangles();
        let cs = build_param_charts(&pos, &idx);
        assert_eq!(total_chart_faces(&cs), 2);
    }

    #[test]
    fn test_empty_mesh() {
        let cs = build_param_charts(&[], &[]);
        assert_eq!(chart_count(&cs), 0);
    }

    #[test]
    fn test_largest_chart_some() {
        let (pos, idx) = connected_quad();
        let cs = build_param_charts(&pos, &idx);
        assert!(largest_chart(&cs).is_some());
    }

    #[test]
    fn test_uvs_in_range() {
        let (pos, idx) = connected_quad();
        let cs = build_param_charts(&pos, &idx);
        for c in &cs.charts {
            for &uv in &c.uvs {
                assert!((0.0..=1.0).contains(&uv[0]));
                assert!((0.0..=1.0).contains(&uv[1]));
            }
        }
    }

    #[test]
    fn test_to_json() {
        let (pos, idx) = connected_quad();
        let cs = build_param_charts(&pos, &idx);
        let j = chart_set_to_json(&cs);
        assert!(j.contains("chart_count"));
    }

    #[test]
    fn test_uv_utilization_positive() {
        let (pos, idx) = connected_quad();
        let cs = build_param_charts(&pos, &idx);
        assert!(uv_utilization(&cs) > 0.0);
    }

    #[test]
    fn test_uv_count_per_chart() {
        let (pos, idx) = connected_quad();
        let cs = build_param_charts(&pos, &idx);
        for c in &cs.charts {
            assert_eq!(c.uvs.len(), c.face_indices.len() * 3);
        }
    }

    #[test]
    fn test_empty_utilization_zero() {
        let cs = ParamChartSet { charts: vec![] };
        assert_eq!(uv_utilization(&cs), 0.0);
    }
}
