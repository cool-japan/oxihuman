#![allow(dead_code)]
//! Texture atlas packing and region lookup.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct AtlasRegion {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TextureAtlas {
    width: u32,
    height: u32,
    regions: Vec<AtlasRegion>,
}

#[allow(dead_code)]
pub fn new_texture_atlas(width: u32, height: u32) -> TextureAtlas {
    TextureAtlas {
        width,
        height,
        regions: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_atlas_region(atlas: &mut TextureAtlas, name: &str, x: u32, y: u32, w: u32, h: u32) {
    atlas.regions.push(AtlasRegion {
        name: name.to_string(),
        x,
        y,
        w,
        h,
    });
}

#[allow(dead_code)]
pub fn region_count(atlas: &TextureAtlas) -> usize {
    atlas.regions.len()
}

#[allow(dead_code)]
pub fn region_uv_rect(atlas: &TextureAtlas, index: usize) -> Option<[f32; 4]> {
    atlas.regions.get(index).map(|r| {
        [
            r.x as f32 / atlas.width as f32,
            r.y as f32 / atlas.height as f32,
            r.w as f32 / atlas.width as f32,
            r.h as f32 / atlas.height as f32,
        ]
    })
}

#[allow(dead_code)]
pub fn atlas_width(atlas: &TextureAtlas) -> u32 {
    atlas.width
}

#[allow(dead_code)]
pub fn atlas_height(atlas: &TextureAtlas) -> u32 {
    atlas.height
}

#[allow(dead_code)]
pub fn atlas_to_json(atlas: &TextureAtlas) -> String {
    let regions: Vec<String> = atlas
        .regions
        .iter()
        .map(|r| {
            format!(
                "{{\"name\":\"{}\",\"x\":{},\"y\":{},\"w\":{},\"h\":{}}}",
                r.name, r.x, r.y, r.w, r.h
            )
        })
        .collect();
    format!(
        "{{\"width\":{},\"height\":{},\"regions\":[{}]}}",
        atlas.width,
        atlas.height,
        regions.join(",")
    )
}

#[allow(dead_code)]
pub fn region_by_name<'a>(atlas: &'a TextureAtlas, name: &str) -> Option<&'a AtlasRegion> {
    atlas.regions.iter().find(|r| r.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_texture_atlas() {
        let a = new_texture_atlas(1024, 1024);
        assert_eq!(atlas_width(&a), 1024);
        assert_eq!(atlas_height(&a), 1024);
    }

    #[test]
    fn test_add_atlas_region() {
        let mut a = new_texture_atlas(256, 256);
        add_atlas_region(&mut a, "tile", 0, 0, 64, 64);
        assert_eq!(region_count(&a), 1);
    }

    #[test]
    fn test_region_uv_rect() {
        let mut a = new_texture_atlas(100, 100);
        add_atlas_region(&mut a, "r", 10, 20, 30, 40);
        let uv = region_uv_rect(&a, 0).expect("should succeed");
        assert!((uv[0] - 0.1).abs() < 1e-6);
        assert!((uv[1] - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_region_uv_rect_none() {
        let a = new_texture_atlas(100, 100);
        assert!(region_uv_rect(&a, 0).is_none());
    }

    #[test]
    fn test_region_by_name() {
        let mut a = new_texture_atlas(256, 256);
        add_atlas_region(&mut a, "icon", 0, 0, 32, 32);
        assert!(region_by_name(&a, "icon").is_some());
        assert!(region_by_name(&a, "nope").is_none());
    }

    #[test]
    fn test_atlas_to_json() {
        let a = new_texture_atlas(512, 512);
        let json = atlas_to_json(&a);
        assert!(json.contains("\"width\":512"));
    }

    #[test]
    fn test_region_count_empty() {
        let a = new_texture_atlas(64, 64);
        assert_eq!(region_count(&a), 0);
    }

    #[test]
    fn test_multiple_regions() {
        let mut a = new_texture_atlas(256, 256);
        for i in 0..4 {
            add_atlas_region(&mut a, &format!("r{i}"), i * 64, 0, 64, 64);
        }
        assert_eq!(region_count(&a), 4);
    }

    #[test]
    fn test_atlas_dimensions() {
        let a = new_texture_atlas(2048, 1024);
        assert_eq!(atlas_width(&a), 2048);
        assert_eq!(atlas_height(&a), 1024);
    }

    #[test]
    fn test_region_by_name_returns_data() {
        let mut a = new_texture_atlas(100, 100);
        add_atlas_region(&mut a, "test", 5, 10, 20, 30);
        let r = region_by_name(&a, "test").expect("should succeed");
        assert_eq!(r.x, 5);
        assert_eq!(r.w, 20);
    }
}
