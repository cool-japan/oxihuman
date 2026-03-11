//! UV seam stitching — merging UV islands across seams.

#[allow(dead_code)]
pub struct UvSeam {
    pub edge_a: [u32; 2], // vertex indices on mesh side A
    pub edge_b: [u32; 2], // vertex indices on mesh side B
}

#[allow(dead_code)]
pub struct StitchResult {
    pub merged_uvs: Vec<[f32; 2]>,
    pub remap: Vec<usize>, // old UV index -> new UV index
    pub stitch_count: usize,
}

/// UV island (S suffix to avoid collision with existing UvIsland).
#[allow(dead_code)]
pub struct UvIslandS {
    pub uv_indices: Vec<usize>,
    pub aabb_min: [f32; 2],
    pub aabb_max: [f32; 2],
}

fn uv_dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).sqrt()
}

/// Find UV seams by detecting edges where 3D positions match but UVs differ.
#[allow(dead_code)]
pub fn find_uv_seams(positions: &[[f32; 3]], uvs: &[[f32; 2]], indices: &[u32]) -> Vec<UvSeam> {
    let mut seams = Vec::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        for t2 in (t + 1)..tri_count {
            for &(ea, eb) in &[
                ([t * 3, t * 3 + 1], [t2 * 3, t2 * 3 + 1]),
                ([t * 3 + 1, t * 3 + 2], [t2 * 3, t2 * 3 + 1]),
                ([t * 3, t * 3 + 2], [t2 * 3 + 1, t2 * 3 + 2]),
            ] {
                let va0 = indices[ea[0]] as usize;
                let va1 = indices[ea[1]] as usize;
                let vb0 = indices[eb[0]] as usize;
                let vb1 = indices[eb[1]] as usize;

                if va0 < positions.len()
                    && va1 < positions.len()
                    && vb0 < positions.len()
                    && vb1 < positions.len()
                {
                    let same_pos = (positions[va0][0] - positions[vb0][0]).abs() < 1e-5
                        && (positions[va0][1] - positions[vb0][1]).abs() < 1e-5
                        && (positions[va0][2] - positions[vb0][2]).abs() < 1e-5;

                    if same_pos
                        && va0 < uvs.len()
                        && va1 < uvs.len()
                        && vb0 < uvs.len()
                        && vb1 < uvs.len()
                        && uv_dist(uvs[va0], uvs[vb0]) > 1e-4
                    {
                        seams.push(UvSeam {
                            edge_a: [va0 as u32, va1 as u32],
                            edge_b: [vb0 as u32, vb1 as u32],
                        });
                    }
                }
            }
        }
    }
    seams
}

/// Return the average UV positions for each endpoint of the seam.
#[allow(dead_code)]
pub fn stitch_seam(uvs: &[[f32; 2]], seam: &UvSeam) -> ([f32; 2], [f32; 2]) {
    let a0 = seam.edge_a[0] as usize;
    let a1 = seam.edge_a[1] as usize;
    let b0 = seam.edge_b[0] as usize;
    let b1 = seam.edge_b[1] as usize;

    let p0 = if a0 < uvs.len() && b0 < uvs.len() {
        [
            (uvs[a0][0] + uvs[b0][0]) * 0.5,
            (uvs[a0][1] + uvs[b0][1]) * 0.5,
        ]
    } else if a0 < uvs.len() {
        uvs[a0]
    } else {
        [0.0, 0.0]
    };

    let p1 = if a1 < uvs.len() && b1 < uvs.len() {
        [
            (uvs[a1][0] + uvs[b1][0]) * 0.5,
            (uvs[a1][1] + uvs[b1][1]) * 0.5,
        ]
    } else if a1 < uvs.len() {
        uvs[a1]
    } else {
        [0.0, 0.0]
    };

    (p0, p1)
}

