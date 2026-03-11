#![allow(dead_code)]
//! Sort faces by area.

/// Sort result with face indices and areas.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FaceAreaSort {
    pub sorted_indices: Vec<usize>,
    pub areas: Vec<f32>,
}

fn triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt()
}

/// Sort faces by area descending.
#[allow(dead_code)]
pub fn sort_faces_by_area(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> FaceAreaSort {
    let mut face_areas: Vec<(usize, f32)> = indices
        .iter()
        .enumerate()
        .map(|(i, tri)| {
            let a = positions[tri[0] as usize];
            let b = positions[tri[1] as usize];
            let c = positions[tri[2] as usize];
            (i, triangle_area(a, b, c))
        })
        .collect();
    face_areas.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    FaceAreaSort {
        sorted_indices: face_areas.iter().map(|&(i, _)| i).collect(),
        areas: face_areas.iter().map(|&(_, a)| a).collect(),
    }
}

/// Get the largest N faces.
#[allow(dead_code)]
pub fn largest_faces(sort: &FaceAreaSort, n: usize) -> &[usize] {
    let end = n.min(sort.sorted_indices.len());
    &sort.sorted_indices[..end]
}

/// Get the smallest N faces.
#[allow(dead_code)]
pub fn smallest_faces(sort: &FaceAreaSort, n: usize) -> Vec<usize> {
    let len = sort.sorted_indices.len();
    let start = len.saturating_sub(n);
    sort.sorted_indices[start..].to_vec()
}

/// Compute median face area.
#[allow(dead_code)]
pub fn median_face_area(sort: &FaceAreaSort) -> f32 {
    if sort.areas.is_empty() {
        return 0.0;
    }
    let mut sorted = sort.areas.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    sorted[mid]
}

/// Get face area at a given percentile (0.0..1.0).
#[allow(dead_code)]
pub fn face_area_percentile(sort: &FaceAreaSort, percentile: f32) -> f32 {
    if sort.areas.is_empty() {
        return 0.0;
    }
    let mut sorted = sort.areas.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((percentile * (sorted.len() - 1) as f32) as usize).min(sorted.len() - 1);
    sorted[idx]
}

/// Get the range (min, max) of face areas.
#[allow(dead_code)]
pub fn face_area_range(sort: &FaceAreaSort) -> (f32, f32) {
    if sort.areas.is_empty() {
        return (0.0, 0.0);
    }
    let mn = sort.areas.iter().copied().fold(f32::MAX, f32::min);
    let mx = sort.areas.iter().copied().fold(f32::MIN, f32::max);
    (mn, mx)
}

/// Return sorted face indices.
#[allow(dead_code)]
pub fn sorted_face_indices(sort: &FaceAreaSort) -> &[usize] {
    &sort.sorted_indices
}

/// Sort ascending instead of descending.
#[allow(dead_code)]
pub fn sort_ascending(sort: &mut FaceAreaSort) {
    sort.sorted_indices.reverse();
    sort.areas.reverse();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let p = vec![
            [0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
        ];
        let i = vec![[0, 1, 2], [3, 4, 5]];
        (p, i)
    }

    #[test]
    fn test_sort_faces_by_area() {
        let (p, i) = sample();
        let s = sort_faces_by_area(&p, &i);
        assert_eq!(s.sorted_indices.len(), 2);
        // First should be the larger face
        assert_eq!(s.sorted_indices[0], 0);
    }

    #[test]
    fn test_largest_faces() {
        let (p, i) = sample();
        let s = sort_faces_by_area(&p, &i);
        let l = largest_faces(&s, 1);
        assert_eq!(l.len(), 1);
    }

    #[test]
    fn test_smallest_faces() {
        let (p, i) = sample();
        let s = sort_faces_by_area(&p, &i);
        let sm = smallest_faces(&s, 1);
        assert_eq!(sm.len(), 1);
    }

    #[test]
    fn test_median_face_area() {
        let (p, i) = sample();
        let s = sort_faces_by_area(&p, &i);
        let m = median_face_area(&s);
        assert!(m > 0.0);
    }

    #[test]
    fn test_face_area_percentile() {
        let (p, i) = sample();
        let s = sort_faces_by_area(&p, &i);
        let low = face_area_percentile(&s, 0.0);
        let high = face_area_percentile(&s, 1.0);
        assert!(low <= high);
    }

    #[test]
    fn test_face_area_range() {
        let (p, i) = sample();
        let s = sort_faces_by_area(&p, &i);
        let (mn, mx) = face_area_range(&s);
        assert!(mn <= mx);
    }

    #[test]
    fn test_sorted_face_indices() {
        let (p, i) = sample();
        let s = sort_faces_by_area(&p, &i);
        assert_eq!(sorted_face_indices(&s).len(), 2);
    }

    #[test]
    fn test_sort_ascending() {
        let (p, i) = sample();
        let mut s = sort_faces_by_area(&p, &i);
        sort_ascending(&mut s);
        assert_eq!(s.sorted_indices[0], 1);
    }

    #[test]
    fn test_median_empty() {
        let s = FaceAreaSort { sorted_indices: vec![], areas: vec![] };
        assert_eq!(median_face_area(&s), 0.0);
    }

    #[test]
    fn test_range_empty() {
        let s = FaceAreaSort { sorted_indices: vec![], areas: vec![] };
        assert_eq!(face_area_range(&s), (0.0, 0.0));
    }
}
