#![allow(dead_code)]

//! Face splitting operations for triangle meshes.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceSplit {
    pub new_positions: Vec<[f32; 3]>,
    pub new_indices: Vec<u32>,
    pub split_count: usize,
}

#[allow(dead_code)]
pub fn split_face_at_center(positions: &[[f32; 3]], face: [u32; 3]) -> FaceSplit {
    let a = positions[face[0] as usize];
    let b = positions[face[1] as usize];
    let c = positions[face[2] as usize];
    let center = [(a[0]+b[0]+c[0])/3.0, (a[1]+b[1]+c[1])/3.0, (a[2]+b[2]+c[2])/3.0];
    let new_idx = positions.len() as u32;
    let mut new_pos = positions.to_vec();
    new_pos.push(center);
    let new_indices = vec![
        face[0], face[1], new_idx,
        face[1], face[2], new_idx,
        face[2], face[0], new_idx,
    ];
    FaceSplit { new_positions: new_pos, new_indices, split_count: 3 }
}

#[allow(dead_code)]
pub fn split_face_at_edge(positions: &[[f32; 3]], face: [u32; 3], edge_idx: usize) -> FaceSplit {
    let e0 = face[edge_idx % 3] as usize;
    let e1 = face[(edge_idx + 1) % 3] as usize;
    let opp = face[(edge_idx + 2) % 3];
    let mid = [
        (positions[e0][0] + positions[e1][0]) * 0.5,
        (positions[e0][1] + positions[e1][1]) * 0.5,
        (positions[e0][2] + positions[e1][2]) * 0.5,
    ];
    let new_idx = positions.len() as u32;
    let mut new_pos = positions.to_vec();
    new_pos.push(mid);
    let new_indices = vec![
        face[edge_idx % 3], new_idx, opp,
        new_idx, face[(edge_idx + 1) % 3], opp,
    ];
    FaceSplit { new_positions: new_pos, new_indices, split_count: 2 }
}

#[allow(dead_code)]
pub fn split_quad_to_tris(quad: [u32; 4]) -> Vec<u32> {
    vec![quad[0], quad[1], quad[2], quad[0], quad[2], quad[3]]
}

#[allow(dead_code)]
pub fn split_ngon_to_tris(ngon: &[u32]) -> Vec<u32> {
    if ngon.len() < 3 { return Vec::new(); }
    let mut tris = Vec::new();
    for i in 1..ngon.len() - 1 {
        tris.push(ngon[0]);
        tris.push(ngon[i]);
        tris.push(ngon[i + 1]);
    }
    tris
}

#[allow(dead_code)]
pub fn split_count_fs(split: &FaceSplit) -> usize {
    split.split_count
}

#[allow(dead_code)]
pub fn split_creates_vertices(split: &FaceSplit, original_count: usize) -> usize {
    split.new_positions.len() - original_count
}

#[allow(dead_code)]
pub fn split_to_json(split: &FaceSplit) -> String {
    format!(
        "{{\"split_count\":{},\"vertex_count\":{},\"index_count\":{}}}",
        split.split_count, split.new_positions.len(), split.new_indices.len()
    )
}

#[allow(dead_code)]
pub fn split_validate(split: &FaceSplit) -> bool {
    split.new_indices.len().is_multiple_of(3) && !split.new_positions.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tri_pos() -> Vec<[f32; 3]> {
        vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]]
    }

    #[test]
    fn test_split_at_center() {
        let s = split_face_at_center(&tri_pos(), [0,1,2]);
        assert_eq!(s.split_count, 3);
        assert_eq!(s.new_indices.len(), 9);
    }

    #[test]
    fn test_split_at_edge() {
        let s = split_face_at_edge(&tri_pos(), [0,1,2], 0);
        assert_eq!(s.split_count, 2);
        assert_eq!(s.new_indices.len(), 6);
    }

    #[test]
    fn test_split_quad() {
        let t = split_quad_to_tris([0,1,2,3]);
        assert_eq!(t.len(), 6);
    }

    #[test]
    fn test_split_ngon() {
        let t = split_ngon_to_tris(&[0,1,2,3,4]);
        assert_eq!(t.len(), 9);
    }

    #[test]
    fn test_split_ngon_tri() {
        let t = split_ngon_to_tris(&[0,1,2]);
        assert_eq!(t.len(), 3);
    }

    #[test]
    fn test_split_ngon_empty() {
        assert!(split_ngon_to_tris(&[0,1]).is_empty());
    }

    #[test]
    fn test_split_count() {
        let s = split_face_at_center(&tri_pos(), [0,1,2]);
        assert_eq!(split_count_fs(&s), 3);
    }

    #[test]
    fn test_split_creates_vertices() {
        let s = split_face_at_center(&tri_pos(), [0,1,2]);
        assert_eq!(split_creates_vertices(&s, 3), 1);
    }

    #[test]
    fn test_split_to_json() {
        let s = split_face_at_center(&tri_pos(), [0,1,2]);
        assert!(split_to_json(&s).contains("\"split_count\":3"));
    }

    #[test]
    fn test_split_validate() {
        let s = split_face_at_center(&tri_pos(), [0,1,2]);
        assert!(split_validate(&s));
    }
}