/// Stitch all seams, merging UV coordinates and producing remap.
#[allow(dead_code)]
pub fn stitch_all_seams(uvs: &mut [[f32; 2]], seams: &[UvSeam]) -> StitchResult {
    let n = uvs.len();
    let mut remap: Vec<usize> = (0..n).collect();
    let mut stitch_count = 0;

    for seam in seams {
        let a0 = seam.edge_a[0] as usize;
        let a1 = seam.edge_a[1] as usize;
        let b0 = seam.edge_b[0] as usize;
        let b1 = seam.edge_b[1] as usize;

        if a0 < n && b0 < n {
            let merged = [
                (uvs[a0][0] + uvs[b0][0]) * 0.5,
                (uvs[a0][1] + uvs[b0][1]) * 0.5,
            ];
            uvs[a0] = merged;
            uvs[b0] = merged;
            remap[b0] = remap[a0];
            stitch_count += 1;
        }
        if a1 < n && b1 < n {
            let merged = [
                (uvs[a1][0] + uvs[b1][0]) * 0.5,
                (uvs[a1][1] + uvs[b1][1]) * 0.5,
            ];
            uvs[a1] = merged;
            uvs[b1] = merged;
            remap[b1] = remap[a1];
            stitch_count += 1;
        }
    }

    StitchResult {
        merged_uvs: uvs.to_owned(),
        remap,
        stitch_count,
    }
}

/// Length of the seam edge in UV space.
#[allow(dead_code)]
pub fn uv_seam_length(uvs: &[[f32; 2]], seam: &UvSeam) -> f32 {
    let a0 = seam.edge_a[0] as usize;
    let a1 = seam.edge_a[1] as usize;
    if a0 < uvs.len() && a1 < uvs.len() {
        uv_dist(uvs[a0], uvs[a1])
    } else {
        0.0
    }
}

/// Detect UV islands using connected components on the triangle index buffer.
#[allow(dead_code)]
pub fn detect_uv_islands_stitched(uvs: &[[f32; 2]], indices: &[u32]) -> Vec<UvIslandS> {
    let n = uvs.len();
    let mut label = vec![usize::MAX; n];
    let tri_count = indices.len() / 3;
    let mut island_id = 0;

    for t in 0..tri_count {
        let a = indices[t * 3] as usize;
        let b = indices[t * 3 + 1] as usize;
        let c = indices[t * 3 + 2] as usize;
        if a >= n || b >= n || c >= n {
            continue;
        }
        let existing = [label[a], label[b], label[c]]
            .into_iter()
            .filter(|&l| l != usize::MAX)
            .min();
        let id = existing.unwrap_or_else(|| {
            let id = island_id;
            island_id += 1;
            id
        });
        label[a] = id;
        label[b] = id;
        label[c] = id;
    }

    if island_id == 0 {
        return Vec::new();
    }

    let mut islands: Vec<Vec<usize>> = vec![Vec::new(); island_id];
    for (i, &l) in label.iter().enumerate() {
        if l != usize::MAX && l < island_id {
            islands[l].push(i);
        }
    }

    islands
        .into_iter()
        .filter(|idxs| !idxs.is_empty())
        .map(|uv_indices| {
            let mut mn = [f32::INFINITY; 2];
            let mut mx = [f32::NEG_INFINITY; 2];
            for &i in &uv_indices {
                mn[0] = mn[0].min(uvs[i][0]);
                mn[1] = mn[1].min(uvs[i][1]);
                mx[0] = mx[0].max(uvs[i][0]);
                mx[1] = mx[1].max(uvs[i][1]);
            }
            UvIslandS {
                uv_indices,
                aabb_min: mn,
                aabb_max: mx,
            }
        })
        .collect()
}

/// Approximate area of a UV island (bounding box area).
#[allow(dead_code)]
pub fn island_area_s(island: &UvIslandS, _uvs: &[[f32; 2]]) -> f32 {
    let w = (island.aabb_max[0] - island.aabb_min[0]).max(0.0);
    let h = (island.aabb_max[1] - island.aabb_min[1]).max(0.0);
    w * h
}

