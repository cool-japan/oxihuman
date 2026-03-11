// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Light volume v2 — deferred volumetric light sphere/cone representation.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightVolumeKind {
    Sphere,
    Cone,
    Box,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightVolumeV2 {
    pub id: u32,
    pub kind: LightVolumeKind,
    pub radius: f32,
    pub intensity: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct LightVolumeSetV2 {
    pub volumes: Vec<LightVolumeV2>,
}

#[allow(dead_code)]
pub fn new_light_volume_set_v2() -> LightVolumeSetV2 {
    LightVolumeSetV2::default()
}

#[allow(dead_code)]
pub fn lv2_add(
    set: &mut LightVolumeSetV2,
    id: u32,
    kind: LightVolumeKind,
    radius: f32,
    intensity: f32,
) {
    set.volumes.push(LightVolumeV2 {
        id,
        kind,
        radius: radius.max(0.0),
        intensity: intensity.max(0.0),
        enabled: true,
    });
}

#[allow(dead_code)]
pub fn lv2_remove(set: &mut LightVolumeSetV2, id: u32) {
    set.volumes.retain(|v| v.id != id);
}

#[allow(dead_code)]
pub fn lv2_clear(set: &mut LightVolumeSetV2) {
    set.volumes.clear();
}

#[allow(dead_code)]
pub fn lv2_count(set: &LightVolumeSetV2) -> usize {
    set.volumes.len()
}

#[allow(dead_code)]
pub fn lv2_enabled_count(set: &LightVolumeSetV2) -> usize {
    set.volumes.iter().filter(|v| v.enabled).count()
}

#[allow(dead_code)]
pub fn lv2_set_enabled(set: &mut LightVolumeSetV2, id: u32, v: bool) {
    if let Some(e) = set.volumes.iter_mut().find(|e| e.id == id) {
        e.enabled = v;
    }
}

#[allow(dead_code)]
pub fn lv2_total_intensity(set: &LightVolumeSetV2) -> f32 {
    set.volumes
        .iter()
        .filter(|v| v.enabled)
        .map(|v| v.intensity)
        .sum()
}

#[allow(dead_code)]
pub fn lv2_sphere_volume(radius: f32) -> f32 {
    (4.0 / 3.0) * PI * radius * radius * radius
}

#[allow(dead_code)]
pub fn lv2_solid_angle_rad(set: &LightVolumeSetV2) -> f32 {
    let t = lv2_total_intensity(set);
    if t > 0.0 {
        (1.0 / t).atan().min(PI * 0.25)
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn lv2_to_json(set: &LightVolumeSetV2) -> String {
    format!(
        "{{\"count\":{},\"enabled\":{}}}",
        lv2_count(set),
        lv2_enabled_count(set)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_is_empty() {
        assert_eq!(lv2_count(&new_light_volume_set_v2()), 0);
    }
    #[test]
    fn add_increments_count() {
        let mut s = new_light_volume_set_v2();
        lv2_add(&mut s, 0, LightVolumeKind::Sphere, 2.0, 1.0);
        assert_eq!(lv2_count(&s), 1);
    }
    #[test]
    fn clear_empties() {
        let mut s = new_light_volume_set_v2();
        lv2_add(&mut s, 0, LightVolumeKind::Sphere, 1.0, 1.0);
        lv2_clear(&mut s);
        assert_eq!(lv2_count(&s), 0);
    }
    #[test]
    fn remove_by_id() {
        let mut s = new_light_volume_set_v2();
        lv2_add(&mut s, 5, LightVolumeKind::Cone, 1.0, 1.0);
        lv2_remove(&mut s, 5);
        assert_eq!(lv2_count(&s), 0);
    }
    #[test]
    fn enabled_count_after_disable() {
        let mut s = new_light_volume_set_v2();
        lv2_add(&mut s, 0, LightVolumeKind::Sphere, 1.0, 1.0);
        lv2_set_enabled(&mut s, 0, false);
        assert_eq!(lv2_enabled_count(&s), 0);
    }
    #[test]
    fn total_intensity_sums() {
        let mut s = new_light_volume_set_v2();
        lv2_add(&mut s, 0, LightVolumeKind::Sphere, 1.0, 2.0);
        lv2_add(&mut s, 1, LightVolumeKind::Sphere, 1.0, 3.0);
        assert!((lv2_total_intensity(&s) - 5.0).abs() < 1e-5);
    }
    #[test]
    fn sphere_volume_positive() {
        assert!(lv2_sphere_volume(1.0) > 0.0);
    }
    #[test]
    fn sphere_volume_doubles_radius_eight_times() {
        assert!((lv2_sphere_volume(2.0) / lv2_sphere_volume(1.0) - 8.0).abs() < 1e-4);
    }
    #[test]
    fn solid_angle_nonneg() {
        assert!(lv2_solid_angle_rad(&new_light_volume_set_v2()) >= 0.0);
    }
    #[test]
    fn to_json_has_enabled() {
        assert!(lv2_to_json(&new_light_volume_set_v2()).contains("\"enabled\""));
    }
}
