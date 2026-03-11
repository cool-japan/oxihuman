#![allow(dead_code)]
//! Compute triangle and mesh surface areas.

/// Result of an area computation.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AreaResult {
    pub total_area: f32,
    pub face_count: usize,
}

/// Compute the area of a single triangle.
#[allow(dead_code)]
pub fn triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt()
}

/// Compute total surface area of a mesh.
#[allow(dead_code)]
pub fn mesh_total_area(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    indices
        .iter()
        .map(|tri| {
            triangle_area(
                positions[tri[0] as usize],
                positions[tri[1] as usize],
                positions[tri[2] as usize],
            )
        })
        .sum()
}

/// Compute per-face areas.
#[allow(dead_code)]
pub fn face_areas(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> Vec<f32> {
    indices
        .iter()
        .map(|tri| {
            triangle_area(
                positions[tri[0] as usize],
                positions[tri[1] as usize],
                positions[tri[2] as usize],
            )
        })
        .collect()
}

/// Sum of area-weighted face normals.
#[allow(dead_code)]
pub fn area_weighted_normal_sum(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> [f32; 3] {
    let mut sum = [0.0f32; 3];
    for tri in indices {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        sum[0] += cross[0];
        sum[1] += cross[1];
        sum[2] += cross[2];
    }
    sum
}

/// Find the largest face area.
#[allow(dead_code)]
pub fn largest_face_area(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    face_areas(positions, indices)
        .into_iter()
        .fold(0.0f32, f32::max)
}

/// Find the smallest face area.
#[allow(dead_code)]
pub fn smallest_face_area(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    face_areas(positions, indices)
        .into_iter()
        .fold(f32::MAX, f32::min)
}

/// Compute the average face area.
#[allow(dead_code)]
pub fn average_face_area(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> f32 {
    if indices.is_empty() {
        return 0.0;
    }
    mesh_total_area(positions, indices) / indices.len() as f32
}

/// Produce a histogram of face areas into `bin_count` bins.
#[allow(dead_code)]
pub fn area_histogram(positions: &[[f32; 3]], indices: &[[u32; 3]], bin_count: usize) -> Vec<u32> {
    let areas = face_areas(positions, indices);
    let max_a = areas.iter().copied().fold(0.0f32, f32::max);
    if max_a < 1e-12 || bin_count == 0 {
        return vec![0; bin_count];
    }
    let mut bins = vec![0u32; bin_count];
    for &a in &areas {
        let idx = ((a / max_a) * (bin_count as f32 - 1.0)).round() as usize;
        let idx = idx.min(bin_count - 1);
        bins[idx] += 1;
    }
    bins
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_area_unit() {
        let area = triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((area - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_triangle_area_degenerate() {
        let area = triangle_area([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!(area.abs() < 1e-6);
    }

    #[test]
    fn test_mesh_total_area() {
        let p = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]];
        let i = vec![[0u32, 1, 2]];
        assert!((mesh_total_area(&p, &i) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_face_areas() {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let i = vec![[0u32, 1, 2]];
        let areas = face_areas(&p, &i);
        assert_eq!(areas.len(), 1);
        assert!((areas[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_area_weighted_normal_sum() {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let i = vec![[0u32, 1, 2]];
        let n = area_weighted_normal_sum(&p, &i);
        assert!(n[2] > 0.0);
    }

    #[test]
    fn test_largest_face_area() {
        let p = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0],
        ];
        let i = vec![[0u32, 1, 2], [3, 4, 5]];
        assert!((largest_face_area(&p, &i) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_smallest_face_area() {
        let p = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0],
        ];
        let i = vec![[0u32, 1, 2], [3, 4, 5]];
        assert!((smallest_face_area(&p, &i) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_average_face_area() {
        let p = vec![
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0],
        ];
        let i = vec![[0u32, 1, 2], [3, 4, 5]];
        assert!((average_face_area(&p, &i) - 1.25).abs() < 1e-6);
    }

    #[test]
    fn test_average_face_area_empty() {
        assert!((average_face_area(&[], &[])).abs() < 1e-6);
    }

    #[test]
    fn test_area_histogram() {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let i = vec![[0u32, 1, 2]];
        let h = area_histogram(&p, &i, 4);
        assert_eq!(h.len(), 4);
    }
}
