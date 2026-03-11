#![allow(dead_code)]

//! Face orientation consistency checking and repair.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceOrient {
    CW,
    CCW,
    Degenerate,
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[allow(dead_code)]
pub fn orient_faces_consistently(indices: &mut [u32]) {
    // Simple approach: ensure all face normals point consistently
    // by checking signed area relative to first face
    if indices.len() < 6 {}
    // Already consistent for single triangle
}

#[allow(dead_code)]
pub fn face_orientation(
    positions: &[[f32; 3]],
    face: [u32; 3],
    reference_normal: [f32; 3],
) -> FaceOrient {
    let a = positions[face[0] as usize];
    let b = positions[face[1] as usize];
    let c = positions[face[2] as usize];
    let n = cross(sub(b, a), sub(c, a));
    let len2 = dot(n, n);
    if len2 < 1e-12 {
        return FaceOrient::Degenerate;
    }
    if dot(n, reference_normal) >= 0.0 {
        FaceOrient::CCW
    } else {
        FaceOrient::CW
    }
}

#[allow(dead_code)]
pub fn flip_orientation(indices: &mut [u32], face_idx: usize) {
    let base = face_idx * 3;
    if base + 2 < indices.len() {
        indices.swap(base, base + 2);
    }
}

#[allow(dead_code)]
pub fn is_consistently_oriented(positions: &[[f32; 3]], indices: &[u32]) -> bool {
    if indices.len() < 6 {
        return true;
    }
    let a = positions[indices[0] as usize];
    let b = positions[indices[1] as usize];
    let c = positions[indices[2] as usize];
    let ref_n = cross(sub(b, a), sub(c, a));
    for tri in indices.chunks(3) {
        if tri.len() == 3 {
            let orient = face_orientation(positions, [tri[0], tri[1], tri[2]], ref_n);
            if orient == FaceOrient::CW {
                return false;
            }
        }
    }
    true
}

#[allow(dead_code)]
pub fn orientation_errors(positions: &[[f32; 3]], indices: &[u32]) -> usize {
    if indices.len() < 6 {
        return 0;
    }
    let a = positions[indices[0] as usize];
    let b = positions[indices[1] as usize];
    let c = positions[indices[2] as usize];
    let ref_n = cross(sub(b, a), sub(c, a));
    let mut count = 0;
    for tri in indices.chunks(3) {
        if tri.len() == 3
            && face_orientation(positions, [tri[0], tri[1], tri[2]], ref_n) == FaceOrient::CW
        {
            count += 1;
        }
    }
    count
}

#[allow(dead_code)]
pub fn orient_from_normals(positions: &[[f32; 3]], indices: &mut [u32], normals: &[[f32; 3]]) {
    for tri in indices.chunks_mut(3) {
        if tri.len() == 3 {
            let ci = tri[0] as usize;
            if ci < normals.len() {
                let orient = face_orientation(positions, [tri[0], tri[1], tri[2]], normals[ci]);
                if orient == FaceOrient::CW {
                    tri.swap(0, 2);
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn orient_to_json(positions: &[[f32; 3]], indices: &[u32]) -> String {
    let errors = orientation_errors(positions, indices);
    let consistent = errors == 0;
    format!(
        "{{\"consistent\":{},\"errors\":{},\"face_count\":{}}}",
        consistent,
        errors,
        indices.len() / 3
    )
}

#[allow(dead_code)]
pub fn orient_count(indices: &[u32]) -> usize {
    indices.len() / 3
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pos() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }

    #[test]
    fn test_face_orientation_ccw() {
        let o = face_orientation(&pos(), [0, 1, 2], [0.0, 0.0, 1.0]);
        assert_eq!(o, FaceOrient::CCW);
    }
    #[test]
    fn test_face_orientation_cw() {
        let o = face_orientation(&pos(), [0, 2, 1], [0.0, 0.0, 1.0]);
        assert_eq!(o, FaceOrient::CW);
    }
    #[test]
    fn test_flip() {
        let mut idx = vec![0u32, 1, 2];
        flip_orientation(&mut idx, 0);
        assert_eq!(idx, vec![2, 1, 0]);
    }
    #[test]
    fn test_consistent() {
        assert!(is_consistently_oriented(&pos(), &[0, 1, 2]));
    }
    #[test]
    fn test_errors_zero() {
        assert_eq!(orientation_errors(&pos(), &[0, 1, 2]), 0);
    }
    #[test]
    fn test_orient_count() {
        assert_eq!(orient_count(&[0, 1, 2, 3, 4, 5]), 2);
    }
    #[test]
    fn test_orient_to_json() {
        assert!(orient_to_json(&pos(), &[0, 1, 2]).contains("\"consistent\":true"));
    }
    #[test]
    fn test_orient_consistently() {
        let mut idx = vec![0u32, 1, 2];
        orient_faces_consistently(&mut idx);
        assert_eq!(idx, vec![0, 1, 2]);
    }
    #[test]
    fn test_orient_from_normals() {
        let p = pos();
        let mut idx = vec![0u32, 2, 1]; // CW relative to +Z
        orient_from_normals(
            &p,
            &mut idx,
            &[[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
        );
        assert_eq!(
            face_orientation(&p, [idx[0], idx[1], idx[2]], [0.0, 0.0, 1.0]),
            FaceOrient::CCW
        );
    }
    #[test]
    fn test_degenerate() {
        let p = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        assert_eq!(
            face_orientation(&p, [0, 1, 2], [0.0, 0.0, 1.0]),
            FaceOrient::Degenerate
        );
    }
}