/// Simple island packing: stack islands vertically in `[0,1]` space.
#[allow(dead_code)]
pub fn pack_islands_simple(islands: &mut [UvIslandS], uvs: &mut [[f32; 2]]) {
    let mut cursor_y = 0.0f32;
    for island in islands.iter_mut() {
        let offset_x = -island.aabb_min[0];
        let offset_y = cursor_y - island.aabb_min[1];
        let height = (island.aabb_max[1] - island.aabb_min[1]).max(0.0);

        for &i in &island.uv_indices {
            if i < uvs.len() {
                uvs[i][0] += offset_x;
                uvs[i][1] += offset_y;
            }
        }
        island.aabb_max[0] += offset_x;
        island.aabb_min[0] = 0.0;
        island.aabb_max[1] += offset_y;
        island.aabb_min[1] = cursor_y;
        cursor_y += height + 0.01;
    }
}

/// L2 distance between original and stitched UV coordinates.
#[allow(dead_code)]
pub fn uv_stretch(original: [f32; 2], stitched: [f32; 2]) -> f32 {
    uv_dist(original, stitched)
}

/// Number of seams.
#[allow(dead_code)]
pub fn stitch_seam_count(seams: &[UvSeam]) -> usize {
    seams.len()
}

/// Find all boundary (unshared) UV edges.
#[allow(dead_code)]
pub fn boundary_uv_edges(uvs: &[[f32; 2]], indices: &[u32]) -> Vec<[usize; 2]> {
    use std::collections::HashMap;
    let n = uvs.len();
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[t * 3] as usize;
        let b = indices[t * 3 + 1] as usize;
        let c = indices[t * 3 + 2] as usize;
        if a >= n || b >= n || c >= n {
            continue;
        }
        for (u, v) in [
            (a.min(b), a.max(b)),
            (b.min(c), b.max(c)),
            (a.min(c), a.max(c)),
        ] {
            *edge_count.entry((u, v)).or_insert(0) += 1;
        }
    }
    edge_count
        .into_iter()
        .filter(|&(_, count)| count == 1)
        .map(|((u, v), _)| [u, v])
        .collect()
}

/// Mirror UV coordinates horizontally: U = 1 - U.
#[allow(dead_code)]
pub fn mirror_uvs_horizontal(uvs: &mut [[f32; 2]]) {
    for uv in uvs.iter_mut() {
        uv[0] = 1.0 - uv[0];
    }
}

/// Mirror UV coordinates vertically: V = 1 - V.
#[allow(dead_code)]
pub fn mirror_uvs_vertical(uvs: &mut [[f32; 2]]) {
    for uv in uvs.iter_mut() {
        uv[1] = 1.0 - uv[1];
    }
}

