// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// LOD level data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct LodLevelData {
    vertex_count: usize,
    switch_distance: f32,
}

/// Renderer that selects the appropriate LOD level for a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshLodRenderer {
    levels: Vec<LodLevelData>,
    current: usize,
}

/// Create a new LOD renderer with the given number of levels.
#[allow(dead_code)]
pub fn new_mesh_lod_renderer(level_count: usize) -> MeshLodRenderer {
    let mut levels = Vec::new();
    for i in 0..level_count.max(1) {
        levels.push(LodLevelData {
            vertex_count: (level_count - i) * 1000,
            switch_distance: (i as f32 + 1.0) * 20.0,
        });
    }
    MeshLodRenderer { levels, current: 0 }
}

/// Select the LOD level based on distance.
#[allow(dead_code)]
pub fn select_lod_level(renderer: &mut MeshLodRenderer, distance: f32) -> usize {
    let mut chosen = 0;
    for (i, level) in renderer.levels.iter().enumerate() {
        if distance >= level.switch_distance {
            chosen = i;
        }
    }
    renderer.current = chosen;
    chosen
}

/// Return the current LOD level.
#[allow(dead_code)]
pub fn current_lod(renderer: &MeshLodRenderer) -> usize {
    renderer.current
}

/// Return the vertex count at a given LOD level.
#[allow(dead_code)]
pub fn lod_vertex_count_mlr(renderer: &MeshLodRenderer, level: usize) -> usize {
    renderer.levels.get(level).map_or(0, |l| l.vertex_count)
}

/// Return the switch distance for a given LOD level.
#[allow(dead_code)]
pub fn lod_switch_distance(renderer: &MeshLodRenderer, level: usize) -> f32 {
    renderer.levels.get(level).map_or(0.0, |l| l.switch_distance)
}

/// Return the total number of LOD levels.
#[allow(dead_code)]
pub fn lod_count_mlr(renderer: &MeshLodRenderer) -> usize {
    renderer.levels.len()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn lod_renderer_to_json(renderer: &MeshLodRenderer) -> String {
    let levels: Vec<String> = renderer
        .levels
        .iter()
        .map(|l| {
            format!(
                "{{\"verts\":{},\"dist\":{:.4}}}",
                l.vertex_count, l.switch_distance
            )
        })
        .collect();
    format!(
        "{{\"current\":{},\"levels\":[{}]}}",
        renderer.current,
        levels.join(",")
    )
}

/// Reset to LOD 0.
#[allow(dead_code)]
pub fn lod_renderer_reset(renderer: &mut MeshLodRenderer) {
    renderer.current = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_renderer() {
        let r = new_mesh_lod_renderer(3);
        assert_eq!(lod_count_mlr(&r), 3);
    }

    #[test]
    fn select_close() {
        let mut r = new_mesh_lod_renderer(3);
        let level = select_lod_level(&mut r, 0.0);
        assert_eq!(level, 0);
    }

    #[test]
    fn select_far() {
        let mut r = new_mesh_lod_renderer(3);
        let level = select_lod_level(&mut r, 100.0);
        assert!(level > 0);
    }

    #[test]
    fn current_lod_tracks() {
        let mut r = new_mesh_lod_renderer(3);
        select_lod_level(&mut r, 50.0);
        assert!(current_lod(&r) > 0);
    }

    #[test]
    fn vertex_count() {
        let r = new_mesh_lod_renderer(3);
        assert!(lod_vertex_count_mlr(&r, 0) > 0);
    }

    #[test]
    fn switch_distance() {
        let r = new_mesh_lod_renderer(3);
        assert!(lod_switch_distance(&r, 0) > 0.0);
    }

    #[test]
    fn to_json() {
        let r = new_mesh_lod_renderer(2);
        let j = lod_renderer_to_json(&r);
        assert!(j.contains("\"current\""));
    }

    #[test]
    fn reset() {
        let mut r = new_mesh_lod_renderer(3);
        select_lod_level(&mut r, 100.0);
        lod_renderer_reset(&mut r);
        assert_eq!(current_lod(&r), 0);
    }

    #[test]
    fn single_level() {
        let r = new_mesh_lod_renderer(1);
        assert_eq!(lod_count_mlr(&r), 1);
    }

    #[test]
    fn invalid_level_vertex_count() {
        let r = new_mesh_lod_renderer(2);
        assert_eq!(lod_vertex_count_mlr(&r, 999), 0);
    }
}
