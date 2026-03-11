#![allow(dead_code)]

//! Texture atlas v2 with automatic packing.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub page: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextureAtlasV2 {
    pub page_width: u32,
    pub page_height: u32,
    pub pages: u32,
    pub regions: Vec<(String, AtlasRegion)>,
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub row_height: u32,
    pub current_page: u32,
    pub padding: u32,
}

#[allow(dead_code)]
pub fn new_texture_atlas_v2(page_width: u32, page_height: u32, padding: u32) -> TextureAtlasV2 {
    TextureAtlasV2 {
        page_width,
        page_height,
        pages: 1,
        regions: Vec::new(),
        cursor_x: padding,
        cursor_y: padding,
        row_height: 0,
        current_page: 0,
        padding,
    }
}

#[allow(dead_code)]
pub fn tav2_pack(atlas: &mut TextureAtlasV2, name: &str, width: u32, height: u32) -> Option<AtlasRegion> {
    let w = width + atlas.padding;
    let h = height + atlas.padding;
    if atlas.cursor_x + w > atlas.page_width {
        atlas.cursor_x = atlas.padding;
        atlas.cursor_y += atlas.row_height + atlas.padding;
        atlas.row_height = 0;
    }
    if atlas.cursor_y + h > atlas.page_height {
        atlas.current_page += 1;
        atlas.pages = atlas.pages.max(atlas.current_page + 1);
        atlas.cursor_x = atlas.padding;
        atlas.cursor_y = atlas.padding;
        atlas.row_height = 0;
    }
    let region = AtlasRegion {
        x: atlas.cursor_x,
        y: atlas.cursor_y,
        width,
        height,
        page: atlas.current_page,
    };
    atlas.cursor_x += w;
    if h > atlas.row_height + atlas.padding {
        atlas.row_height = h.saturating_sub(atlas.padding);
    }
    atlas.regions.push((name.to_string(), region));
    Some(region)
}

#[allow(dead_code)]
pub fn tav2_find(atlas: &TextureAtlasV2, name: &str) -> Option<AtlasRegion> {
    atlas.regions.iter().find(|(n, _)| n == name).map(|(_, r)| *r)
}

#[allow(dead_code)]
pub fn tav2_region_count(atlas: &TextureAtlasV2) -> usize {
    atlas.regions.len()
}

#[allow(dead_code)]
pub fn tav2_page_count(atlas: &TextureAtlasV2) -> u32 {
    atlas.pages
}

#[allow(dead_code)]
pub fn tav2_clear(atlas: &mut TextureAtlasV2) {
    atlas.regions.clear();
    atlas.cursor_x = atlas.padding;
    atlas.cursor_y = atlas.padding;
    atlas.row_height = 0;
    atlas.current_page = 0;
    atlas.pages = 1;
}

#[allow(dead_code)]
pub fn tav2_uv(atlas: &TextureAtlasV2, name: &str) -> Option<[f32; 4]> {
    let r = tav2_find(atlas, name)?;
    let u0 = r.x as f32 / atlas.page_width as f32;
    let v0 = r.y as f32 / atlas.page_height as f32;
    let u1 = (r.x + r.width) as f32 / atlas.page_width as f32;
    let v1 = (r.y + r.height) as f32 / atlas.page_height as f32;
    Some([u0, v0, u1, v1])
}

#[allow(dead_code)]
pub fn tav2_to_json(atlas: &TextureAtlasV2) -> String {
    format!(
        "{{\"page_width\":{},\"page_height\":{},\"pages\":{},\"region_count\":{}}}",
        atlas.page_width,
        atlas.page_height,
        atlas.pages,
        atlas.regions.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_atlas() {
        let a = new_texture_atlas_v2(1024, 1024, 2);
        assert_eq!(tav2_region_count(&a), 0);
        assert_eq!(tav2_page_count(&a), 1);
    }

    #[test]
    fn test_pack_single() {
        let mut a = new_texture_atlas_v2(1024, 1024, 2);
        let r = tav2_pack(&mut a, "icon", 64, 64);
        assert!(r.is_some());
        assert_eq!(tav2_region_count(&a), 1);
    }

    #[test]
    fn test_find_region() {
        let mut a = new_texture_atlas_v2(1024, 1024, 2);
        tav2_pack(&mut a, "logo", 128, 128);
        assert!(tav2_find(&a, "logo").is_some());
    }

    #[test]
    fn test_find_nonexistent() {
        let a = new_texture_atlas_v2(1024, 1024, 2);
        assert!(tav2_find(&a, "missing").is_none());
    }

    #[test]
    fn test_uv_coords() {
        let mut a = new_texture_atlas_v2(1024, 1024, 0);
        tav2_pack(&mut a, "tex", 512, 512);
        let uv = tav2_uv(&a, "tex");
        assert!(uv.is_some());
        let uv = uv.unwrap();
        assert!((0.0..=1.0).contains(&uv[0]));
    }

    #[test]
    fn test_clear() {
        let mut a = new_texture_atlas_v2(1024, 1024, 2);
        tav2_pack(&mut a, "x", 64, 64);
        tav2_clear(&mut a);
        assert_eq!(tav2_region_count(&a), 0);
    }

    #[test]
    fn test_multiple_packs() {
        let mut a = new_texture_atlas_v2(1024, 1024, 2);
        tav2_pack(&mut a, "a", 64, 64);
        tav2_pack(&mut a, "b", 64, 64);
        assert_eq!(tav2_region_count(&a), 2);
    }

    #[test]
    fn test_page_overflow() {
        let mut a = new_texture_atlas_v2(128, 128, 0);
        tav2_pack(&mut a, "big1", 128, 128);
        tav2_pack(&mut a, "big2", 128, 128);
        assert!(tav2_page_count(&a) >= 2);
    }

    #[test]
    fn test_to_json() {
        let a = new_texture_atlas_v2(512, 512, 1);
        let json = tav2_to_json(&a);
        assert!(json.contains("page_width"));
    }

    #[test]
    fn test_region_on_correct_page() {
        let mut a = new_texture_atlas_v2(1024, 1024, 0);
        let r = tav2_pack(&mut a, "first", 64, 64).unwrap();
        assert_eq!(r.page, 0);
    }
}
