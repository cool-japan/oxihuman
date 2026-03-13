// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Physics simulation cache export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsFrameData {
    pub frame: u32,
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsCacheExport {
    pub object_name: String,
    pub frames: Vec<PhysicsFrameData>,
}

#[allow(dead_code)]
pub fn new_physics_cache_export(object_name: &str) -> PhysicsCacheExport {
    PhysicsCacheExport { object_name: object_name.to_string(), frames: Vec::new() }
}

#[allow(dead_code)]
pub fn pce_add_frame(exp: &mut PhysicsCacheExport, frame: PhysicsFrameData) {
    exp.frames.push(frame);
}

#[allow(dead_code)]
pub fn pce_frame_count(exp: &PhysicsCacheExport) -> usize {
    exp.frames.len()
}

#[allow(dead_code)]
pub fn pce_get_frame(exp: &PhysicsCacheExport, index: usize) -> Option<&PhysicsFrameData> {
    exp.frames.get(index)
}

#[allow(dead_code)]
pub fn pce_total_particles(exp: &PhysicsCacheExport) -> usize {
    exp.frames.first().map(|f| f.positions.len()).unwrap_or(0)
}

#[allow(dead_code)]
pub fn pce_to_json(exp: &PhysicsCacheExport) -> String {
    format!(
        r#"{{"object":"{}","frames":{},"particles":{}}}"#,
        exp.object_name,
        exp.frames.len(),
        pce_total_particles(exp)
    )
}

#[allow(dead_code)]
pub fn pce_validate(exp: &PhysicsCacheExport) -> bool {
    !exp.object_name.is_empty()
        && exp.frames.iter().all(|f| f.positions.len() == f.velocities.len())
}

#[allow(dead_code)]
pub fn pce_clear(exp: &mut PhysicsCacheExport) {
    exp.frames.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(frame: u32, n: usize) -> PhysicsFrameData {
        PhysicsFrameData {
            frame,
            positions: vec![[0.0; 3]; n],
            velocities: vec![[0.0; 3]; n],
        }
    }

    #[test]
    fn new_export_empty() {
        let exp = new_physics_cache_export("cloth");
        assert_eq!(pce_frame_count(&exp), 0);
    }

    #[test]
    fn add_frame_increments() {
        let mut exp = new_physics_cache_export("cloth");
        pce_add_frame(&mut exp, make_frame(0, 10));
        assert_eq!(pce_frame_count(&exp), 1);
    }

    #[test]
    fn get_frame_by_index() {
        let mut exp = new_physics_cache_export("cloth");
        pce_add_frame(&mut exp, make_frame(5, 4));
        let f = pce_get_frame(&exp, 0).expect("should succeed");
        assert_eq!(f.frame, 5);
    }

    #[test]
    fn total_particles_from_first_frame() {
        let mut exp = new_physics_cache_export("cloth");
        pce_add_frame(&mut exp, make_frame(0, 100));
        assert_eq!(pce_total_particles(&exp), 100);
    }

    #[test]
    fn validate_ok() {
        let mut exp = new_physics_cache_export("obj");
        pce_add_frame(&mut exp, make_frame(0, 5));
        assert!(pce_validate(&exp));
    }

    #[test]
    fn validate_fails_empty_name() {
        let exp = new_physics_cache_export("");
        assert!(!pce_validate(&exp));
    }

    #[test]
    fn clear_removes_frames() {
        let mut exp = new_physics_cache_export("obj");
        pce_add_frame(&mut exp, make_frame(0, 2));
        pce_clear(&mut exp);
        assert_eq!(pce_frame_count(&exp), 0);
    }

    #[test]
    fn to_json_has_object() {
        let exp = new_physics_cache_export("my_obj");
        let json = pce_to_json(&exp);
        assert!(json.contains("my_obj"));
        assert!(json.contains("frames"));
    }
}
