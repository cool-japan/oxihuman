#![allow(dead_code)]
//! Face flipping utilities.

use std::collections::HashMap;

/// Map from undirected edge key to face adjacency entries: (face_idx, directed_a, directed_b).
type EdgeFaceMap = HashMap<(u32, u32), Vec<(usize, u32, u32)>>;

/// Face flip tracker.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FaceFlip {
    pub flipped_indices: Vec<usize>,
}

/// Flip a single face (reverse winding order of a triangle).
#[allow(dead_code)]
pub fn flip_face_ff(tri: [u32; 3]) -> [u32; 3] {
    [tri[0], tri[2], tri[1]]
}

/// Flip all faces in a triangle list.
#[allow(dead_code)]
pub fn flip_all_faces_ff(tris: &[[u32; 3]]) -> Vec<[u32; 3]> {
    tris.iter().map(|t| flip_face_ff(*t)).collect()
}

/// Flip selected faces by index.
#[allow(dead_code)]
pub fn flip_selected_faces(tris: &mut [[u32; 3]], selection: &[usize]) {
    for &i in selection {
        if i < tris.len() {
            tris[i] = flip_face_ff(tris[i]);
        }
    }
}

/// Check if a face needs flipping based on expected normal direction.
#[allow(dead_code)]
pub fn needs_flip(positions: &[[f32; 3]], tri: [u32; 3], expected_normal: [f32; 3]) -> bool {
    let a = positions[tri[0] as usize];
    let b = positions[tri[1] as usize];
    let c = positions[tri[2] as usize];
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let dot = n[0] * expected_normal[0] + n[1] * expected_normal[1] + n[2] * expected_normal[2];
    dot < 0.0
}

/// Count how many faces have been flipped.
#[allow(dead_code)]
pub fn flip_count(ff: &FaceFlip) -> usize {
    ff.flipped_indices.len()
}

/// Make winding order consistent by BFS-propagating winding from face 0.
///
/// For each pair of adjacent faces sharing an undirected edge: if the shared
/// edge is traversed in the **same** directed sense in both faces, the
/// neighbour is flipped (consistent winding requires opposite traversal of
/// shared edges in adjacent faces). Disconnected components are each seeded
/// from their first unvisited face.
#[allow(dead_code)]
pub fn consistent_winding(tris: &[[u32; 3]]) -> Vec<[u32; 3]> {
    use std::collections::VecDeque;

    let n = tris.len();
    if n == 0 {
        return Vec::new();
    }

    // Build edge → list of (face_index, directed_edge (a,b)) for each undirected edge.
    // Undirected key: (min(a,b), max(a,b)).
    let mut edge_faces: EdgeFaceMap = EdgeFaceMap::new();
    for (fi, tri) in tris.iter().enumerate() {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push((fi, a, b));
        }
    }

    let mut result: Vec<[u32; 3]> = tris.to_vec();
    let mut visited = vec![false; n];
    let mut queue: VecDeque<usize> = VecDeque::new();

    let mut seed = 0usize;
    loop {
        // Find next unvisited face as a connected-component seed.
        while seed < n && visited[seed] {
            seed += 1;
        }
        if seed >= n {
            break;
        }
        visited[seed] = true;
        queue.push_back(seed);

        while let Some(fi) = queue.pop_front() {
            let tri = result[fi];
            for k in 0..3 {
                let a = tri[k];
                let b = tri[(k + 1) % 3];
                let key = if a < b { (a, b) } else { (b, a) };

                let entries = match edge_faces.get(&key) {
                    Some(e) => e.clone(),
                    None => continue,
                };

                // Find the directed edge for face fi in the original tris (not mutated
                // version) to keep the edge-face map consistent.
                // We look at the directed edge of fi as stored in result[fi].
                for (nfi, na, nb) in entries {
                    if nfi == fi || visited[nfi] {
                        continue;
                    }
                    // Check if neighbour nfi traverses this edge in the same direction as fi.
                    // fi's directed edge for this undirected edge: (a, b) as computed above.
                    // nfi's directed edge: (na, nb).
                    // Consistent winding = opposite directions, i.e. fi has (a,b) and nfi has (b,a).
                    // If nfi has the same direction (na == a and nb == b), flip it.
                    let same_direction = na == a && nb == b;
                    if same_direction {
                        result[nfi] = flip_face_ff(result[nfi]);
                        // Update edge_faces to reflect the flip so subsequent neighbours
                        // see the corrected directions.
                        for edge_k in 0..3 {
                            let ea = result[nfi][edge_k];
                            let eb = result[nfi][(edge_k + 1) % 3];
                            let ekey = if ea < eb { (ea, eb) } else { (eb, ea) };
                            if let Some(list) = edge_faces.get_mut(&ekey) {
                                for entry in list.iter_mut() {
                                    if entry.0 == nfi {
                                        entry.1 = ea;
                                        entry.2 = eb;
                                    }
                                }
                            }
                        }
                    }
                    visited[nfi] = true;
                    queue.push_back(nfi);
                }
            }
        }
    }

    result
}

