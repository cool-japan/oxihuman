// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Magnetic attraction point modifier — draws nearby vertices toward a 3-D point.

/// A magnetic attraction point in world-space.
#[derive(Debug, Clone)]
pub struct MagnetPoint {
    pub position: [f32; 3],
    pub strength: f32,
    pub radius: f32,
    pub label: String,
}

/// Collection of magnet points.
#[derive(Debug, Default)]
pub struct MagnetSet {
    magnets: Vec<MagnetPoint>,
}

/// Create a new, empty magnet set.
pub fn new_magnet_set() -> MagnetSet {
    MagnetSet::default()
}

/// Add a magnet at the given world-space position.
pub fn add_magnet(
    set: &mut MagnetSet,
    position: [f32; 3],
    strength: f32,
    radius: f32,
    label: &str,
) {
    set.magnets.push(MagnetPoint {
        position,
        strength: strength.clamp(0.0, 1.0),
        radius: radius.max(0.0),
        label: label.to_owned(),
    });
}

/// Number of magnets in the set.
pub fn magnet_count(set: &MagnetSet) -> usize {
    set.magnets.len()
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Linear falloff weight of a magnet at `position` for a vertex at `vertex_pos`.
pub fn magnet_weight(magnet: &MagnetPoint, vertex_pos: [f32; 3]) -> f32 {
    let d = dist3(magnet.position, vertex_pos);
    if magnet.radius < 1e-8 {
        return 0.0;
    }
    let t = (1.0 - d / magnet.radius).clamp(0.0, 1.0);
    t * magnet.strength
}

/// Apply all magnets to a vertex buffer (in-place, additive blend).
pub fn apply_magnets(set: &MagnetSet, positions: &mut [[f32; 3]]) {
    for pos in positions.iter_mut() {
        for m in &set.magnets {
            let w = magnet_weight(m, *pos);
            if w > 0.0 {
                pos[0] += (m.position[0] - pos[0]) * w;
                pos[1] += (m.position[1] - pos[1]) * w;
                pos[2] += (m.position[2] - pos[2]) * w;
            }
        }
    }
}

/// Average radius over all magnets.
pub fn average_magnet_radius(set: &MagnetSet) -> f32 {
    if set.magnets.is_empty() {
        return 0.0;
    }
    let sum: f32 = set.magnets.iter().map(|m| m.radius).sum();
    sum / set.magnets.len() as f32
}

/// Serialize magnet set to JSON-style string.
pub fn magnet_set_to_json(set: &MagnetSet) -> String {
    let entries: Vec<String> = set
        .magnets
        .iter()
        .map(|m| {
            format!(
                r#"{{"label":"{}", "pos":[{:.3},{:.3},{:.3}], "strength":{:.4}, "radius":{:.4}}}"#,
                m.label, m.position[0], m.position[1], m.position[2], m.strength, m.radius
            )
        })
        .collect();
    format!("[{}]", entries.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_magnet_set_count_is_zero() {
        /* fresh set has no magnets */
        let s = new_magnet_set();
        assert_eq!(magnet_count(&s), 0);
    }

    #[test]
    fn add_magnet_increments_count() {
        /* adding one magnet yields count 1 */
        let mut s = new_magnet_set();
        add_magnet(&mut s, [0.0, 0.0, 0.0], 1.0, 2.0, "center");
        assert_eq!(magnet_count(&s), 1);
    }

    #[test]
    fn strength_is_clamped_to_one() {
        /* strength 3 should be stored as 1 */
        let mut s = new_magnet_set();
        add_magnet(&mut s, [0.0, 0.0, 0.0], 3.0, 1.0, "strong");
        assert!((s.magnets[0].strength - 1.0).abs() < 1e-6);
    }

    #[test]
    fn magnet_weight_at_center_equals_strength() {
        /* weight at the magnet position itself should equal its strength */
        let m = MagnetPoint {
            position: [0.0; 3],
            strength: 0.8,
            radius: 1.0,
            label: "t".into(),
        };
        let w = magnet_weight(&m, [0.0, 0.0, 0.0]);
        assert!((w - 0.8).abs() < 1e-5);
    }

    #[test]
    fn magnet_weight_outside_radius_is_zero() {
        /* vertex beyond the radius should have zero weight */
        let m = MagnetPoint {
            position: [0.0; 3],
            strength: 1.0,
            radius: 1.0,
            label: "t".into(),
        };
        let w = magnet_weight(&m, [5.0, 0.0, 0.0]);
        assert_eq!(w, 0.0);
    }

    #[test]
    fn apply_magnets_moves_vertex_toward_magnet() {
        /* vertex should move closer to the magnet position */
        let mut s = new_magnet_set();
        add_magnet(&mut s, [10.0, 0.0, 0.0], 1.0, 20.0, "pull");
        let mut pos = [[0.0_f32, 0.0, 0.0]];
        apply_magnets(&s, &mut pos);
        assert!(pos[0][0] > 0.0);
    }

    #[test]
    fn average_radius_empty_is_zero() {
        /* empty set average radius is zero */
        let s = new_magnet_set();
        assert_eq!(average_magnet_radius(&s), 0.0);
    }

    #[test]
    fn average_radius_correct() {
        /* average of radii 2 and 4 should be 3 */
        let mut s = new_magnet_set();
        add_magnet(&mut s, [0.0; 3], 1.0, 2.0, "a");
        add_magnet(&mut s, [0.0; 3], 1.0, 4.0, "b");
        assert!((average_magnet_radius(&s) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn json_contains_label_key() {
        /* serialised output should contain "label" */
        let mut s = new_magnet_set();
        add_magnet(&mut s, [1.0, 2.0, 3.0], 0.5, 1.0, "test");
        let j = magnet_set_to_json(&s);
        assert!(j.contains("label"));
    }

    #[test]
    fn json_empty_is_empty_array() {
        /* empty set gives [] */
        let s = new_magnet_set();
        assert_eq!(magnet_set_to_json(&s), "[]");
    }
}
