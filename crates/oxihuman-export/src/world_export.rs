// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! World/environment export.

/// World environment data.
#[derive(Debug, Clone)]
pub struct WorldData {
    pub name: String,
    pub horizon_color: [f32; 3],
    pub ambient_color: [f32; 3],
    pub ambient_energy: f32,
    pub fog_start: f32,
    pub fog_end: f32,
    pub hdri_path: Option<String>,
}

/// Create a new `WorldData`.
pub fn new_world_data(name: &str) -> WorldData {
    WorldData {
        name: name.to_string(),
        horizon_color: [0.05, 0.05, 0.05],
        ambient_color: [0.0, 0.0, 0.0],
        ambient_energy: 1.0,
        fog_start: 5.0,
        fog_end: 50.0,
        hdri_path: None,
    }
}

/// Serialize to JSON.
pub fn world_to_json(w: &WorldData) -> String {
    format!(
        "{{\"name\":\"{}\",\"ambient_energy\":{},\"fog_start\":{},\"fog_end\":{},\"has_hdri\":{}}}",
        w.name,
        w.ambient_energy,
        w.fog_start,
        w.fog_end,
        w.hdri_path.is_some()
    )
}

/// Returns true if an HDRI path is set.
pub fn world_has_hdri(w: &WorldData) -> bool {
    w.hdri_path.is_some()
}

/// Ambient energy value.
pub fn world_ambient_energy(w: &WorldData) -> f32 {
    w.ambient_energy
}

/// Visibility distance (fog_end - fog_start).
pub fn world_fog_visibility(w: &WorldData) -> f32 {
    (w.fog_end - w.fog_start).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_world_data() {
        let w = new_world_data("world");
        assert_eq!(w.name, "world");
    }

    #[test]
    fn test_world_to_json() {
        let w = new_world_data("env");
        let j = world_to_json(&w);
        assert!(j.contains("env"));
    }

    #[test]
    fn test_world_has_hdri_false() {
        let w = new_world_data("w");
        assert!(!world_has_hdri(&w));
    }

    #[test]
    fn test_world_ambient_energy() {
        let w = new_world_data("w");
        assert!((world_ambient_energy(&w) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_world_fog_visibility() {
        let w = new_world_data("w");
        assert!(world_fog_visibility(&w) > 0.0);
    }
}