/// Clamp all UV coordinates to [0, 1].
#[allow(dead_code)]
pub fn clamp_uvs(uvs: &mut [[f32; 2]]) {
    for uv in uvs.iter_mut() {
        uv[0] = uv[0].clamp(0.0, 1.0);
        uv[1] = uv[1].clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_uvs() -> Vec<[f32; 2]> {
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    }

    fn simple_indices() -> Vec<u32> {
        vec![0, 1, 2, 0, 2, 3]
    }

    #[test]
    fn test_stitch_seam_count_zero() {
        let seams: Vec<UvSeam> = Vec::new();
        assert_eq!(stitch_seam_count(&seams), 0);
    }

    #[test]
    fn test_stitch_seam_count_nonzero() {
        let seams = vec![
            UvSeam {
                edge_a: [0, 1],
                edge_b: [2, 3],
            },
            UvSeam {
                edge_a: [1, 2],
                edge_b: [3, 0],
            },
        ];
        assert_eq!(stitch_seam_count(&seams), 2);
    }

    #[test]
    fn test_uv_seam_length() {
        let uvs = simple_uvs();
        let seam = UvSeam {
            edge_a: [0, 1],
            edge_b: [2, 3],
        };
        let len = uv_seam_length(&uvs, &seam);
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_uv_seam_length_out_of_bounds() {
        let uvs = simple_uvs();
        let seam = UvSeam {
            edge_a: [100, 101],
            edge_b: [2, 3],
        };
        let len = uv_seam_length(&uvs, &seam);
        assert!((len).abs() < 1e-6);
    }

    #[test]
    fn test_stitch_seam_averages() {
        let uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.2, 0.0], [1.0, 0.0]];
        let seam = UvSeam {
            edge_a: [0, 1],
            edge_b: [2, 3],
        };
        let (p0, _p1) = stitch_seam(&uvs, &seam);
        assert!((p0[0] - 0.1).abs() < 1e-5);
    }

    #[test]
    fn test_stitch_all_seams_remap() {
        let mut uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.2, 0.0], [1.0, 0.0]];
        let seams = vec![UvSeam {
            edge_a: [0, 1],
            edge_b: [2, 3],
        }];
        let result = stitch_all_seams(&mut uvs, &seams);
        assert_eq!(result.remap.len(), 4);
        assert_eq!(result.remap[2], result.remap[0]);
    }

    #[test]
    fn test_stitch_all_seams_count() {
        let mut uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.2, 0.0], [1.0, 0.0]];
        let seams = vec![UvSeam {
            edge_a: [0, 1],
            edge_b: [2, 3],
        }];
        let result = stitch_all_seams(&mut uvs, &seams);
        assert!(result.stitch_count >= 1);
    }

    #[test]
    fn test_mirror_uvs_horizontal() {
        let mut uvs = vec![[0.0f32, 0.5], [1.0, 0.5], [0.25, 0.5]];
        mirror_uvs_horizontal(&mut uvs);
        assert!((uvs[0][0] - 1.0).abs() < 1e-6);
        assert!((uvs[1][0]).abs() < 1e-6);
        assert!((uvs[2][0] - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_mirror_uvs_vertical() {
        let mut uvs = vec![[0.5f32, 0.0], [0.5, 1.0], [0.5, 0.25]];
        mirror_uvs_vertical(&mut uvs);
        assert!((uvs[0][1] - 1.0).abs() < 1e-6);
        assert!((uvs[1][1]).abs() < 1e-6);
        assert!((uvs[2][1] - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_uvs() {
        let mut uvs = vec![[-0.5f32, 1.5], [0.5, 0.5], [2.0, -1.0]];
        clamp_uvs(&mut uvs);
        assert!((uvs[0][0]).abs() < 1e-6);
        assert!((uvs[0][1] - 1.0).abs() < 1e-6);
        assert!((uvs[2][0] - 1.0).abs() < 1e-6);
        assert!((uvs[2][1]).abs() < 1e-6);
    }

    #[test]
    fn test_detect_uv_islands_stitched_basic() {
        let uvs = simple_uvs();
        let indices = simple_indices();
        let islands = detect_uv_islands_stitched(&uvs, &indices);
        assert!(!islands.is_empty());
    }

    #[test]
    fn test_detect_uv_islands_stitched_empty() {
        let uvs: Vec<[f32; 2]> = Vec::new();
        let indices: Vec<u32> = Vec::new();
        let islands = detect_uv_islands_stitched(&uvs, &indices);
        assert!(islands.is_empty());
    }

    #[test]
    fn test_island_area_s_basic() {
        let uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let island = UvIslandS {
            uv_indices: vec![0, 1, 2, 3],
            aabb_min: [0.0, 0.0],
            aabb_max: [1.0, 1.0],
        };
        let area = island_area_s(&island, &uvs);
        assert!((area - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_boundary_uv_edges() {
        let uvs = simple_uvs();
        let indices = simple_indices();
        let edges = boundary_uv_edges(&uvs, &indices);
        // A square mesh has 4 boundary edges
        assert!(!edges.is_empty());
    }

    #[test]
    fn test_uv_stretch_same() {
        let stretch = uv_stretch([0.5, 0.5], [0.5, 0.5]);
        assert!(stretch.abs() < 1e-6);
    }

    #[test]
    fn test_uv_stretch_diff() {
        let stretch = uv_stretch([0.0, 0.0], [1.0, 0.0]);
        assert!((stretch - 1.0).abs() < 1e-5);
    }
}
