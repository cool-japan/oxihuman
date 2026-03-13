// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Geometry cache v2 export: extended geometry cache format with normals and UVs.

/// Magic bytes for format identification.
pub const GCV2_MAGIC: &[u8; 4] = b"GCV2";
/// Current format version.
pub const GCV2_VERSION: u32 = 2;

/// A single frame in the geometry cache v2.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeoCacheV2Frame {
    pub time: f32,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
}

/// The full geometry cache v2 export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeoCacheV2Export {
    pub frames: Vec<GeoCacheV2Frame>,
    pub vertex_count: usize,
}

/// Create a new empty v2 cache.
#[allow(dead_code)]
pub fn new_geo_cache_v2(vertex_count: usize) -> GeoCacheV2Export {
    GeoCacheV2Export {
        frames: Vec::new(),
        vertex_count,
    }
}

/// Add a frame.
#[allow(dead_code)]
pub fn add_geo_v2_frame(
    cache: &mut GeoCacheV2Export,
    time: f32,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
) {
    cache.frames.push(GeoCacheV2Frame {
        time,
        positions,
        normals,
        uvs,
    });
}

/// Frame count.
#[allow(dead_code)]
pub fn geo_v2_frame_count(cache: &GeoCacheV2Export) -> usize {
    cache.frames.len()
}

/// Duration of the cache.
#[allow(dead_code)]
pub fn geo_v2_duration(cache: &GeoCacheV2Export) -> f32 {
    cache.frames.iter().map(|f| f.time).fold(0.0_f32, f32::max)
}

/// Validate: all frames have matching vertex count.
#[allow(dead_code)]
pub fn validate_geo_cache_v2(cache: &GeoCacheV2Export) -> bool {
    cache.frames.iter().all(|f| {
        f.positions.len() == cache.vertex_count
            && (f.normals.is_empty() || f.normals.len() == cache.vertex_count)
            && (f.uvs.is_empty() || f.uvs.len() == cache.vertex_count)
    })
}

/// Estimated byte size.
#[allow(dead_code)]
pub fn geo_v2_size_bytes(cache: &GeoCacheV2Export) -> usize {
    cache
        .frames
        .iter()
        .map(|f| f.positions.len() * 12 + f.normals.len() * 12 + f.uvs.len() * 8)
        .sum::<usize>()
        + 8
}

/// Binary header bytes.
#[allow(dead_code)]
pub fn geo_v2_header_bytes() -> Vec<u8> {
    let mut h = GCV2_MAGIC.to_vec();
    h.extend_from_slice(&GCV2_VERSION.to_le_bytes());
    h
}

/// Export to JSON.
#[allow(dead_code)]
pub fn geo_cache_v2_to_json(cache: &GeoCacheV2Export) -> String {
    format!(
        "{{\"version\":{},\"vertex_count\":{},\"frame_count\":{}}}",
        GCV2_VERSION,
        cache.vertex_count,
        geo_v2_frame_count(cache)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_cache() -> GeoCacheV2Export {
        let mut c = new_geo_cache_v2(3);
        add_geo_v2_frame(
            &mut c,
            0.0,
            vec![[0.0; 3]; 3],
            vec![[0.0, 0.0, 1.0]; 3],
            vec![[0.0; 2]; 3],
        );
        c
    }

    #[test]
    fn test_new_cache() {
        let c = new_geo_cache_v2(10);
        assert_eq!(c.vertex_count, 10);
        assert_eq!(geo_v2_frame_count(&c), 0);
    }

    #[test]
    fn test_add_frame() {
        let c = simple_cache();
        assert_eq!(geo_v2_frame_count(&c), 1);
    }

    #[test]
    fn test_geo_v2_duration() {
        let mut c = new_geo_cache_v2(3);
        add_geo_v2_frame(&mut c, 1.5, vec![[0.0; 3]; 3], vec![], vec![]);
        assert!((geo_v2_duration(&c) - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_validate_valid() {
        let c = simple_cache();
        assert!(validate_geo_cache_v2(&c));
    }

    #[test]
    fn test_validate_wrong_vertex_count() {
        let mut c = new_geo_cache_v2(3);
        add_geo_v2_frame(&mut c, 0.0, vec![[0.0; 3]; 2], vec![], vec![]);
        assert!(!validate_geo_cache_v2(&c));
    }

    #[test]
    fn test_geo_v2_size_bytes() {
        let c = simple_cache();
        let sz = geo_v2_size_bytes(&c);
        assert!(sz > 0);
    }

    #[test]
    fn test_header_bytes() {
        let h = geo_v2_header_bytes();
        assert!(h.starts_with(b"GCV2"));
    }

    #[test]
    fn test_geo_cache_v2_to_json() {
        let c = simple_cache();
        let j = geo_cache_v2_to_json(&c);
        assert!(j.contains("\"version\":2"));
    }

    #[test]
    fn test_duration_empty() {
        let c = new_geo_cache_v2(0);
        assert!((geo_v2_duration(&c)).abs() < 1e-9);
    }

    #[test]
    fn test_magic_bytes() {
        assert_eq!(GCV2_MAGIC, b"GCV2");
    }
}
