#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export LOD group configuration.

#[allow(dead_code)]
pub struct LodLevel {
    pub mesh_name: String,
    pub screen_size: f32,
    pub triangle_count: u32,
}

#[allow(dead_code)]
pub struct LodGroupExport {
    pub name: String,
    pub levels: Vec<LodLevel>,
}

#[allow(dead_code)]
pub fn new_lod_group_export(name: &str) -> LodGroupExport {
    LodGroupExport {
        name: name.to_string(),
        levels: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_lod_level(exp: &mut LodGroupExport, mesh: &str, screen_size: f32, tris: u32) {
    exp.levels.push(LodLevel {
        mesh_name: mesh.to_string(),
        screen_size,
        triangle_count: tris,
    });
}

#[allow(dead_code)]
pub fn export_lod_group_to_json(exp: &LodGroupExport) -> String {
    let mut s = format!("{{\"name\":\"{}\",\"levels\":[", exp.name);
    for (i, lv) in exp.levels.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"mesh\":\"{}\",\"screen_size\":{},\"triangles\":{}}}",
            lv.mesh_name, lv.screen_size, lv.triangle_count
        ));
    }
    s.push_str("]}");
    s
}

#[allow(dead_code)]
pub fn level_count(exp: &LodGroupExport) -> usize {
    exp.levels.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> LodGroupExport {
        let mut exp = new_lod_group_export("Character");
        add_lod_level(&mut exp, "char_lod0", 1.0, 10000);
        add_lod_level(&mut exp, "char_lod1", 0.5, 5000);
        add_lod_level(&mut exp, "char_lod2", 0.1, 1000);
        exp
    }

    #[test]
    fn new_export_has_no_levels() {
        let exp = new_lod_group_export("test");
        assert_eq!(level_count(&exp), 0);
    }

    #[test]
    fn add_level_increases_count() {
        let mut exp = new_lod_group_export("test");
        add_lod_level(&mut exp, "mesh0", 1.0, 5000);
        assert_eq!(level_count(&exp), 1);
    }

    #[test]
    fn name_preserved() {
        let exp = new_lod_group_export("MyChar");
        assert_eq!(exp.name, "MyChar");
    }

    #[test]
    fn three_levels() {
        let exp = sample();
        assert_eq!(level_count(&exp), 3);
    }

    #[test]
    fn mesh_name_stored() {
        let exp = sample();
        assert_eq!(exp.levels[0].mesh_name, "char_lod0");
    }

    #[test]
    fn triangle_count_stored() {
        let exp = sample();
        assert_eq!(exp.levels[2].triangle_count, 1000);
    }

    #[test]
    fn screen_size_stored() {
        let exp = sample();
        assert!((exp.levels[1].screen_size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn json_contains_name() {
        let exp = sample();
        let json = export_lod_group_to_json(&exp);
        assert!(json.contains("Character"));
    }

    #[test]
    fn json_contains_mesh_name() {
        let exp = sample();
        let json = export_lod_group_to_json(&exp);
        assert!(json.contains("char_lod0"));
    }

    #[test]
    fn json_valid_brackets() {
        let exp = sample();
        let json = export_lod_group_to_json(&exp);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }
}
