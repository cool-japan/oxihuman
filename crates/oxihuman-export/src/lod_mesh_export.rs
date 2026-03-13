// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-LOD mesh export with quality metrics and size estimates.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodMeshLevel {
    pub level: u32,
    pub triangle_count: u32,
    pub vertex_count: u32,
    pub screen_size_threshold: f32,
    pub mesh_name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodMeshExport {
    pub base_mesh_name: String,
    pub levels: Vec<LodMeshLevel>,
}

#[allow(dead_code)]
pub fn new_lod_mesh_export(base_name: &str) -> LodMeshExport {
    LodMeshExport {
        base_mesh_name: base_name.to_string(),
        levels: Vec::new(),
    }
}

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn add_lod_mesh_level(
    exp: &mut LodMeshExport,
    level: u32,
    tris: u32,
    verts: u32,
    threshold: f32,
) {
    let name = format!("{}_lod{}", exp.base_mesh_name, level);
    exp.levels.push(LodMeshLevel {
        level,
        triangle_count: tris,
        vertex_count: verts,
        screen_size_threshold: threshold,
        mesh_name: name,
    });
}

#[allow(dead_code)]
pub fn lod_level_count(exp: &LodMeshExport) -> usize {
    exp.levels.len()
}

#[allow(dead_code)]
pub fn total_triangle_count(exp: &LodMeshExport) -> u32 {
    exp.levels.iter().map(|l| l.triangle_count).sum()
}

#[allow(dead_code)]
pub fn reduction_ratio(exp: &LodMeshExport) -> f32 {
    if exp.levels.len() < 2 {
        return 1.0;
    }
    let base = exp.levels[0].triangle_count as f32;
    let last = exp.levels.last().map_or(0, |l| l.triangle_count) as f32;
    if base == 0.0 {
        1.0
    } else {
        last / base
    }
}

#[allow(dead_code)]
pub fn find_lod_level(exp: &LodMeshExport, level: u32) -> Option<&LodMeshLevel> {
    exp.levels.iter().find(|l| l.level == level)
}

#[allow(dead_code)]
pub fn lod_mesh_to_json(exp: &LodMeshExport) -> String {
    format!(
        "{{\"base\":\"{}\",\"lod_count\":{}}}",
        exp.base_mesh_name,
        lod_level_count(exp)
    )
}

#[allow(dead_code)]
pub fn lod_levels_sorted(exp: &LodMeshExport) -> bool {
    exp.levels
        .windows(2)
        .all(|pair| pair[0].triangle_count >= pair[1].triangle_count)
}

#[allow(dead_code)]
pub fn sort_lod_levels(exp: &mut LodMeshExport) {
    exp.levels
        .sort_by(|a, b| b.triangle_count.cmp(&a.triangle_count));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> LodMeshExport {
        let mut exp = new_lod_mesh_export("character");
        add_lod_mesh_level(&mut exp, 0, 10000, 8000, 1.0);
        add_lod_mesh_level(&mut exp, 1, 5000, 4000, 0.5);
        add_lod_mesh_level(&mut exp, 2, 1000, 800, 0.1);
        exp
    }

    #[test]
    fn test_empty() {
        let exp = new_lod_mesh_export("base");
        assert_eq!(lod_level_count(&exp), 0);
    }

    #[test]
    fn test_add_levels() {
        let exp = sample();
        assert_eq!(lod_level_count(&exp), 3);
    }

    #[test]
    fn test_find_level() {
        let exp = sample();
        assert!(find_lod_level(&exp, 1).is_some());
    }

    #[test]
    fn test_total_triangles() {
        let exp = sample();
        assert_eq!(total_triangle_count(&exp), 16000);
    }

    #[test]
    fn test_reduction_ratio() {
        let exp = sample();
        let r = reduction_ratio(&exp);
        assert!(r < 1.0);
    }

    #[test]
    fn test_lod_sorted() {
        let exp = sample();
        assert!(lod_levels_sorted(&exp));
    }

    #[test]
    fn test_json_output() {
        let exp = sample();
        let j = lod_mesh_to_json(&exp);
        assert!(j.contains("character"));
    }

    #[test]
    fn test_mesh_name_generated() {
        let exp = sample();
        assert!(exp.levels[0].mesh_name.contains("lod0"));
    }

    #[test]
    fn test_single_level_ratio_one() {
        let mut exp = new_lod_mesh_export("x");
        add_lod_mesh_level(&mut exp, 0, 5000, 4000, 1.0);
        assert!((reduction_ratio(&exp) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_sort_levels() {
        let mut exp = new_lod_mesh_export("x");
        add_lod_mesh_level(&mut exp, 1, 100, 80, 0.1);
        add_lod_mesh_level(&mut exp, 0, 1000, 800, 1.0);
        sort_lod_levels(&mut exp);
        assert!(lod_levels_sorted(&exp));
    }
}
