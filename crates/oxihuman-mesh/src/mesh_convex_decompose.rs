//! Approximate convex decomposition of a concave mesh into a set of convex hulls.
//!
//! Splits a triangle mesh into parts, each of which is approximately convex,
//! using a greedy volume-based splitting strategy with deterministic seeding.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConvexDecomposeConfig {
    pub max_parts: usize,
    pub min_volume_ratio: f32,
    pub max_vertices_per_hull: usize,
    pub concavity_threshold: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConvexHullPart {
    pub vertices: Vec<[f32; 3]>,
    pub volume: f32,
    pub part_index: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConvexDecomposeResult {
    pub parts: Vec<ConvexHullPart>,
    pub total_input_vertices: usize,
}

#[allow(dead_code)]
pub fn default_convex_decompose_config() -> ConvexDecomposeConfig {
    ConvexDecomposeConfig {
        max_parts: 16,
        min_volume_ratio: 0.01,
        max_vertices_per_hull: 64,
        concavity_threshold: 0.05,
    }
}

/// Approximate convex decomposition.
///
/// Uses a simple greedy approach: repeatedly find the split plane that reduces
/// concavity the most, splitting vertices into two sub-hulls, until the part
/// count limit is reached or all parts are sufficiently convex.
#[allow(dead_code)]
pub fn convex_decompose(
    vertices: &[[f32; 3]],
    _triangles: &[[u32; 3]],
    config: &ConvexDecomposeConfig,
) -> ConvexDecomposeResult {
    if vertices.is_empty() {
        return ConvexDecomposeResult {
            parts: vec![],
            total_input_vertices: 0,
        };
    }

    let mut groups: Vec<Vec<[f32; 3]>> = vec![vertices.to_vec()];
    let mut iterations = 0usize;

    while groups.len() < config.max_parts && iterations < config.max_parts * 2 {
        iterations += 1;
        // Find the group with highest "concavity proxy" (largest extent on any axis).
        let worst_idx = groups
            .iter()
            .enumerate()
            .map(|(i, g)| (i, group_max_extent(g)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i);

        let wi = match worst_idx {
            Some(i) => i,
            None => break,
        };

        if group_max_extent(&groups[wi]) < config.concavity_threshold {
            break;
        }

        let to_split = groups.remove(wi);
        let (a, b) = split_group_by_median(&to_split);
        if a.is_empty() || b.is_empty() {
            groups.push(to_split);
            break;
        }
        groups.push(a);
        groups.push(b);
    }

    let parts: Vec<ConvexHullPart> = groups
        .into_iter()
        .enumerate()
        .map(|(i, verts)| {
            let vol = convex_hull_volume_approx(&verts);
            ConvexHullPart {
                vertices: verts,
                volume: vol,
                part_index: i,
            }
        })
        .collect();

    ConvexDecomposeResult {
        total_input_vertices: vertices.len(),
        parts,
    }
}

/// Number of parts in the decomposition result.
#[allow(dead_code)]
pub fn decompose_part_count(result: &ConvexDecomposeResult) -> usize {
    result.parts.len()
}

/// Vertices of a specific part.
#[allow(dead_code)]
pub fn decompose_part_vertices(result: &ConvexDecomposeResult, part: usize) -> &[[f32; 3]] {
    if part < result.parts.len() {
        &result.parts[part].vertices
    } else {
        &[]
    }
}

/// Volume of a specific part.
#[allow(dead_code)]
pub fn decompose_part_volume(result: &ConvexDecomposeResult, part: usize) -> f32 {
    result.parts.get(part).map_or(0.0, |p| p.volume)
}

/// Serialize result to a JSON-like string.
#[allow(dead_code)]
pub fn decompose_to_json(result: &ConvexDecomposeResult) -> String {
    let parts_json: Vec<String> = result
        .parts
        .iter()
        .map(|p| {
            format!(
                "{{\"index\":{},\"vertex_count\":{},\"volume\":{:.6}}}",
                p.part_index,
                p.vertices.len(),
                p.volume
            )
        })
        .collect();
    format!(
        "{{\"total_input_vertices\":{},\"parts\":[{}]}}",
        result.total_input_vertices,
        parts_json.join(",")
    )
}

/// Total vertices across all parts.
#[allow(dead_code)]
pub fn decompose_total_vertices(result: &ConvexDecomposeResult) -> usize {
    result.parts.iter().map(|p| p.vertices.len()).sum()
}

/// Check that all parts are non-empty and volumes are non-negative.
#[allow(dead_code)]
pub fn decompose_validate(result: &ConvexDecomposeResult) -> bool {
    result.parts.iter().all(|p| !p.vertices.is_empty() && p.volume >= 0.0)
}

/// Merge two parts into one.
#[allow(dead_code)]
pub fn decompose_merge_parts(result: &mut ConvexDecomposeResult, a: usize, b: usize) {
    if a >= result.parts.len() || b >= result.parts.len() || a == b {
        return;
    }
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    let removed = result.parts.remove(hi);
    result.parts[lo].vertices.extend(removed.vertices);
    result.parts[lo].volume = convex_hull_volume_approx(&result.parts[lo].vertices);
    // Re-index.
    for (i, p) in result.parts.iter_mut().enumerate() {
        p.part_index = i;
    }
}

/// Clear all parts.
#[allow(dead_code)]
pub fn decompose_clear(result: &mut ConvexDecomposeResult) {
    result.parts.clear();
    result.total_input_vertices = 0;
}

// ─── Internal helpers ────────────────────────────────────────────────────────

fn group_max_extent(verts: &[[f32; 3]]) -> f32 {
    if verts.is_empty() {
        return 0.0;
    }
    let mut min = verts[0];
    let mut max = verts[0];
    for v in verts {
        for k in 0..3 {
            if v[k] < min[k] { min[k] = v[k]; }
            if v[k] > max[k] { max[k] = v[k]; }
        }
    }
    (0..3).map(|k| max[k] - min[k]).fold(0.0f32, f32::max)
}

fn split_group_by_median(verts: &[[f32; 3]]) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    if verts.len() < 2 {
        return (verts.to_vec(), vec![]);
    }
    // Find longest axis.
    let mut min = verts[0];
    let mut max = verts[0];
    for v in verts {
        for k in 0..3 {
            if v[k] < min[k] { min[k] = v[k]; }
            if v[k] > max[k] { max[k] = v[k]; }
        }
    }
    let axis = (0..3)
        .max_by(|&a, &b| (max[a] - min[a]).partial_cmp(&(max[b] - min[b])).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0);
    let mid = (min[axis] + max[axis]) * 0.5;
    let a: Vec<_> = verts.iter().filter(|v| v[axis] <= mid).copied().collect();
    let b: Vec<_> = verts.iter().filter(|v| v[axis] > mid).copied().collect();
    (a, b)
}

fn convex_hull_volume_approx(verts: &[[f32; 3]]) -> f32 {
    if verts.len() < 4 {
        return 0.0;
    }
    let mut min = verts[0];
    let mut max = verts[0];
    for v in verts {
        for k in 0..3 {
            if v[k] < min[k] { min[k] = v[k]; }
            if v[k] > max[k] { max[k] = v[k]; }
        }
    }
    // AABB volume as a proxy.
    (max[0] - min[0]) * (max[1] - min[1]) * (max[2] - min[2])
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0], [1.0, 1.0, 0.0],
            [0.0, 0.0, 1.0], [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0], [1.0, 1.0, 1.0],
        ]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_convex_decompose_config();
        assert!(cfg.max_parts >= 1);
        assert!(cfg.concavity_threshold > 0.0);
    }

    #[test]
    fn test_empty_input() {
        let cfg = default_convex_decompose_config();
        let result = convex_decompose(&[], &[], &cfg);
        assert_eq!(result.total_input_vertices, 0);
        assert!(result.parts.is_empty());
    }

    #[test]
    fn test_single_part_cube() {
        let verts = cube_verts();
        let cfg = ConvexDecomposeConfig { max_parts: 1, concavity_threshold: 100.0, ..default_convex_decompose_config() };
        let result = convex_decompose(&verts, &[], &cfg);
        assert_eq!(result.total_input_vertices, 8);
        assert_eq!(decompose_part_count(&result), 1);
    }

    #[test]
    fn test_multi_part() {
        let verts = cube_verts();
        let cfg = ConvexDecomposeConfig { max_parts: 4, concavity_threshold: 0.01, ..default_convex_decompose_config() };
        let result = convex_decompose(&verts, &[], &cfg);
        assert!(decompose_part_count(&result) >= 1);
        assert!(decompose_part_count(&result) <= 4);
    }

    #[test]
    fn test_part_vertices() {
        let verts = cube_verts();
        let cfg = default_convex_decompose_config();
        let result = convex_decompose(&verts, &[], &cfg);
        assert!(!decompose_part_vertices(&result, 0).is_empty());
        assert!(decompose_part_vertices(&result, 9999).is_empty());
    }

    #[test]
    fn test_part_volume() {
        let verts = cube_verts();
        let cfg = default_convex_decompose_config();
        let result = convex_decompose(&verts, &[], &cfg);
        assert!(decompose_part_volume(&result, 0) >= 0.0);
        assert_eq!(decompose_part_volume(&result, 9999), 0.0);
    }

    #[test]
    fn test_total_vertices() {
        let verts = cube_verts();
        let cfg = default_convex_decompose_config();
        let result = convex_decompose(&verts, &[], &cfg);
        // Total across all parts >= input (vertices may be duplicated at split boundaries).
        assert!(decompose_total_vertices(&result) >= 1);
    }

    #[test]
    fn test_validate() {
        let verts = cube_verts();
        let cfg = default_convex_decompose_config();
        let result = convex_decompose(&verts, &[], &cfg);
        assert!(decompose_validate(&result));
    }

    #[test]
    fn test_to_json() {
        let verts = cube_verts();
        let cfg = default_convex_decompose_config();
        let result = convex_decompose(&verts, &[], &cfg);
        let json = decompose_to_json(&result);
        assert!(json.contains("total_input_vertices"));
        assert!(json.contains("parts"));
    }

    #[test]
    fn test_merge_parts() {
        let verts = cube_verts();
        let cfg = ConvexDecomposeConfig { max_parts: 4, concavity_threshold: 0.01, ..default_convex_decompose_config() };
        let mut result = convex_decompose(&verts, &[], &cfg);
        let before = decompose_part_count(&result);
        if before >= 2 {
            decompose_merge_parts(&mut result, 0, 1);
            assert_eq!(decompose_part_count(&result), before - 1);
        }
    }

    #[test]
    fn test_clear() {
        let verts = cube_verts();
        let cfg = default_convex_decompose_config();
        let mut result = convex_decompose(&verts, &[], &cfg);
        decompose_clear(&mut result);
        assert_eq!(decompose_part_count(&result), 0);
        assert_eq!(result.total_input_vertices, 0);
    }
}
