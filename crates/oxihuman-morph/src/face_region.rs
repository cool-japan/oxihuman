#![allow(dead_code)]

/// A named face region defined by a set of vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceRegion {
    name: String,
    vertex_indices: Vec<u32>,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
}

/// A bitmask indicating which vertices belong to a region.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RegionMask {
    bits: Vec<bool>,
}

/// Create a new face region with the given name and vertex indices.
#[allow(dead_code)]
pub fn new_face_region(name: &str, vertex_indices: &[u32], bounds_min: [f32; 3], bounds_max: [f32; 3]) -> FaceRegion {
    FaceRegion {
        name: name.to_string(),
        vertex_indices: vertex_indices.to_vec(),
        bounds_min,
        bounds_max,
    }
}

/// Return the number of vertices in the region.
#[allow(dead_code)]
pub fn region_vertex_count(region: &FaceRegion) -> usize {
    region.vertex_indices.len()
}

/// Check whether a vertex index is contained in the region.
#[allow(dead_code)]
pub fn region_contains_vertex(region: &FaceRegion, idx: u32) -> bool {
    region.vertex_indices.contains(&idx)
}

/// Return the axis-aligned bounding box of the region as (min, max).
#[allow(dead_code)]
pub fn region_bounds(region: &FaceRegion) -> ([f32; 3], [f32; 3]) {
    (region.bounds_min, region.bounds_max)
}

/// Merge two face regions into one, combining their vertex sets.
#[allow(dead_code)]
pub fn merge_face_regions(a: &FaceRegion, b: &FaceRegion) -> FaceRegion {
    let mut indices = a.vertex_indices.clone();
    for &idx in &b.vertex_indices {
        if !indices.contains(&idx) {
            indices.push(idx);
        }
    }
    let min = [
        a.bounds_min[0].min(b.bounds_min[0]),
        a.bounds_min[1].min(b.bounds_min[1]),
        a.bounds_min[2].min(b.bounds_min[2]),
    ];
    let max = [
        a.bounds_max[0].max(b.bounds_max[0]),
        a.bounds_max[1].max(b.bounds_max[1]),
        a.bounds_max[2].max(b.bounds_max[2]),
    ];
    FaceRegion {
        name: format!("{}+{}", a.name, b.name),
        vertex_indices: indices,
        bounds_min: min,
        bounds_max: max,
    }
}

/// Convert a face region to a boolean mask for a given total vertex count.
#[allow(dead_code)]
pub fn region_to_mask(region: &FaceRegion, total_vertices: usize) -> RegionMask {
    let mut bits = vec![false; total_vertices];
    for &idx in &region.vertex_indices {
        if (idx as usize) < total_vertices {
            bits[idx as usize] = true;
        }
    }
    RegionMask { bits }
}

/// Compute the union of two region masks.
#[allow(dead_code)]
pub fn region_mask_union(a: &RegionMask, b: &RegionMask) -> RegionMask {
    let len = a.bits.len().max(b.bits.len());
    let mut bits = vec![false; len];
    for i in 0..len {
        let va = a.bits.get(i).copied().unwrap_or(false);
        let vb = b.bits.get(i).copied().unwrap_or(false);
        bits[i] = va || vb;
    }
    RegionMask { bits }
}

/// Compute the intersection of two region masks.
#[allow(dead_code)]
pub fn region_mask_intersect(a: &RegionMask, b: &RegionMask) -> RegionMask {
    let len = a.bits.len().min(b.bits.len());
    let mut bits = vec![false; len];
    for i in 0..len {
        bits[i] = a.bits[i] && b.bits[i];
    }
    RegionMask { bits }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_region() -> FaceRegion {
        new_face_region("nose", &[0, 1, 2, 5], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0])
    }

    #[test]
    fn test_new_face_region() {
        let r = sample_region();
        assert_eq!(r.name, "nose");
        assert_eq!(r.vertex_indices.len(), 4);
    }

    #[test]
    fn test_region_vertex_count() {
        let r = sample_region();
        assert_eq!(region_vertex_count(&r), 4);
    }

    #[test]
    fn test_region_contains_vertex() {
        let r = sample_region();
        assert!(region_contains_vertex(&r, 2));
        assert!(!region_contains_vertex(&r, 3));
    }

    #[test]
    fn test_region_bounds() {
        let r = sample_region();
        let (mn, mx) = region_bounds(&r);
        assert_eq!(mn, [0.0, 0.0, 0.0]);
        assert_eq!(mx, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_merge_face_regions() {
        let a = new_face_region("a", &[0, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = new_face_region("b", &[1, 2], [-1.0, 0.0, 0.0], [2.0, 1.0, 1.0]);
        let m = merge_face_regions(&a, &b);
        assert_eq!(region_vertex_count(&m), 3);
        assert_eq!(m.bounds_min[0], -1.0);
        assert_eq!(m.bounds_max[0], 2.0);
    }

    #[test]
    fn test_region_to_mask() {
        let r = new_face_region("r", &[1, 3], [0.0; 3], [1.0; 3]);
        let mask = region_to_mask(&r, 5);
        assert!(!mask.bits[0]);
        assert!(mask.bits[1]);
        assert!(!mask.bits[2]);
        assert!(mask.bits[3]);
    }

    #[test]
    fn test_region_mask_union() {
        let a = RegionMask { bits: vec![true, false, true] };
        let b = RegionMask { bits: vec![false, true, false] };
        let u = region_mask_union(&a, &b);
        assert!(u.bits[0] && u.bits[1] && u.bits[2]);
    }

    #[test]
    fn test_region_mask_intersect() {
        let a = RegionMask { bits: vec![true, false, true] };
        let b = RegionMask { bits: vec![true, true, false] };
        let i = region_mask_intersect(&a, &b);
        assert!(i.bits[0]);
        assert!(!i.bits[1]);
        assert!(!i.bits[2]);
    }

    #[test]
    fn test_mask_union_different_lengths() {
        let a = RegionMask { bits: vec![true] };
        let b = RegionMask { bits: vec![false, true, true] };
        let u = region_mask_union(&a, &b);
        assert_eq!(u.bits.len(), 3);
        assert!(u.bits[0]);
    }

    #[test]
    fn test_empty_region() {
        let r = new_face_region("empty", &[], [0.0; 3], [0.0; 3]);
        assert_eq!(region_vertex_count(&r), 0);
    }
}
