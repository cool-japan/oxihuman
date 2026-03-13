#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export mesh cache (per-frame vertex positions).

#[allow(dead_code)]
pub struct MeshCacheFrame {
    pub frame: u32,
    pub positions: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub struct MeshCacheExport {
    pub name: String,
    pub frames: Vec<MeshCacheFrame>,
}

#[allow(dead_code)]
pub fn new_mesh_cache_export(name: &str) -> MeshCacheExport {
    MeshCacheExport { name: name.to_string(), frames: vec![] }
}

#[allow(dead_code)]
pub fn add_frame(exp: &mut MeshCacheExport, frame: u32, positions: Vec<[f32; 3]>) {
    exp.frames.push(MeshCacheFrame { frame, positions });
}

#[allow(dead_code)]
pub fn export_mesh_cache_to_json(exp: &MeshCacheExport) -> String {
    let frames_str: Vec<String> = exp.frames.iter().map(|f| {
        let pos_str: Vec<String> = f.positions.iter().map(|p| {
            format!("[{},{},{}]", p[0], p[1], p[2])
        }).collect();
        format!(r#"{{"frame":{},"positions":[{}]}}"#, f.frame, pos_str.join(","))
    }).collect();
    format!(r#"{{"name":"{}","frames":[{}]}}"#, exp.name, frames_str.join(","))
}

#[allow(dead_code)]
pub fn frame_count(exp: &MeshCacheExport) -> usize {
    exp.frames.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cache_has_name() {
        let c = new_mesh_cache_export("deform");
        assert_eq!(c.name, "deform");
    }

    #[test]
    fn new_cache_empty_frames() {
        let c = new_mesh_cache_export("x");
        assert_eq!(frame_count(&c), 0);
    }

    #[test]
    fn add_frame_increments() {
        let mut c = new_mesh_cache_export("x");
        add_frame(&mut c, 0, vec![[0.0, 0.0, 0.0]]);
        assert_eq!(frame_count(&c), 1);
    }

    #[test]
    fn frame_number_stored() {
        let mut c = new_mesh_cache_export("x");
        add_frame(&mut c, 10, vec![]);
        assert_eq!(c.frames[0].frame, 10);
    }

    #[test]
    fn positions_stored() {
        let mut c = new_mesh_cache_export("x");
        add_frame(&mut c, 0, vec![[1.0, 2.0, 3.0]]);
        assert_eq!(c.frames[0].positions.len(), 1);
        assert!((c.frames[0].positions[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn export_json_contains_name() {
        let c = new_mesh_cache_export("sim_cache");
        let json = export_mesh_cache_to_json(&c);
        assert!(json.contains("sim_cache"));
    }

    #[test]
    fn export_json_empty_frames() {
        let c = new_mesh_cache_export("x");
        let json = export_mesh_cache_to_json(&c);
        assert!(json.contains("frames"));
    }

    #[test]
    fn multiple_frames() {
        let mut c = new_mesh_cache_export("anim");
        add_frame(&mut c, 0, vec![[0.0; 3]]);
        add_frame(&mut c, 1, vec![[1.0, 0.0, 0.0]]);
        assert_eq!(frame_count(&c), 2);
    }

    #[test]
    fn positions_per_frame_count() {
        let mut c = new_mesh_cache_export("x");
        let pos = vec![[0.0; 3]; 10];
        add_frame(&mut c, 0, pos);
        assert_eq!(c.frames[0].positions.len(), 10);
    }
}
