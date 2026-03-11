//! Lightmap data export.
#![allow(dead_code)]

/// Lightmap data store.
#[allow(dead_code)]
pub struct LightmapData2 {
    pub texels: Vec<[f32; 3]>,
    pub width: usize,
    pub height: usize,
}

/// Lightmap export configuration.
#[allow(dead_code)]
pub struct LightmapExport2 {
    pub data: LightmapData2,
}

/// Create a new lightmap.
#[allow(dead_code)]
pub fn new_lightmap2(width: usize, height: usize) -> LightmapData2 {
    LightmapData2 { texels: vec![[0.0; 3]; width * height], width, height }
}

/// Set a texel value.
#[allow(dead_code)]
pub fn set_texel2(lm: &mut LightmapData2, x: usize, y: usize, rgb: [f32; 3]) {
    let idx = y * lm.width + x;
    if idx < lm.texels.len() { lm.texels[idx] = rgb; }
}

/// Get a texel value.
#[allow(dead_code)]
pub fn get_texel2(lm: &LightmapData2, x: usize, y: usize) -> [f32; 3] {
    let idx = y * lm.width + x;
    lm.texels.get(idx).copied().unwrap_or([0.0; 3])
}

/// Export lightmap as raw bytes (RGB, f32 LE).
#[allow(dead_code)]
pub fn export_lightmap2_raw(lm: &LightmapData2) -> Vec<u8> {
    lm.texels.iter().flat_map(|t| {
        let mut b = Vec::new();
        for &v in t { b.extend_from_slice(&v.to_le_bytes()); }
        b
    }).collect()
}

/// Get lightmap width.
#[allow(dead_code)]
pub fn lightmap2_width(lm: &LightmapData2) -> usize { lm.width }

/// Get lightmap height.
#[allow(dead_code)]
pub fn lightmap2_height(lm: &LightmapData2) -> usize { lm.height }

/// Get texel count.
#[allow(dead_code)]
pub fn lightmap2_texel_count(lm: &LightmapData2) -> usize { lm.texels.len() }

/// Convert lightmap to byte vec (u8 RGB).
#[allow(dead_code)]
pub fn lightmap2_to_bytes(lm: &LightmapData2) -> Vec<u8> {
    lm.texels.iter().flat_map(|t| {
        [(t[0] * 255.0) as u8, (t[1] * 255.0) as u8, (t[2] * 255.0) as u8]
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lightmap_size() {
        let lm = new_lightmap2(4, 4);
        assert_eq!(lightmap2_texel_count(&lm), 16);
    }

    #[test]
    fn test_set_get_texel() {
        let mut lm = new_lightmap2(4, 4);
        set_texel2(&mut lm, 1, 2, [1.0, 0.5, 0.0]);
        let t = get_texel2(&lm, 1, 2);
        assert!((t[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_get_texel_oob() {
        let lm = new_lightmap2(4, 4);
        let t = get_texel2(&lm, 100, 100);
        assert!((t[0]).abs() < 1e-5);
    }

    #[test]
    fn test_export_raw_bytes() {
        let lm = new_lightmap2(2, 2);
        let b = export_lightmap2_raw(&lm);
        assert_eq!(b.len(), 4 * 3 * 4);
    }

    #[test]
    fn test_lightmap_width() {
        let lm = new_lightmap2(8, 4);
        assert_eq!(lightmap2_width(&lm), 8);
    }

    #[test]
    fn test_lightmap_height() {
        let lm = new_lightmap2(8, 4);
        assert_eq!(lightmap2_height(&lm), 4);
    }

    #[test]
    fn test_lightmap_to_bytes_count() {
        let lm = new_lightmap2(2, 2);
        let b = lightmap2_to_bytes(&lm);
        assert_eq!(b.len(), 4 * 3);
    }

    #[test]
    fn test_lightmap_export2_struct() {
        let data = new_lightmap2(2, 2);
        let le = LightmapExport2 { data };
        assert_eq!(le.data.width, 2);
    }

    #[test]
    fn test_texel_zero_default() {
        let lm = new_lightmap2(3, 3);
        for t in &lm.texels { assert!((t[0]).abs() < 1e-5); }
    }
}
