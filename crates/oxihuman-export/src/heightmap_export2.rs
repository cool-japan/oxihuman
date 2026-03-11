//! Heightmap export utilities.
#![allow(dead_code)]

/// Heightmap export data.
#[allow(dead_code)]
pub struct HeightmapExport2 {
    pub data: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

/// Export heightmap as raw f32 bytes.
#[allow(dead_code)]
pub fn export_heightmap2_raw(hm: &HeightmapExport2) -> Vec<u8> {
    hm.data.iter().flat_map(|f| f.to_le_bytes()).collect()
}

/// Convert heightmap to R16 format (0..=65535).
#[allow(dead_code)]
pub fn heightmap2_to_r16(hm: &HeightmapExport2) -> Vec<u16> {
    let (mn, mx) = (heightmap2_min(hm), heightmap2_max(hm));
    let range = (mx - mn).max(1e-8);
    hm.data.iter().map(|&v| ((v - mn) / range * 65535.0) as u16).collect()
}

/// Normalize heightmap values to [0, 1].
#[allow(dead_code)]
pub fn heightmap2_normalize(hm: &mut HeightmapExport2) {
    let mn = heightmap2_min(hm); let mx = heightmap2_max(hm);
    let range = (mx - mn).max(1e-8);
    for v in hm.data.iter_mut() { *v = (*v - mn) / range; }
}

/// Minimum value in heightmap.
#[allow(dead_code)]
pub fn heightmap2_min(hm: &HeightmapExport2) -> f32 {
    hm.data.iter().copied().fold(f32::MAX, f32::min)
}

/// Maximum value in heightmap.
#[allow(dead_code)]
pub fn heightmap2_max(hm: &HeightmapExport2) -> f32 {
    hm.data.iter().copied().fold(f32::MIN, f32::max)
}

/// Convert heightmap to string (CSV-like).
#[allow(dead_code)]
pub fn heightmap2_to_string(hm: &HeightmapExport2) -> String {
    hm.data.iter().map(|v| format!("{:.4}", v)).collect::<Vec<_>>().join(",")
}

/// Get vertex count (width * height).
#[allow(dead_code)]
pub fn heightmap2_vertex_count(hm: &HeightmapExport2) -> usize {
    hm.width * hm.height
}

/// Get cell count ((width-1)*(height-1)).
#[allow(dead_code)]
pub fn heightmap2_cell_count(hm: &HeightmapExport2) -> usize {
    (hm.width.saturating_sub(1)) * (hm.height.saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_hm() -> HeightmapExport2 {
        HeightmapExport2 { data: vec![0.0, 0.5, 1.0, 0.25], width: 2, height: 2 }
    }

    #[test]
    fn test_export_raw_bytes() {
        let hm = sample_hm();
        let b = export_heightmap2_raw(&hm);
        assert_eq!(b.len(), 4 * 4);
    }

    #[test]
    fn test_to_r16() {
        let hm = sample_hm();
        let r = heightmap2_to_r16(&hm);
        assert_eq!(r.len(), 4);
        assert_eq!(r[0], 0);
        assert_eq!(r[2], 65535);
    }

    #[test]
    fn test_normalize() {
        let mut hm = sample_hm();
        heightmap2_normalize(&mut hm);
        assert!((heightmap2_min(&hm)).abs() < 1e-5);
        assert!((heightmap2_max(&hm) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_min() {
        let hm = sample_hm();
        assert!((heightmap2_min(&hm)).abs() < 1e-5);
    }

    #[test]
    fn test_max() {
        let hm = sample_hm();
        assert!((heightmap2_max(&hm) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_string() {
        let hm = sample_hm();
        let s = heightmap2_to_string(&hm);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_vertex_count() {
        let hm = sample_hm();
        assert_eq!(heightmap2_vertex_count(&hm), 4);
    }

    #[test]
    fn test_cell_count() {
        let hm = sample_hm();
        assert_eq!(heightmap2_cell_count(&hm), 1);
    }

    #[test]
    fn test_heightmap_struct() {
        let hm = HeightmapExport2 { data: vec![1.0], width: 1, height: 1 };
        assert_eq!(hm.data.len(), 1);
    }
}
