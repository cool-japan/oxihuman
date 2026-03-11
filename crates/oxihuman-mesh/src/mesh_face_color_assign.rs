#![allow(dead_code)]
//! Per-face color assignment.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceColorAssign { colors: Vec<[f32; 4]> }

#[allow(dead_code)]
pub fn assign_face_color(fca: &mut FaceColorAssign, face: usize, color: [f32; 4]) {
    if face < fca.colors.len() { fca.colors[face] = color; }
}

#[allow(dead_code)]
pub fn color_at_face(fca: &FaceColorAssign, face: usize) -> [f32; 4] {
    fca.colors.get(face).copied().unwrap_or([0.0; 4])
}

#[allow(dead_code)]
pub fn face_color_count(fca: &FaceColorAssign) -> usize { fca.colors.len() }

#[allow(dead_code)]
pub fn clear_face_colors(fca: &mut FaceColorAssign) { fca.colors.clear(); }

#[allow(dead_code)]
pub fn face_colors_to_bytes(fca: &FaceColorAssign) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(fca.colors.len() * 16);
    for c in &fca.colors { for &v in c { bytes.extend_from_slice(&v.to_le_bytes()); } }
    bytes
}

#[allow(dead_code)]
pub fn face_color_to_json(fca: &FaceColorAssign) -> String {
    let cs: Vec<String> = fca.colors.iter().map(|c| format!("[{:.4},{:.4},{:.4},{:.4}]", c[0],c[1],c[2],c[3])).collect();
    format!("{{\"face_colors\":[{}]}}", cs.join(","))
}

#[allow(dead_code)]
pub fn random_face_colors_deterministic(face_count: usize) -> FaceColorAssign {
    let mut colors = Vec::with_capacity(face_count);
    for i in 0..face_count {
        let h = ((i as f32 * 0.618034) % 1.0) * 6.0;
        let sector = h.floor() as u32 % 6;
        let f = h - h.floor();
        let (r, g, b) = match sector {
            0 => (1.0, f, 0.0), 1 => (1.0-f, 1.0, 0.0), 2 => (0.0, 1.0, f),
            3 => (0.0, 1.0-f, 1.0), 4 => (f, 0.0, 1.0), _ => (1.0, 0.0, 1.0-f),
        };
        colors.push([r, g, b, 1.0]);
    }
    FaceColorAssign { colors }
}

#[allow(dead_code)]
pub fn validate_face_colors(fca: &FaceColorAssign) -> bool {
    fca.colors.iter().all(|c| c.iter().all(|&v| (0.0..=1.0).contains(&v)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_random_det() { let fca = random_face_colors_deterministic(10); assert_eq!(face_color_count(&fca), 10); }
    #[test] fn test_assign() { let mut fca = random_face_colors_deterministic(3); assign_face_color(&mut fca, 0, [1.0,0.0,0.0,1.0]); assert!((color_at_face(&fca, 0)[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_color_at() { let fca = random_face_colors_deterministic(3); let c = color_at_face(&fca, 0); assert!(c[3] > 0.0); }
    #[test] fn test_count() { let fca = random_face_colors_deterministic(5); assert_eq!(face_color_count(&fca), 5); }
    #[test] fn test_clear() { let mut fca = random_face_colors_deterministic(3); clear_face_colors(&mut fca); assert_eq!(face_color_count(&fca), 0); }
    #[test] fn test_bytes() { let fca = random_face_colors_deterministic(2); assert_eq!(face_colors_to_bytes(&fca).len(), 32); }
    #[test] fn test_json() { let fca = random_face_colors_deterministic(2); assert!(face_color_to_json(&fca).contains("face_colors")); }
    #[test] fn test_validate() { let fca = random_face_colors_deterministic(5); assert!(validate_face_colors(&fca)); }
    #[test] fn test_oob() { let fca = random_face_colors_deterministic(1); assert!((color_at_face(&fca, 99)[0] - 0.0).abs() < 1e-9); }
    #[test] fn test_empty() { let fca = random_face_colors_deterministic(0); assert_eq!(face_color_count(&fca), 0); }
}
