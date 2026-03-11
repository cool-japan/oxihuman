#![allow(dead_code)]
//! Texture mipmap: represents a texture with multiple mip levels.

/// A texture mipmap chain.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextureMipmap {
    base_width: u32,
    base_height: u32,
    levels: u32,
    bytes_per_pixel: u32,
}

/// Create a new mipmap description.
#[allow(dead_code)]
pub fn new_texture_mipmap(width: u32, height: u32, bytes_per_pixel: u32) -> TextureMipmap {
    let max_dim = width.max(height).max(1);
    let levels = (max_dim as f32).log2().floor() as u32 + 1;
    TextureMipmap {
        base_width: width,
        base_height: height,
        levels,
        bytes_per_pixel,
    }
}

/// Return the number of mip levels.
#[allow(dead_code)]
pub fn mip_level_count(mip: &TextureMipmap) -> u32 {
    mip.levels
}

/// Return the width at a given mip level.
#[allow(dead_code)]
pub fn mip_width_at(mip: &TextureMipmap, level: u32) -> u32 {
    if level >= mip.levels {
        return 0;
    }
    (mip.base_width >> level).max(1)
}

/// Return the height at a given mip level.
#[allow(dead_code)]
pub fn mip_height_at(mip: &TextureMipmap, level: u32) -> u32 {
    if level >= mip.levels {
        return 0;
    }
    (mip.base_height >> level).max(1)
}

/// Return the data size in bytes at a given mip level.
#[allow(dead_code)]
pub fn mip_data_size(mip: &TextureMipmap, level: u32) -> u32 {
    mip_width_at(mip, level) * mip_height_at(mip, level) * mip.bytes_per_pixel
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn mip_to_json(mip: &TextureMipmap) -> String {
    format!(
        "{{\"base_width\":{},\"base_height\":{},\"levels\":{},\"bpp\":{}}}",
        mip.base_width, mip.base_height, mip.levels, mip.bytes_per_pixel
    )
}

/// Stub: generate mipmap (just returns the mipmap metadata, no actual data).
#[allow(dead_code)]
pub fn generate_mipmap_stub(width: u32, height: u32, bpp: u32) -> TextureMipmap {
    new_texture_mipmap(width, height, bpp)
}

/// Return the total size of all mip levels in bytes.
#[allow(dead_code)]
pub fn mip_total_size(mip: &TextureMipmap) -> u32 {
    (0..mip.levels).map(|l| mip_data_size(mip, l)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mipmap() {
        let m = new_texture_mipmap(256, 256, 4);
        assert_eq!(mip_level_count(&m), 9); // log2(256)+1
    }

    #[test]
    fn test_mip_width() {
        let m = new_texture_mipmap(256, 256, 4);
        assert_eq!(mip_width_at(&m, 0), 256);
        assert_eq!(mip_width_at(&m, 1), 128);
    }

    #[test]
    fn test_mip_height() {
        let m = new_texture_mipmap(256, 128, 4);
        assert_eq!(mip_height_at(&m, 0), 128);
        assert_eq!(mip_height_at(&m, 1), 64);
    }

    #[test]
    fn test_mip_min_size() {
        let m = new_texture_mipmap(256, 256, 4);
        let last = mip_level_count(&m) - 1;
        assert_eq!(mip_width_at(&m, last), 1);
    }

    #[test]
    fn test_data_size() {
        let m = new_texture_mipmap(256, 256, 4);
        assert_eq!(mip_data_size(&m, 0), 256 * 256 * 4);
    }

    #[test]
    fn test_out_of_range_level() {
        let m = new_texture_mipmap(256, 256, 4);
        assert_eq!(mip_width_at(&m, 100), 0);
    }

    #[test]
    fn test_to_json() {
        let m = new_texture_mipmap(64, 64, 4);
        let json = mip_to_json(&m);
        assert!(json.contains("\"base_width\":64"));
    }

    #[test]
    fn test_generate_stub() {
        let m = generate_mipmap_stub(128, 128, 4);
        assert!(mip_level_count(&m) > 0);
    }

    #[test]
    fn test_total_size() {
        let m = new_texture_mipmap(4, 4, 1);
        // 4x4=16, 2x2=4, 1x1=1 = 21
        assert_eq!(mip_total_size(&m), 21);
    }

    #[test]
    fn test_1x1_texture() {
        let m = new_texture_mipmap(1, 1, 4);
        assert_eq!(mip_level_count(&m), 1);
        assert_eq!(mip_total_size(&m), 4);
    }
}
