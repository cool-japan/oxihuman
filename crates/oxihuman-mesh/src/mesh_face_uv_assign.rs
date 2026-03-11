#![allow(dead_code)]
//! Per-face UV assignment.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceUvAssign { uvs: Vec<[[f32;2];3]> }

#[allow(dead_code)]
pub fn assign_face_uv(fua: &mut FaceUvAssign, face: usize, uvs: [[f32;2];3]) {
    if face < fua.uvs.len() { fua.uvs[face] = uvs; }
}
#[allow(dead_code)]
pub fn uv_at_face_vertex(fua: &FaceUvAssign, face: usize, vert: usize) -> [f32;2] {
    fua.uvs.get(face).and_then(|f| f.get(vert)).copied().unwrap_or([0.0;2])
}
#[allow(dead_code)]
pub fn face_uv_count(fua: &FaceUvAssign) -> usize { fua.uvs.len() }
#[allow(dead_code)]
pub fn clear_face_uvs(fua: &mut FaceUvAssign) { fua.uvs.clear(); }
#[allow(dead_code)]
pub fn face_uvs_to_bytes(fua: &FaceUvAssign) -> Vec<u8> {
    let mut b = Vec::with_capacity(fua.uvs.len()*24);
    for f in &fua.uvs { for uv in f { for &v in uv { b.extend_from_slice(&v.to_le_bytes()); } } }
    b
}
#[allow(dead_code)]
pub fn face_uv_to_json(fua: &FaceUvAssign) -> String {
    let fs: Vec<String> = fua.uvs.iter().map(|f| {
        let us: Vec<String> = f.iter().map(|uv| format!("[{:.4},{:.4}]",uv[0],uv[1])).collect();
        format!("[{}]", us.join(","))
    }).collect();
    format!("{{\"face_uvs\":[{}]}}", fs.join(","))
}
#[allow(dead_code)]
pub fn validate_face_uvs(fua: &FaceUvAssign) -> bool {
    fua.uvs.iter().all(|f| f.iter().all(|uv| uv.iter().all(|&v| v.is_finite())))
}
#[allow(dead_code)]
pub fn face_uv_bounds(fua: &FaceUvAssign) -> ([f32;2], [f32;2]) {
    let mut lo = [f32::INFINITY; 2];
    let mut hi = [f32::NEG_INFINITY; 2];
    for f in &fua.uvs { for uv in f { for i in 0..2 { lo[i] = lo[i].min(uv[i]); hi[i] = hi[i].max(uv[i]); } } }
    (lo, hi)
}

impl FaceUvAssign {
    #[allow(dead_code)]
    pub fn new(face_count: usize) -> Self {
        FaceUvAssign { uvs: vec![[[0.0;2];3]; face_count] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let fua = FaceUvAssign::new(3); assert_eq!(face_uv_count(&fua), 3); }
    #[test] fn test_assign() { let mut fua = FaceUvAssign::new(2); assign_face_uv(&mut fua, 0, [[0.0,0.0],[1.0,0.0],[0.5,1.0]]); assert!((uv_at_face_vertex(&fua,0,1)[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_at() { let fua = FaceUvAssign::new(1); assert!((uv_at_face_vertex(&fua,0,0)[0]).abs() < 1e-6); }
    #[test] fn test_count() { let fua = FaceUvAssign::new(5); assert_eq!(face_uv_count(&fua), 5); }
    #[test] fn test_clear() { let mut fua = FaceUvAssign::new(3); clear_face_uvs(&mut fua); assert_eq!(face_uv_count(&fua), 0); }
    #[test] fn test_bytes() { let fua = FaceUvAssign::new(1); assert_eq!(face_uvs_to_bytes(&fua).len(), 24); }
    #[test] fn test_json() { let fua = FaceUvAssign::new(1); assert!(face_uv_to_json(&fua).contains("face_uvs")); }
    #[test] fn test_validate() { let fua = FaceUvAssign::new(2); assert!(validate_face_uvs(&fua)); }
    #[test] fn test_bounds() { let mut fua = FaceUvAssign::new(1); assign_face_uv(&mut fua, 0, [[0.0,0.0],[1.0,0.0],[0.5,1.0]]); let (lo,hi) = face_uv_bounds(&fua); assert!((hi[0] - 1.0).abs() < 1e-6); assert!((lo[0]).abs() < 1e-6); }
    #[test] fn test_oob() { let fua = FaceUvAssign::new(1); assert!((uv_at_face_vertex(&fua,5,0)[0]).abs() < 1e-9); }
}
