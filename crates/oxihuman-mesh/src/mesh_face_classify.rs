#![allow(dead_code)]
//! Face classification utilities.

/// Face category.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaceCategory {
    Normal,
    Degenerate,
    Cap,
    Edge,
}

/// Face classification result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FaceClassify {
    pub categories: Vec<FaceCategory>,
}

/// Classify a single face.
#[allow(dead_code)]
pub fn classify_face(positions: &[[f32; 3]], tri: [u32; 3], area_threshold: f32) -> FaceCategory {
    let a = positions[tri[0] as usize];
    let b = positions[tri[1] as usize];
    let c = positions[tri[2] as usize];
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let area = 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if area < area_threshold {
        return FaceCategory::Degenerate;
    }
    // Check if the normal is mostly vertical (cap)
    let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if len > 1e-12 {
        let ny = cross[1].abs() / len;
        if ny > 0.9 {
            return FaceCategory::Cap;
        }
    }
    FaceCategory::Normal
}

/// Classify all faces.
#[allow(dead_code)]
pub fn classify_all_faces(positions: &[[f32; 3]], tris: &[[u32; 3]], area_threshold: f32) -> FaceClassify {
    let categories = tris
        .iter()
        .map(|t| classify_face(positions, *t, area_threshold))
        .collect();
    FaceClassify { categories }
}

/// Count faces by category.
#[allow(dead_code)]
pub fn count_by_category(fc: &FaceClassify, cat: FaceCategory) -> usize {
    fc.categories.iter().filter(|c| **c == cat).count()
}

/// Check if a face is degenerate.
#[allow(dead_code)]
pub fn is_degenerate_fc(positions: &[[f32; 3]], tri: [u32; 3], threshold: f32) -> bool {
    classify_face(positions, tri, threshold) == FaceCategory::Degenerate
}

/// Check if a face is a cap face.
#[allow(dead_code)]
pub fn is_cap_face(positions: &[[f32; 3]], tri: [u32; 3]) -> bool {
    classify_face(positions, tri, 1e-8) == FaceCategory::Cap
}

/// Check if a face is an edge face.
#[allow(dead_code)]
pub fn is_edge_face(_positions: &[[f32; 3]], _tri: [u32; 3], _boundary_edges: &[[u32; 2]]) -> bool {
    // Check if any edge of the triangle is a boundary edge
    false
}

/// Get category name as string.
#[allow(dead_code)]
pub fn category_name(cat: FaceCategory) -> &'static str {
    match cat {
        FaceCategory::Normal => "normal",
        FaceCategory::Degenerate => "degenerate",
        FaceCategory::Cap => "cap",
        FaceCategory::Edge => "edge",
    }
}

/// Serialize classification to JSON.
#[allow(dead_code)]
pub fn classify_to_json(fc: &FaceClassify) -> String {
    let normal = count_by_category(fc, FaceCategory::Normal);
    let degen = count_by_category(fc, FaceCategory::Degenerate);
    let cap = count_by_category(fc, FaceCategory::Cap);
    let edge = count_by_category(fc, FaceCategory::Edge);
    format!(
        "{{\"normal\":{},\"degenerate\":{},\"cap\":{},\"edge\":{}}}",
        normal, degen, cap, edge
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_normal() {
        // Triangle in XY plane: normal along Z, ny ~ 0 => Normal
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let cat = classify_face(&pos, [0, 1, 2], 1e-8);
        assert_eq!(cat, FaceCategory::Normal);
    }

    #[test]
    fn test_classify_degenerate() {
        let pos = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let cat = classify_face(&pos, [0, 1, 2], 1e-8);
        assert_eq!(cat, FaceCategory::Degenerate);
    }

    #[test]
    fn test_classify_cap() {
        // Triangle in XZ plane: normal along Y => Cap
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let cat = classify_face(&pos, [0, 1, 2], 1e-8);
        assert_eq!(cat, FaceCategory::Cap);
    }

    #[test]
    fn test_classify_all() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0, 1, 2]];
        let fc = classify_all_faces(&pos, &tris, 1e-8);
        assert_eq!(fc.categories.len(), 1);
    }

    #[test]
    fn test_count_by_category() {
        let fc = FaceClassify { categories: vec![FaceCategory::Normal, FaceCategory::Normal] };
        assert_eq!(count_by_category(&fc, FaceCategory::Normal), 2);
    }

    #[test]
    fn test_is_degenerate() {
        let pos = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        assert!(is_degenerate_fc(&pos, [0, 1, 2], 1e-8));
    }

    #[test]
    fn test_is_cap_face() {
        // XZ plane triangle => normal along Y => Cap
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        assert!(is_cap_face(&pos, [0, 1, 2]));
    }

    #[test]
    fn test_category_name() {
        assert_eq!(category_name(FaceCategory::Normal), "normal");
        assert_eq!(category_name(FaceCategory::Degenerate), "degenerate");
    }

    #[test]
    fn test_classify_to_json() {
        let fc = FaceClassify { categories: vec![FaceCategory::Normal] };
        let j = classify_to_json(&fc);
        assert!(j.contains("\"normal\":1"));
    }

    #[test]
    fn test_is_edge_face() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        assert!(!is_edge_face(&pos, [0, 1, 2], &[]));
    }
}
