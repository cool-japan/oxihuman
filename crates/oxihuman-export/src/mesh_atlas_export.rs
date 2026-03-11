// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export mesh texture atlas layouts with UV island placement data.

/// UV island in an atlas.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AtlasIsland {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub face_count: u32,
}

/// Atlas layout export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshAtlasExport {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub islands: Vec<AtlasIsland>,
}

#[allow(dead_code)]
pub fn new_mesh_atlas_export(w: u32, h: u32) -> MeshAtlasExport {
    MeshAtlasExport { atlas_width: w, atlas_height: h, islands: Vec::new() }
}

#[allow(dead_code)]
pub fn atlas_add_island(export: &mut MeshAtlasExport, id: u32, x: f32, y: f32, w: f32, h: f32, faces: u32) -> usize {
    let idx = export.islands.len();
    export.islands.push(AtlasIsland { id, x, y, width: w, height: h, face_count: faces });
    idx
}

#[allow(dead_code)]
pub fn atlas_island_count(export: &MeshAtlasExport) -> usize { export.islands.len() }

#[allow(dead_code)]
pub fn atlas_utilization(export: &MeshAtlasExport) -> f32 {
    let total = export.atlas_width as f32 * export.atlas_height as f32;
    if total < 1e-12 { return 0.0; }
    let used: f32 = export.islands.iter().map(|i| i.width * i.height).sum();
    (used / total).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn atlas_total_faces(export: &MeshAtlasExport) -> u32 {
    export.islands.iter().map(|i| i.face_count).sum()
}

#[allow(dead_code)]
pub fn atlas_find_island(export: &MeshAtlasExport, id: u32) -> Option<usize> {
    export.islands.iter().position(|i| i.id == id)
}

#[allow(dead_code)]
pub fn atlas_to_json(export: &MeshAtlasExport) -> String {
    let islands: Vec<String> = export.islands.iter().map(|i| {
        format!(r#"{{"id":{},"x":{:.2},"y":{:.2},"w":{:.2},"h":{:.2},"faces":{}}}"#,
            i.id, i.x, i.y, i.width, i.height, i.face_count)
    }).collect();
    format!(r#"{{"width":{},"height":{},"islands":[{}]}}"#, export.atlas_width, export.atlas_height, islands.join(","))
}

#[allow(dead_code)]
pub fn atlas_largest_island(export: &MeshAtlasExport) -> Option<usize> {
    export.islands.iter().enumerate()
        .max_by(|(_, a), (_, b)| (a.width * a.height).partial_cmp(&(b.width * b.height)).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
}

#[allow(dead_code)]
pub fn atlas_validate(export: &MeshAtlasExport) -> bool {
    let aw = export.atlas_width as f32;
    let ah = export.atlas_height as f32;
    export.islands.iter().all(|i| {
        i.x >= 0.0 && i.y >= 0.0 && i.x + i.width <= aw && i.y + i.height <= ah
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_atlas() {
        let a = new_mesh_atlas_export(1024, 1024);
        assert_eq!(atlas_island_count(&a), 0);
    }

    #[test]
    fn test_add_island() {
        let mut a = new_mesh_atlas_export(1024, 1024);
        atlas_add_island(&mut a, 0, 0.0, 0.0, 100.0, 100.0, 10);
        assert_eq!(atlas_island_count(&a), 1);
    }

    #[test]
    fn test_utilization() {
        let mut a = new_mesh_atlas_export(100, 100);
        atlas_add_island(&mut a, 0, 0.0, 0.0, 50.0, 50.0, 5);
        assert!((atlas_utilization(&a) - 0.25).abs() < 1e-4);
    }

    #[test]
    fn test_total_faces() {
        let mut a = new_mesh_atlas_export(100, 100);
        atlas_add_island(&mut a, 0, 0.0, 0.0, 10.0, 10.0, 5);
        atlas_add_island(&mut a, 1, 20.0, 0.0, 10.0, 10.0, 3);
        assert_eq!(atlas_total_faces(&a), 8);
    }

    #[test]
    fn test_find_island() {
        let mut a = new_mesh_atlas_export(100, 100);
        atlas_add_island(&mut a, 42, 0.0, 0.0, 10.0, 10.0, 1);
        assert_eq!(atlas_find_island(&a, 42), Some(0));
    }

    #[test]
    fn test_find_missing() {
        let a = new_mesh_atlas_export(100, 100);
        assert_eq!(atlas_find_island(&a, 99), None);
    }

    #[test]
    fn test_to_json() {
        let mut a = new_mesh_atlas_export(512, 512);
        atlas_add_island(&mut a, 0, 0.0, 0.0, 100.0, 100.0, 10);
        let json = atlas_to_json(&a);
        assert!(json.contains("islands"));
    }

    #[test]
    fn test_largest_island() {
        let mut a = new_mesh_atlas_export(100, 100);
        atlas_add_island(&mut a, 0, 0.0, 0.0, 10.0, 10.0, 1);
        atlas_add_island(&mut a, 1, 0.0, 0.0, 50.0, 50.0, 1);
        assert_eq!(atlas_largest_island(&a), Some(1));
    }

    #[test]
    fn test_validate_ok() {
        let mut a = new_mesh_atlas_export(100, 100);
        atlas_add_island(&mut a, 0, 0.0, 0.0, 50.0, 50.0, 1);
        assert!(atlas_validate(&a));
    }

    #[test]
    fn test_validate_fail() {
        let mut a = new_mesh_atlas_export(100, 100);
        atlas_add_island(&mut a, 0, 0.0, 0.0, 200.0, 200.0, 1);
        assert!(!atlas_validate(&a));
    }

}
