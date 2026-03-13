// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A silhouette edge.
#[allow(dead_code)]
#[derive(Clone)]
pub struct SilhouetteEdge {
    pub a: u32,
    pub b: u32,
}

/// Result of silhouette extraction.
#[allow(dead_code)]
pub struct SilhouetteResult {
    pub edges: Vec<SilhouetteEdge>,
    pub view_dir: [f32; 3],
}

fn face_normal_sil(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-9 {
        return [0.0; 3];
    }
    [n[0] / len, n[1] / len, n[2] / len]
}

fn dot3_sil(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Extract silhouette edges for a given view direction.
#[allow(dead_code)]
pub fn extract_silhouette(
    positions: &[[f32; 3]],
    indices: &[u32],
    view_dir: [f32; 3],
) -> SilhouetteResult {
    let nf = indices.len() / 3;
    let mut face_normals: Vec<[f32; 3]> = Vec::with_capacity(nf);
    for fi in 0..nf {
        let a = positions[indices[fi * 3] as usize];
        let b = positions[indices[fi * 3 + 1] as usize];
        let c = positions[indices[fi * 3 + 2] as usize];
        face_normals.push(face_normal_sil(a, b, c));
    }

    // Build edge-face map
    let mut edge_faces: std::collections::HashMap<(u32, u32), Vec<usize>> =
        std::collections::HashMap::new();
    for fi in 0..nf {
        let [a, b, c] = [indices[fi * 3], indices[fi * 3 + 1], indices[fi * 3 + 2]];
        for (u, v) in [(a, b), (b, c), (c, a)] {
            let key = if u < v { (u, v) } else { (v, u) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut silhouette = Vec::new();
    for ((ea, eb), faces) in &edge_faces {
        if faces.len() == 2 {
            let d0 = dot3_sil(face_normals[faces[0]], view_dir);
            let d1 = dot3_sil(face_normals[faces[1]], view_dir);
            if d0 * d1 < 0.0 {
                silhouette.push(SilhouetteEdge { a: *ea, b: *eb });
            }
        } else if faces.len() == 1 {
            // Boundary edges are always silhouette
            silhouette.push(SilhouetteEdge { a: *ea, b: *eb });
        }
    }

    SilhouetteResult {
        edges: silhouette,
        view_dir,
    }
}

/// Count silhouette edges.
#[allow(dead_code)]
pub fn silhouette_edge_count(result: &SilhouetteResult) -> usize {
    result.edges.len()
}

/// Check if any silhouette edges exist.
#[allow(dead_code)]
pub fn has_silhouette(result: &SilhouetteResult) -> bool {
    !result.edges.is_empty()
}

/// Collect unique silhouette vertex indices.
#[allow(dead_code)]
pub fn silhouette_vertices(result: &SilhouetteResult) -> Vec<u32> {
    let mut v: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for e in &result.edges {
        v.insert(e.a);
        v.insert(e.b);
    }
    let mut out: Vec<_> = v.into_iter().collect();
    out.sort();
    out
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn silhouette_to_json(result: &SilhouetteResult) -> String {
    format!(
        r#"{{"silhouette_edges":{},"view_dir":[{:.3},{:.3},{:.3}]}}"#,
        result.edges.len(),
        result.view_dir[0],
        result.view_dir[1],
        result.view_dir[2]
    )
}

/// Build a simple view direction from camera position.
#[allow(dead_code)]
pub fn view_dir_from_camera(cam: [f32; 3], target: [f32; 3]) -> [f32; 3] {
    let d = [target[0] - cam[0], target[1] - cam[1], target[2] - cam[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len < 1e-9 {
        return [0.0, 0.0, -1.0];
    }
    [d[0] / len, d[1] / len, d[2] / len]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        (pos, vec![0_u32, 1, 2])
    }

    #[test]
    fn single_tri_has_boundary_silhouette() {
        let (pos, idx) = simple_mesh();
        let r = extract_silhouette(&pos, &idx, [0.0, 0.0, 1.0]);
        assert!(has_silhouette(&r));
    }

    #[test]
    fn silhouette_edges_positive() {
        let (pos, idx) = simple_mesh();
        let r = extract_silhouette(&pos, &idx, [0.0, 0.0, 1.0]);
        assert!(silhouette_edge_count(&r) > 0);
    }

    #[test]
    fn silhouette_vertices_nonempty() {
        let (pos, idx) = simple_mesh();
        let r = extract_silhouette(&pos, &idx, [0.0, 0.0, 1.0]);
        assert!(!silhouette_vertices(&r).is_empty());
    }

    #[test]
    fn json_has_edges() {
        let r = SilhouetteResult {
            edges: vec![SilhouetteEdge { a: 0, b: 1 }],
            view_dir: [0.0, 0.0, 1.0],
        };
        let j = silhouette_to_json(&r);
        assert!(j.contains("\"silhouette_edges\":1"));
    }

    #[test]
    fn view_dir_normalized() {
        let v = view_dir_from_camera([0.0, 0.0, 5.0], [0.0, 0.0, 0.0]);
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn empty_mesh_no_silhouette() {
        let r = extract_silhouette(&[], &[], [0.0, 0.0, 1.0]);
        assert!(!has_silhouette(&r));
    }

    #[test]
    fn view_dir_zero_camera() {
        let v = view_dir_from_camera([0.0; 3], [0.0; 3]);
        assert_eq!(v, [0.0, 0.0, -1.0]);
    }

    #[test]
    fn count_matches_len() {
        let r = SilhouetteResult {
            edges: vec![SilhouetteEdge { a: 0, b: 1 }, SilhouetteEdge { a: 1, b: 2 }],
            view_dir: [0.0, 0.0, 1.0],
        };
        assert_eq!(silhouette_edge_count(&r), 2);
    }

    #[test]
    fn has_silhouette_empty() {
        let r = SilhouetteResult {
            edges: vec![],
            view_dir: [0.0, 0.0, 1.0],
        };
        assert!(!has_silhouette(&r));
    }

    #[test]
    fn vertices_unique_sorted() {
        let r = SilhouetteResult {
            edges: vec![SilhouetteEdge { a: 2, b: 0 }, SilhouetteEdge { a: 0, b: 1 }],
            view_dir: [0.0, 0.0, 1.0],
        };
        let v = silhouette_vertices(&r);
        assert_eq!(v, vec![0, 1, 2]);
    }
}