/// Detect which faces are flipped relative to average normal.
#[allow(dead_code)]
pub fn detect_flipped(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> FaceFlip {
    let mut flipped = Vec::new();
    // Compute average normal
    let mut avg = [0.0_f32; 3];
    for tri in tris {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        avg[0] += e1[1] * e2[2] - e1[2] * e2[1];
        avg[1] += e1[2] * e2[0] - e1[0] * e2[2];
        avg[2] += e1[0] * e2[1] - e1[1] * e2[0];
    }
    for (i, tri) in tris.iter().enumerate() {
        if needs_flip(positions, *tri, avg) {
            flipped.push(i);
        }
    }
    FaceFlip {
        flipped_indices: flipped,
    }
}

/// Flip normals along with faces.
#[allow(dead_code)]
pub fn flip_normals_with_faces(
    normals: &mut [[f32; 3]],
    face_indices: &[usize],
    tris: &mut [[u32; 3]],
) {
    for &i in face_indices {
        if i < tris.len() {
            tris[i] = flip_face_ff(tris[i]);
            for idx in &tris[i] {
                let ni = *idx as usize;
                if ni < normals.len() {
                    normals[ni] = [-normals[ni][0], -normals[ni][1], -normals[ni][2]];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flip_face() {
        assert_eq!(flip_face_ff([0, 1, 2]), [0, 2, 1]);
    }

    #[test]
    fn test_flip_all() {
        let r = flip_all_faces_ff(&[[0, 1, 2], [3, 4, 5]]);
        assert_eq!(r, vec![[0, 2, 1], [3, 5, 4]]);
    }

    #[test]
    fn test_flip_selected() {
        let mut tris = [[0, 1, 2], [3, 4, 5]];
        flip_selected_faces(&mut tris, &[1]);
        assert_eq!(tris[1], [3, 5, 4]);
        assert_eq!(tris[0], [0, 1, 2]);
    }

    #[test]
    fn test_needs_flip() {
        let pos = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        assert!(!needs_flip(&pos, [0, 1, 2], [0.0, 0.0, 1.0]));
        assert!(needs_flip(&pos, [0, 1, 2], [0.0, 0.0, -1.0]));
    }

    #[test]
    fn test_flip_count() {
        let ff = FaceFlip {
            flipped_indices: vec![0, 2],
        };
        assert_eq!(flip_count(&ff), 2);
    }

    #[test]
    fn test_consistent_winding() {
        // A single triangle is trivially consistent — must come back unchanged.
        let tris = vec![[0u32, 1, 2]];
        let result = consistent_winding(&tris);
        assert_eq!(result.len(), 1);
        // After consistent winding the vertices must be a rotation of [0,1,2]
        // or a flip of it — for a single triangle it stays as-is.
        assert_eq!(result[0], [0, 1, 2]);
    }

    #[test]
    fn test_consistent_winding_flips_reversed() {
        // Two triangles sharing edge (1, 2):
        //   tri0: [0, 1, 2] — directed edge 1→2
        //   tri1: [3, 1, 2] — directed edge 1→2 (same direction as tri0, inconsistent)
        // After consistent_winding tri1 should be flipped so its shared
        // directed edge becomes 2→1, i.e. [3, 2, 1].
        let tris = vec![[0u32, 1, 2], [3, 1, 2]];
        let result = consistent_winding(&tris);
        assert_eq!(result.len(), 2);
        // tri0 is the seed — must remain [0, 1, 2]
        assert_eq!(result[0], [0, 1, 2]);
        // tri1's directed traversal of the shared undirected edge (1,2) must
        // now be 2→1, which means the triangle is [3, 2, 1].
        assert_eq!(result[1], [3, 2, 1]);
    }

    #[test]
    fn test_detect_flipped() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0, 1, 2]];
        let ff = detect_flipped(&pos, &tris);
        assert!(ff.flipped_indices.is_empty());
    }

    #[test]
    fn test_flip_normals_with_faces_empty() {
        let mut normals: Vec<[f32; 3]> = vec![];
        let mut tris: Vec<[u32; 3]> = vec![];
        flip_normals_with_faces(&mut normals, &[], &mut tris);
        assert!(tris.is_empty());
    }

    #[test]
    fn test_flip_face_identity() {
        let t = [5, 6, 7];
        let flipped = flip_face_ff(flip_face_ff(t));
        assert_eq!(flipped, t);
    }

    #[test]
    fn test_detect_flipped_multiple() {
        // Two faces with same winding: both contribute +Z normal.
        // Third face reversed: contributes -Z. Average is +Z so third is flipped.
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 1.0, 0.0],
            [5.0, 0.0, 0.0],
        ];
        let tris = vec![[0, 1, 2], [3, 4, 5], [6, 7, 8]];
        let ff = detect_flipped(&pos, &tris);
        assert_eq!(ff.flipped_indices.len(), 1);
    }
}
