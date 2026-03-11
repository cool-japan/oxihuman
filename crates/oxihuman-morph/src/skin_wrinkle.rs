#![allow(dead_code)]
//! Skin wrinkle system for morph-based wrinkle simulation.

/// A single wrinkle entry with position, depth, and activation state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinWrinkle {
    /// Human-readable name.
    pub name: String,
    /// Wrinkle depth factor in [0, 1].
    pub depth: f32,
    /// Whether the wrinkle is active.
    pub active: bool,
    /// Region index this wrinkle belongs to.
    pub region: usize,
}

/// A map of wrinkles across the skin surface.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WrinkleMap {
    /// Resolution width.
    pub width: usize,
    /// Resolution height.
    pub height: usize,
    /// Per-texel intensity values.
    pub data: Vec<f32>,
    /// Collection of wrinkle entries.
    pub wrinkles: Vec<SkinWrinkle>,
}

/// Create a new [`WrinkleMap`] with the given resolution.
#[allow(dead_code)]
pub fn new_wrinkle_map(width: usize, height: usize) -> WrinkleMap {
    WrinkleMap {
        width,
        height,
        data: vec![0.0; width * height],
        wrinkles: Vec::new(),
    }
}

/// Add a wrinkle entry to the map.
#[allow(dead_code)]
pub fn add_wrinkle(map: &mut WrinkleMap, name: &str, depth: f32, region: usize) {
    map.wrinkles.push(SkinWrinkle {
        name: name.to_string(),
        depth: depth.clamp(0.0, 1.0),
        active: true,
        region,
    });
}

/// Query wrinkle intensity at a texel coordinate.
#[allow(dead_code)]
pub fn wrinkle_intensity_at(map: &WrinkleMap, x: usize, y: usize) -> f32 {
    if x >= map.width || y >= map.height {
        return 0.0;
    }
    map.data[y * map.width + x]
}

/// Return the number of wrinkle entries.
#[allow(dead_code)]
pub fn wrinkle_count(map: &WrinkleMap) -> usize {
    map.wrinkles.len()
}

/// Activate a wrinkle by index.
#[allow(dead_code)]
pub fn activate_wrinkle(map: &mut WrinkleMap, index: usize) {
    if let Some(w) = map.wrinkles.get_mut(index) {
        w.active = true;
    }
}

/// Deactivate a wrinkle by index.
#[allow(dead_code)]
pub fn deactivate_wrinkle(map: &mut WrinkleMap, index: usize) {
    if let Some(w) = map.wrinkles.get_mut(index) {
        w.active = false;
    }
}

/// Serialize the wrinkle map data to a byte vector (little-endian f32).
#[allow(dead_code)]
pub fn wrinkle_map_to_bytes(map: &WrinkleMap) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(map.data.len() * 4);
    for &v in &map.data {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes
}

/// Smooth the wrinkle map using a simple box blur (single pass).
#[allow(dead_code)]
pub fn smooth_wrinkle_map(map: &mut WrinkleMap) {
    if map.width < 3 || map.height < 3 {
        return;
    }
    let mut smoothed = vec![0.0_f32; map.data.len()];
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let mut sum = 0.0_f32;
            for dy in 0..3_usize {
                for dx in 0..3_usize {
                    let ny = y + dy - 1;
                    let nx = x + dx - 1;
                    sum += map.data[ny * map.width + nx];
                }
            }
            smoothed[y * map.width + x] = sum / 9.0;
        }
    }
    map.data = smoothed;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wrinkle_map() {
        let m = new_wrinkle_map(4, 4);
        assert_eq!(m.width, 4);
        assert_eq!(m.height, 4);
        assert_eq!(m.data.len(), 16);
    }

    #[test]
    fn test_add_wrinkle() {
        let mut m = new_wrinkle_map(2, 2);
        add_wrinkle(&mut m, "forehead", 0.5, 0);
        assert_eq!(wrinkle_count(&m), 1);
        assert_eq!(m.wrinkles[0].name, "forehead");
    }

    #[test]
    fn test_wrinkle_intensity_at_zero() {
        let m = new_wrinkle_map(4, 4);
        assert!((wrinkle_intensity_at(&m, 1, 1) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_wrinkle_intensity_at_out_of_bounds() {
        let m = new_wrinkle_map(2, 2);
        assert!((wrinkle_intensity_at(&m, 5, 5) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_wrinkle_count_empty() {
        let m = new_wrinkle_map(2, 2);
        assert_eq!(wrinkle_count(&m), 0);
    }

    #[test]
    fn test_activate_deactivate() {
        let mut m = new_wrinkle_map(2, 2);
        add_wrinkle(&mut m, "crow", 0.3, 1);
        deactivate_wrinkle(&mut m, 0);
        assert!(!m.wrinkles[0].active);
        activate_wrinkle(&mut m, 0);
        assert!(m.wrinkles[0].active);
    }

    #[test]
    fn test_wrinkle_map_to_bytes() {
        let m = new_wrinkle_map(2, 2);
        let bytes = wrinkle_map_to_bytes(&m);
        assert_eq!(bytes.len(), 16); // 4 floats * 4 bytes
    }

    #[test]
    fn test_smooth_wrinkle_map() {
        let mut m = new_wrinkle_map(4, 4);
        m.data[5] = 9.0; // center-ish
        smooth_wrinkle_map(&mut m);
        // After smoothing the value should have spread
        assert!(m.data[5] < 9.0);
    }

    #[test]
    fn test_depth_clamped() {
        let mut m = new_wrinkle_map(2, 2);
        add_wrinkle(&mut m, "deep", 1.5, 0);
        assert!((m.wrinkles[0].depth - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_deactivate_out_of_bounds() {
        let mut m = new_wrinkle_map(2, 2);
        deactivate_wrinkle(&mut m, 99); // should not panic
        assert_eq!(wrinkle_count(&m), 0);
    }
}
