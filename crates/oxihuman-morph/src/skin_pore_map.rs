#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinPoreMap {
    intensities: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_skin_pore_map(vertex_count: usize) -> SkinPoreMap {
    SkinPoreMap { intensities: vec![0.0; vertex_count] }
}

#[allow(dead_code)]
pub fn set_pore_intensity_spm(map: &mut SkinPoreMap, idx: usize, val: f32) {
    if idx < map.intensities.len() { map.intensities[idx] = val.clamp(0.0, 1.0); }
}

#[allow(dead_code)]
pub fn pore_intensity_at(map: &SkinPoreMap, idx: usize) -> f32 {
    if idx < map.intensities.len() { map.intensities[idx] } else { 0.0 }
}

#[allow(dead_code)]
pub fn pore_vertex_count(map: &SkinPoreMap) -> usize { map.intensities.len() }

#[allow(dead_code)]
pub fn pore_to_params(map: &SkinPoreMap) -> Vec<f32> { map.intensities.clone() }

#[allow(dead_code)]
pub fn pore_map_to_json(map: &SkinPoreMap) -> String {
    let avg = if map.intensities.is_empty() { 0.0 } else { map.intensities.iter().sum::<f32>() / map.intensities.len() as f32 };
    format!("{{\"vertex_count\":{},\"avg_intensity\":{:.4}}}", map.intensities.len(), avg)
}

#[allow(dead_code)]
pub fn smooth_pore_map(map: &mut SkinPoreMap, iterations: usize) {
    for _ in 0..iterations {
        let prev = map.intensities.clone();
        for i in 0..map.intensities.len() {
            let mut sum = prev[i];
            let mut count = 1.0_f32;
            if i > 0 { sum += prev[i - 1]; count += 1.0; }
            if i + 1 < prev.len() { sum += prev[i + 1]; count += 1.0; }
            map.intensities[i] = sum / count;
        }
    }
}

#[allow(dead_code)]
pub fn clear_pore_map(map: &mut SkinPoreMap) {
    for v in map.intensities.iter_mut() { *v = 0.0; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let m = new_skin_pore_map(10); assert_eq!(pore_vertex_count(&m), 10); }
    #[test] fn test_set_get() { let mut m = new_skin_pore_map(5); set_pore_intensity_spm(&mut m, 2, 0.8); assert!((pore_intensity_at(&m, 2) - 0.8).abs() < 1e-6); }
    #[test] fn test_clamp() { let mut m = new_skin_pore_map(1); set_pore_intensity_spm(&mut m, 0, 2.0); assert!((pore_intensity_at(&m, 0) - 1.0).abs() < 1e-6); }
    #[test] fn test_oob() { let m = new_skin_pore_map(1); assert!((pore_intensity_at(&m, 5)).abs() < 1e-6); }
    #[test] fn test_to_params() { let m = new_skin_pore_map(3); assert_eq!(pore_to_params(&m).len(), 3); }
    #[test] fn test_json() { let m = new_skin_pore_map(2); assert!(pore_map_to_json(&m).contains("vertex_count")); }
    #[test] fn test_smooth() { let mut m = new_skin_pore_map(5); set_pore_intensity_spm(&mut m, 2, 1.0); smooth_pore_map(&mut m, 1); assert!(pore_intensity_at(&m, 1) > 0.0); }
    #[test] fn test_clear() { let mut m = new_skin_pore_map(3); set_pore_intensity_spm(&mut m, 0, 0.5); clear_pore_map(&mut m); assert!((pore_intensity_at(&m, 0)).abs() < 1e-6); }
    #[test] fn test_empty() { let m = new_skin_pore_map(0); assert_eq!(pore_vertex_count(&m), 0); }
    #[test] fn test_json_empty() { let m = new_skin_pore_map(0); assert!(pore_map_to_json(&m).contains("0")); }
}
