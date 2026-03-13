// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Force field (wind/turbulence) mesh — force field emitter attached to mesh geometry.

/// Kind of force field effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceFieldKind {
    Wind,
    Turbulence,
    Vortex,
    Drag,
}

/// Force field emitter descriptor.
#[derive(Debug, Clone)]
pub struct ForceFieldMesh {
    pub kind: ForceFieldKind,
    pub strength: f32,
    pub falloff_power: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub direction: [f32; 3],
    pub label: String,
}

/// Create a wind force field.
pub fn new_wind_field(strength: f32, direction: [f32; 3], label: &str) -> ForceFieldMesh {
    ForceFieldMesh {
        kind: ForceFieldKind::Wind,
        strength,
        falloff_power: 2.0,
        min_distance: 0.0,
        max_distance: 10.0,
        direction,
        label: label.to_owned(),
    }
}

/// Create a turbulence force field.
pub fn new_turbulence_field(strength: f32, label: &str) -> ForceFieldMesh {
    ForceFieldMesh {
        kind: ForceFieldKind::Turbulence,
        strength,
        falloff_power: 2.0,
        min_distance: 0.0,
        max_distance: 10.0,
        direction: [0.0, 1.0, 0.0],
        label: label.to_owned(),
    }
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Compute the field strength at a given world-space position from the emitter origin.
pub fn field_strength_at(ff: &ForceFieldMesh, emitter_pos: [f32; 3], query_pos: [f32; 3]) -> f32 {
    let d = dist3(emitter_pos, query_pos).max(ff.min_distance);
    if d > ff.max_distance || ff.max_distance < 1e-8 {
        return 0.0;
    }
    let falloff = (1.0 - d / ff.max_distance).powf(ff.falloff_power);
    ff.strength * falloff
}

/// Return the kind name string.
pub fn field_kind_name(ff: &ForceFieldMesh) -> &'static str {
    match ff.kind {
        ForceFieldKind::Wind => "wind",
        ForceFieldKind::Turbulence => "turbulence",
        ForceFieldKind::Vortex => "vortex",
        ForceFieldKind::Drag => "drag",
    }
}

/// Serialize to JSON-style string.
pub fn force_field_to_json(ff: &ForceFieldMesh) -> String {
    format!(
        r#"{{"label":"{}", "kind":"{}", "strength":{:.4}, "max_dist":{:.4}}}"#,
        ff.label,
        field_kind_name(ff),
        ff.strength,
        ff.max_distance
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wind_field_kind_is_wind() {
        /* wind field should have Wind kind */
        let f = new_wind_field(1.0, [0.0, 1.0, 0.0], "w");
        assert_eq!(f.kind, ForceFieldKind::Wind);
    }

    #[test]
    fn turbulence_field_kind_is_turbulence() {
        /* turbulence field should have Turbulence kind */
        let f = new_turbulence_field(1.0, "t");
        assert_eq!(f.kind, ForceFieldKind::Turbulence);
    }

    #[test]
    fn field_strength_at_emitter_equals_strength() {
        /* at distance 0 (min_dist=0) the result equals the field strength */
        let f = new_wind_field(5.0, [0.0, 1.0, 0.0], "w");
        let s = field_strength_at(&f, [0.0; 3], [0.0; 3]);
        assert!((s - 5.0).abs() < 1e-4);
    }

    #[test]
    fn field_strength_beyond_max_dist_is_zero() {
        /* beyond max_distance field strength is zero */
        let f = new_wind_field(5.0, [0.0, 1.0, 0.0], "w");
        let s = field_strength_at(&f, [0.0; 3], [100.0, 0.0, 0.0]);
        assert_eq!(s, 0.0);
    }

    #[test]
    fn field_kind_name_wind() {
        /* wind kind name is "wind" */
        let f = new_wind_field(1.0, [0.0; 3], "w");
        assert_eq!(field_kind_name(&f), "wind");
    }

    #[test]
    fn field_kind_name_turbulence() {
        /* turbulence kind name is "turbulence" */
        let f = new_turbulence_field(1.0, "t");
        assert_eq!(field_kind_name(&f), "turbulence");
    }

    #[test]
    fn json_contains_label() {
        /* JSON includes label */
        let f = new_wind_field(1.0, [0.0; 3], "myWind");
        assert!(force_field_to_json(&f).contains("myWind"));
    }

    #[test]
    fn field_strength_decreases_with_distance() {
        /* strength at distance 5 should be less than at distance 1 */
        let f = new_wind_field(10.0, [0.0, 1.0, 0.0], "w");
        let s1 = field_strength_at(&f, [0.0; 3], [1.0, 0.0, 0.0]);
        let s5 = field_strength_at(&f, [0.0; 3], [5.0, 0.0, 0.0]);
        assert!(s1 > s5);
    }

    #[test]
    fn default_falloff_power_is_two() {
        /* default falloff power should be 2 */
        let f = new_wind_field(1.0, [0.0; 3], "w");
        assert!((f.falloff_power - 2.0).abs() < 1e-5);
    }
}
