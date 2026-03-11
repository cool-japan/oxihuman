#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export occlusion culling data (occluder meshes).

#[allow(dead_code)]
pub struct OccluderMesh {
    pub name: String,
    pub vertex_count: u32,
    pub tri_count: u32,
    pub simplified: bool,
}

#[allow(dead_code)]
pub struct OcclusionCullingExport {
    pub occluders: Vec<OccluderMesh>,
}

#[allow(dead_code)]
pub fn new_occlusion_culling_export() -> OcclusionCullingExport {
    OcclusionCullingExport { occluders: Vec::new() }
}

#[allow(dead_code)]
pub fn add_occluder(
    exp: &mut OcclusionCullingExport,
    name: &str,
    verts: u32,
    tris: u32,
    simplified: bool,
) {
    exp.occluders.push(OccluderMesh {
        name: name.to_string(),
        vertex_count: verts,
        tri_count: tris,
        simplified,
    });
}

#[allow(dead_code)]
pub fn export_occlusion_culling_to_json(exp: &OcclusionCullingExport) -> String {
    let mut s = "{\"occluders\":[".to_string();
    for (i, o) in exp.occluders.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"vertices\":{},\"triangles\":{},\"simplified\":{}}}",
            o.name, o.vertex_count, o.tri_count, o.simplified
        ));
    }
    s.push_str("]}");
    s
}

#[allow(dead_code)]
pub fn occluder_count(exp: &OcclusionCullingExport) -> usize {
    exp.occluders.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> OcclusionCullingExport {
        let mut exp = new_occlusion_culling_export();
        add_occluder(&mut exp, "wall_a", 8, 4, false);
        add_occluder(&mut exp, "pillar_b", 12, 8, true);
        exp
    }

    #[test]
    fn new_export_empty() {
        let exp = new_occlusion_culling_export();
        assert_eq!(occluder_count(&exp), 0);
    }

    #[test]
    fn add_occluder_increases_count() {
        let mut exp = new_occlusion_culling_export();
        add_occluder(&mut exp, "wall", 4, 2, false);
        assert_eq!(occluder_count(&exp), 1);
    }

    #[test]
    fn occluder_name_preserved() {
        let exp = sample();
        assert_eq!(exp.occluders[0].name, "wall_a");
    }

    #[test]
    fn occluder_vertex_count_stored() {
        let exp = sample();
        assert_eq!(exp.occluders[0].vertex_count, 8);
    }

    #[test]
    fn occluder_tri_count_stored() {
        let exp = sample();
        assert_eq!(exp.occluders[0].tri_count, 4);
    }

    #[test]
    fn occluder_simplified_flag() {
        let exp = sample();
        assert!(!exp.occluders[0].simplified);
        assert!(exp.occluders[1].simplified);
    }

    #[test]
    fn two_occluders() {
        let exp = sample();
        assert_eq!(occluder_count(&exp), 2);
    }

    #[test]
    fn json_contains_occluders() {
        let exp = sample();
        let json = export_occlusion_culling_to_json(&exp);
        assert!(json.contains("occluders"));
    }

    #[test]
    fn json_contains_occluder_name() {
        let exp = sample();
        let json = export_occlusion_culling_to_json(&exp);
        assert!(json.contains("wall_a"));
    }

    #[test]
    fn json_valid_brackets() {
        let exp = sample();
        let json = export_occlusion_culling_to_json(&exp);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }
}
