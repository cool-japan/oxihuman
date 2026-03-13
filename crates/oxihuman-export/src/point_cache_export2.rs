#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export particle/point cache.

#[allow(dead_code)]
pub struct PointCacheFrame2 {
    pub frame: u32,
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub struct PointCacheExport2 {
    pub name: String,
    pub frames: Vec<PointCacheFrame2>,
}

#[allow(dead_code)]
pub fn new_point_cache_export(name: &str) -> PointCacheExport2 {
    PointCacheExport2 { name: name.to_string(), frames: vec![] }
}

#[allow(dead_code)]
pub fn add_frame(exp: &mut PointCacheExport2, frame: u32, pos: Vec<[f32; 3]>, vel: Vec<[f32; 3]>) {
    exp.frames.push(PointCacheFrame2 { frame, positions: pos, velocities: vel });
}

#[allow(dead_code)]
pub fn export_point_cache_to_json(exp: &PointCacheExport2) -> String {
    let frames_str: Vec<String> = exp.frames.iter().map(|f| {
        format!(r#"{{"frame":{},"points":{}}}"#, f.frame, f.positions.len())
    }).collect();
    format!(r#"{{"name":"{}","frames":[{}]}}"#, exp.name, frames_str.join(","))
}

#[allow(dead_code)]
pub fn total_points(exp: &PointCacheExport2) -> usize {
    exp.frames.iter().map(|f| f.positions.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cache_has_name() {
        let c = new_point_cache_export("particles");
        assert_eq!(c.name, "particles");
    }

    #[test]
    fn new_cache_empty() {
        let c = new_point_cache_export("x");
        assert_eq!(c.frames.len(), 0);
    }

    #[test]
    fn add_frame_increments() {
        let mut c = new_point_cache_export("x");
        add_frame(&mut c, 0, vec![[0.0; 3]], vec![[0.0; 3]]);
        assert_eq!(c.frames.len(), 1);
    }

    #[test]
    fn frame_number_stored() {
        let mut c = new_point_cache_export("x");
        add_frame(&mut c, 5, vec![], vec![]);
        assert_eq!(c.frames[0].frame, 5);
    }

    #[test]
    fn positions_stored() {
        let mut c = new_point_cache_export("x");
        add_frame(&mut c, 0, vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], vec![]);
        assert_eq!(c.frames[0].positions.len(), 2);
    }

    #[test]
    fn velocities_stored() {
        let mut c = new_point_cache_export("x");
        add_frame(&mut c, 0, vec![], vec![[0.0, 1.0, 0.0]]);
        assert_eq!(c.frames[0].velocities.len(), 1);
    }

    #[test]
    fn total_points_sums_all_frames() {
        let mut c = new_point_cache_export("x");
        add_frame(&mut c, 0, vec![[0.0; 3]; 5], vec![]);
        add_frame(&mut c, 1, vec![[0.0; 3]; 3], vec![]);
        assert_eq!(total_points(&c), 8);
    }

    #[test]
    fn export_json_contains_name() {
        let c = new_point_cache_export("my_cache");
        let json = export_point_cache_to_json(&c);
        assert!(json.contains("my_cache"));
    }

    #[test]
    fn export_json_empty_frames() {
        let c = new_point_cache_export("x");
        let json = export_point_cache_to_json(&c);
        assert!(json.contains("frames"));
    }
}
