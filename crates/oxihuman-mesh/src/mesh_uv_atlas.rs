// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! UV atlas management: island tracking, packing stats, and utility helpers.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvAtlasIsland {
    pub id: u32,
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub face_indices: Vec<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvAtlas {
    pub islands: Vec<UvAtlasIsland>,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
pub fn new_uv_atlas(width: u32, height: u32) -> UvAtlas {
    UvAtlas {
        islands: Vec::new(),
        width,
        height,
    }
}

#[allow(dead_code)]
pub fn add_island(atlas: &mut UvAtlas, island: UvAtlasIsland) {
    atlas.islands.push(island);
}

#[allow(dead_code)]
pub fn island_count(atlas: &UvAtlas) -> usize {
    atlas.islands.len()
}

#[allow(dead_code)]
pub fn atlas_utilization(atlas: &UvAtlas) -> f32 {
    if atlas.width == 0 || atlas.height == 0 {
        return 0.0;
    }
    let total = (atlas.width * atlas.height) as f32;
    let used: f32 = atlas
        .islands
        .iter()
        .map(|i| {
            let w = (i.uv_max[0] - i.uv_min[0]).max(0.0) * atlas.width as f32;
            let h = (i.uv_max[1] - i.uv_min[1]).max(0.0) * atlas.height as f32;
            w * h
        })
        .sum();
    (used / total).min(1.0)
}

#[allow(dead_code)]
pub fn largest_island_area(atlas: &UvAtlas) -> f32 {
    atlas
        .islands
        .iter()
        .map(|i| {
            let w = (i.uv_max[0] - i.uv_min[0]).max(0.0);
            let h = (i.uv_max[1] - i.uv_min[1]).max(0.0);
            w * h
        })
        .fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn find_island_by_id(atlas: &UvAtlas, id: u32) -> Option<&UvAtlasIsland> {
    atlas.islands.iter().find(|i| i.id == id)
}

#[allow(dead_code)]
pub fn islands_overlap(a: &UvAtlasIsland, b: &UvAtlasIsland) -> bool {
    a.uv_min[0] < b.uv_max[0]
        && a.uv_max[0] > b.uv_min[0]
        && a.uv_min[1] < b.uv_max[1]
        && a.uv_max[1] > b.uv_min[1]
}

#[allow(dead_code)]
pub fn atlas_to_json(atlas: &UvAtlas) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"island_count\":{}}}",
        atlas.width,
        atlas.height,
        island_count(atlas)
    )
}

#[allow(dead_code)]
pub fn uv_in_atlas_bounds(uv: [f32; 2]) -> bool {
    (0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1])
}

#[allow(dead_code)]
pub fn total_island_faces(atlas: &UvAtlas) -> usize {
    atlas.islands.iter().map(|i| i.face_indices.len()).sum()
}

fn make_island(id: u32, min: [f32; 2], max: [f32; 2]) -> UvAtlasIsland {
    UvAtlasIsland {
        id,
        uv_min: min,
        uv_max: max,
        face_indices: vec![0, 1, 2],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_atlas_empty() {
        let atlas = new_uv_atlas(1024, 1024);
        assert_eq!(island_count(&atlas), 0);
    }

    #[test]
    fn test_add_island() {
        let mut atlas = new_uv_atlas(1024, 1024);
        add_island(&mut atlas, make_island(1, [0.0, 0.0], [0.5, 0.5]));
        assert_eq!(island_count(&atlas), 1);
    }

    #[test]
    fn test_utilization_positive() {
        let mut atlas = new_uv_atlas(1024, 1024);
        add_island(&mut atlas, make_island(1, [0.0, 0.0], [0.5, 0.5]));
        let u = atlas_utilization(&atlas);
        assert!((0.0..=1.0).contains(&u));
    }

    #[test]
    fn test_find_island() {
        let mut atlas = new_uv_atlas(512, 512);
        add_island(&mut atlas, make_island(42, [0.1, 0.1], [0.4, 0.4]));
        assert!(find_island_by_id(&atlas, 42).is_some());
    }

    #[test]
    fn test_find_missing_island() {
        let atlas = new_uv_atlas(512, 512);
        assert!(find_island_by_id(&atlas, 99).is_none());
    }

    #[test]
    fn test_islands_overlap() {
        let a = make_island(1, [0.0, 0.0], [0.5, 0.5]);
        let b = make_island(2, [0.3, 0.3], [0.8, 0.8]);
        assert!(islands_overlap(&a, &b));
    }

    #[test]
    fn test_islands_no_overlap() {
        let a = make_island(1, [0.0, 0.0], [0.3, 0.3]);
        let b = make_island(2, [0.6, 0.6], [0.9, 0.9]);
        assert!(!islands_overlap(&a, &b));
    }

    #[test]
    fn test_uv_in_bounds() {
        assert!(uv_in_atlas_bounds([0.5, 0.5]));
        assert!(!uv_in_atlas_bounds([1.5, 0.5]));
    }

    #[test]
    fn test_json_output() {
        let atlas = new_uv_atlas(1024, 512);
        let j = atlas_to_json(&atlas);
        assert!(j.contains("1024"));
    }

    #[test]
    fn test_total_faces() {
        let mut atlas = new_uv_atlas(1024, 1024);
        add_island(&mut atlas, make_island(1, [0.0, 0.0], [0.5, 0.5]));
        assert_eq!(total_island_faces(&atlas), 3);
    }
}
