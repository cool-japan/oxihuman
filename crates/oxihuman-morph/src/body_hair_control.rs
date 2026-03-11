// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Body hair density and length parameters.

/// Body region for hair distribution.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BodyHairRegion {
    Arms,
    Legs,
    Chest,
    Abdomen,
    Back,
    Shoulders,
}

impl BodyHairRegion {
    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            BodyHairRegion::Arms => "arms",
            BodyHairRegion::Legs => "legs",
            BodyHairRegion::Chest => "chest",
            BodyHairRegion::Abdomen => "abdomen",
            BodyHairRegion::Back => "back",
            BodyHairRegion::Shoulders => "shoulders",
        }
    }
}

/// Per-region body hair entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyHairEntry {
    pub region: BodyHairRegion,
    pub density: f32,
    pub length: f32,
    pub color_darkness: f32,
}

impl BodyHairEntry {
    #[allow(dead_code)]
    pub fn new(region: BodyHairRegion) -> Self {
        BodyHairEntry {
            region,
            density: 0.0,
            length: 0.0,
            color_darkness: 0.5,
        }
    }
}

/// Overall body hair state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyHairState {
    pub entries: Vec<BodyHairEntry>,
    pub global_density: f32,
}

#[allow(dead_code)]
pub fn default_body_hair_state() -> BodyHairState {
    let entries = vec![
        BodyHairEntry::new(BodyHairRegion::Arms),
        BodyHairEntry::new(BodyHairRegion::Legs),
        BodyHairEntry::new(BodyHairRegion::Chest),
        BodyHairEntry::new(BodyHairRegion::Abdomen),
        BodyHairEntry::new(BodyHairRegion::Back),
        BodyHairEntry::new(BodyHairRegion::Shoulders),
    ];
    BodyHairState {
        entries,
        global_density: 0.0,
    }
}

#[allow(dead_code)]
pub fn bh_set_global(state: &mut BodyHairState, v: f32) {
    let v = v.clamp(0.0, 1.0);
    state.global_density = v;
    for e in &mut state.entries {
        e.density = v;
    }
}

#[allow(dead_code)]
pub fn bh_set_region_density(state: &mut BodyHairState, region: BodyHairRegion, v: f32) {
    for e in &mut state.entries {
        if e.region == region {
            e.density = v.clamp(0.0, 1.0);
            return;
        }
    }
}

#[allow(dead_code)]
pub fn bh_set_region_length(state: &mut BodyHairState, region: BodyHairRegion, v: f32) {
    for e in &mut state.entries {
        if e.region == region {
            e.length = v.clamp(0.0, 1.0);
            return;
        }
    }
}

#[allow(dead_code)]
pub fn bh_reset(state: &mut BodyHairState) {
    for e in &mut state.entries {
        e.density = 0.0;
        e.length = 0.0;
        e.color_darkness = 0.5;
    }
    state.global_density = 0.0;
}

#[allow(dead_code)]
pub fn bh_is_smooth(state: &BodyHairState) -> bool {
    state.entries.iter().all(|e| e.density < 1e-6)
}

#[allow(dead_code)]
pub fn bh_average_density(state: &BodyHairState) -> f32 {
    if state.entries.is_empty() {
        return 0.0;
    }
    let sum: f32 = state.entries.iter().map(|e| e.density).sum();
    sum / state.entries.len() as f32
}

#[allow(dead_code)]
pub fn bh_to_json(state: &BodyHairState) -> String {
    format!(
        r#"{{"global_density":{:.4},"region_count":{}}}"#,
        state.global_density,
        state.entries.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_smooth() {
        assert!(bh_is_smooth(&default_body_hair_state()));
    }

    #[test]
    fn set_global_changes_all() {
        let mut s = default_body_hair_state();
        bh_set_global(&mut s, 0.5);
        assert!(!bh_is_smooth(&s));
    }

    #[test]
    fn set_region_density_clamps() {
        let mut s = default_body_hair_state();
        bh_set_region_density(&mut s, BodyHairRegion::Chest, 5.0);
        let e = s
            .entries
            .iter()
            .find(|e| e.region == BodyHairRegion::Chest)
            .unwrap();
        assert!((e.density - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_region_length() {
        let mut s = default_body_hair_state();
        bh_set_region_length(&mut s, BodyHairRegion::Arms, 0.6);
        let e = s
            .entries
            .iter()
            .find(|e| e.region == BodyHairRegion::Arms)
            .unwrap();
        assert!((e.length - 0.6).abs() < 1e-5);
    }

    #[test]
    fn reset_clears() {
        let mut s = default_body_hair_state();
        bh_set_global(&mut s, 1.0);
        bh_reset(&mut s);
        assert!(bh_is_smooth(&s));
    }

    #[test]
    fn average_density_zero_by_default() {
        assert!(bh_average_density(&default_body_hair_state()).abs() < 1e-6);
    }

    #[test]
    fn average_density_after_global() {
        let mut s = default_body_hair_state();
        bh_set_global(&mut s, 0.4);
        assert!((bh_average_density(&s) - 0.4).abs() < 1e-5);
    }

    #[test]
    fn region_names_valid() {
        assert_eq!(BodyHairRegion::Arms.name(), "arms");
        assert_eq!(BodyHairRegion::Back.name(), "back");
    }

    #[test]
    fn to_json_has_global_density() {
        assert!(bh_to_json(&default_body_hair_state()).contains("global_density"));
    }
}
