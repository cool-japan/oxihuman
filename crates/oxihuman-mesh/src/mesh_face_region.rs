#![allow(dead_code)]

//! Face region management for mesh segmentation.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshFaceRegion {
    pub name: String,
    pub faces: Vec<usize>,
}

#[allow(dead_code)]
pub fn new_face_region_mfr(name: &str) -> MeshFaceRegion {
    MeshFaceRegion { name: name.to_string(), faces: Vec::new() }
}

#[allow(dead_code)]
pub fn add_face_to_region(region: &mut MeshFaceRegion, face_idx: usize) {
    if !region.faces.contains(&face_idx) {
        region.faces.push(face_idx);
    }
}

#[allow(dead_code)]
pub fn region_face_count_mfr(region: &MeshFaceRegion) -> usize {
    region.faces.len()
}

#[allow(dead_code)]
pub fn region_bounds_mfr(region: &MeshFaceRegion, positions: &[[f32; 3]], indices: &[u32]) -> ([f32; 3], [f32; 3]) {
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    let mut found = false;
    for &fi in &region.faces {
        let base = fi * 3;
        if base + 2 < indices.len() {
            for &vi in &indices[base..base + 3] {
                let p = positions[vi as usize];
                for i in 0..3 {
                    if p[i] < lo[i] { lo[i] = p[i]; }
                    if p[i] > hi[i] { hi[i] = p[i]; }
                }
                found = true;
            }
        }
    }
    if !found { return ([0.0; 3], [0.0; 3]); }
    (lo, hi)
}

#[allow(dead_code)]
pub fn region_name_mfr(region: &MeshFaceRegion) -> &str {
    &region.name
}

#[allow(dead_code)]
pub fn merge_regions_mfr(a: &MeshFaceRegion, b: &MeshFaceRegion) -> MeshFaceRegion {
    let mut merged = a.clone();
    for &f in &b.faces {
        if !merged.faces.contains(&f) {
            merged.faces.push(f);
        }
    }
    merged.name = format!("{}+{}", a.name, b.name);
    merged
}

#[allow(dead_code)]
pub fn region_to_json_mfr(region: &MeshFaceRegion) -> String {
    let faces_str: Vec<String> = region.faces.iter().map(|f| f.to_string()).collect();
    format!("{{\"name\":\"{}\",\"face_count\":{},\"faces\":[{}]}}", region.name, region.faces.len(), faces_str.join(","))
}

#[allow(dead_code)]
pub fn region_clear_mfr(region: &mut MeshFaceRegion) {
    region.faces.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() { let r = new_face_region_mfr("head"); assert_eq!(r.name, "head"); }
    #[test]
    fn test_add_face() { let mut r = new_face_region_mfr("a"); add_face_to_region(&mut r, 0); assert_eq!(region_face_count_mfr(&r), 1); }
    #[test]
    fn test_no_duplicate() { let mut r = new_face_region_mfr("a"); add_face_to_region(&mut r, 0); add_face_to_region(&mut r, 0); assert_eq!(region_face_count_mfr(&r), 1); }
    #[test]
    fn test_bounds() {
        let mut r = new_face_region_mfr("a"); add_face_to_region(&mut r, 0);
        let pos = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]]; let idx = vec![0,1,2];
        let (lo,hi) = region_bounds_mfr(&r, &pos, &idx);
        assert!((lo[0]).abs() < 1e-6);
        assert!((hi[0] - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_name() { let r = new_face_region_mfr("torso"); assert_eq!(region_name_mfr(&r), "torso"); }
    #[test]
    fn test_merge() {
        let mut a = new_face_region_mfr("a"); add_face_to_region(&mut a, 0);
        let mut b = new_face_region_mfr("b"); add_face_to_region(&mut b, 1);
        let m = merge_regions_mfr(&a, &b);
        assert_eq!(region_face_count_mfr(&m), 2);
    }
    #[test]
    fn test_to_json() { let r = new_face_region_mfr("x"); assert!(region_to_json_mfr(&r).contains("\"name\":\"x\"")); }
    #[test]
    fn test_clear() { let mut r = new_face_region_mfr("a"); add_face_to_region(&mut r, 0); region_clear_mfr(&mut r); assert_eq!(region_face_count_mfr(&r), 0); }
    #[test]
    fn test_empty_bounds() { let r = new_face_region_mfr("a"); let (lo,_) = region_bounds_mfr(&r, &[], &[]); assert!((lo[0]).abs() < 1e-6); }
    #[test]
    fn test_merge_overlap() {
        let mut a = new_face_region_mfr("a"); add_face_to_region(&mut a, 0);
        let mut b = new_face_region_mfr("b"); add_face_to_region(&mut b, 0);
        let m = merge_regions_mfr(&a, &b);
        assert_eq!(region_face_count_mfr(&m), 1);
    }
}
