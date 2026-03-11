// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Beard and facial hair density morph parameters.

/// Facial hair zone.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BeardZone {
    Mustache,
    ChinBeard,
    Cheeks,
    Sideburns,
    Neck,
}

impl BeardZone {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            BeardZone::Mustache => "mustache",
            BeardZone::ChinBeard => "chin_beard",
            BeardZone::Cheeks => "cheeks",
            BeardZone::Sideburns => "sideburns",
            BeardZone::Neck => "neck",
        }
    }
}

/// Per-zone beard density entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BeardZoneEntry {
    pub zone: BeardZone,
    pub density: f32,
    pub length: f32,
    pub thickness: f32,
}

impl BeardZoneEntry {
    #[allow(dead_code)]
    pub fn new(zone: BeardZone) -> Self {
        BeardZoneEntry {
            zone,
            density: 0.0,
            length: 0.0,
            thickness: 0.0,
        }
    }
}

/// Beard density state across all zones.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BeardDensityState {
    pub zones: Vec<BeardZoneEntry>,
    pub global_density: f32,
}

#[allow(dead_code)]
pub fn default_beard_density_state() -> BeardDensityState {
    let zones = vec![
        BeardZoneEntry::new(BeardZone::Mustache),
        BeardZoneEntry::new(BeardZone::ChinBeard),
        BeardZoneEntry::new(BeardZone::Cheeks),
        BeardZoneEntry::new(BeardZone::Sideburns),
        BeardZoneEntry::new(BeardZone::Neck),
    ];
    BeardDensityState {
        zones,
        global_density: 0.0,
    }
}

#[allow(dead_code)]
pub fn bd_set_global(state: &mut BeardDensityState, v: f32) {
    state.global_density = v.clamp(0.0, 1.0);
    for z in &mut state.zones {
        z.density = v.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn bd_set_zone_density(state: &mut BeardDensityState, zone: BeardZone, v: f32) {
    for z in &mut state.zones {
        if z.zone == zone {
            z.density = v.clamp(0.0, 1.0);
            return;
        }
    }
}

#[allow(dead_code)]
pub fn bd_set_zone_length(state: &mut BeardDensityState, zone: BeardZone, v: f32) {
    for z in &mut state.zones {
        if z.zone == zone {
            z.length = v.clamp(0.0, 1.0);
            return;
        }
    }
}

#[allow(dead_code)]
pub fn bd_reset(state: &mut BeardDensityState) {
    for z in &mut state.zones {
        z.density = 0.0;
        z.length = 0.0;
        z.thickness = 0.0;
    }
    state.global_density = 0.0;
}

#[allow(dead_code)]
pub fn bd_is_clean_shaven(state: &BeardDensityState) -> bool {
    state.zones.iter().all(|z| z.density < 1e-6)
}

#[allow(dead_code)]
pub fn bd_average_density(state: &BeardDensityState) -> f32 {
    if state.zones.is_empty() {
        return 0.0;
    }
    let sum: f32 = state.zones.iter().map(|z| z.density).sum();
    sum / state.zones.len() as f32
}

#[allow(dead_code)]
pub fn bd_to_json(state: &BeardDensityState) -> String {
    format!(
        r#"{{"global_density":{:.4},"zone_count":{}}}"#,
        state.global_density,
        state.zones.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_clean_shaven() {
        assert!(bd_is_clean_shaven(&default_beard_density_state()));
    }

    #[test]
    fn set_global_density() {
        let mut s = default_beard_density_state();
        bd_set_global(&mut s, 0.8);
        assert!((s.global_density - 0.8).abs() < 1e-5);
    }

    #[test]
    fn set_global_sets_all_zones() {
        let mut s = default_beard_density_state();
        bd_set_global(&mut s, 0.5);
        assert!(!bd_is_clean_shaven(&s));
    }

    #[test]
    fn set_zone_density_clamps() {
        let mut s = default_beard_density_state();
        bd_set_zone_density(&mut s, BeardZone::Mustache, 2.0);
        let z = s
            .zones
            .iter()
            .find(|z| z.zone == BeardZone::Mustache)
            .unwrap();
        assert!((z.density - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears_all() {
        let mut s = default_beard_density_state();
        bd_set_global(&mut s, 1.0);
        bd_reset(&mut s);
        assert!(bd_is_clean_shaven(&s));
    }

    #[test]
    fn average_density_zero_default() {
        let s = default_beard_density_state();
        assert!(bd_average_density(&s).abs() < 1e-6);
    }

    #[test]
    fn average_density_after_global_set() {
        let mut s = default_beard_density_state();
        bd_set_global(&mut s, 0.6);
        assert!((bd_average_density(&s) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn zone_length_set() {
        let mut s = default_beard_density_state();
        bd_set_zone_length(&mut s, BeardZone::ChinBeard, 0.7);
        let z = s
            .zones
            .iter()
            .find(|z| z.zone == BeardZone::ChinBeard)
            .unwrap();
        assert!((z.length - 0.7).abs() < 1e-5);
    }

    #[test]
    fn to_json_has_global_density() {
        assert!(bd_to_json(&default_beard_density_state()).contains("global_density"));
    }
}
