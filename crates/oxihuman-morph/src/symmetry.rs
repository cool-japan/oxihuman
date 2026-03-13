// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use crate::engine::MeshBuffers;
use oxihuman_core::parser::target::{Delta, TargetFile};
use std::collections::HashMap;

/// A mapping between left and right vertex pairs, plus center vertices.
pub struct SymmetryMap {
    /// (left_vid, right_vid) pairs — left is where x < 0, right where x > 0
    pub pairs: Vec<(u32, u32)>,
    /// Vertices on the center plane (|x| < tolerance)
    pub center_verts: Vec<u32>,
}

impl SymmetryMap {
    /// Build a symmetry map from mesh positions.
    /// For each vertex with x > tol, find the left vertex with x < -tol at the same (y, z).
    /// `tolerance`: max distance to consider a pair symmetric (default 0.001).
    pub fn from_positions(positions: &[[f32; 3]], tolerance: f32) -> Self {
        let tol = if tolerance <= 0.0 { 0.001 } else { tolerance };

        // Quantize a float value to an i64 key using the tolerance as grid size.
        let quantize = |v: f32| -> i64 { (v / tol).round() as i64 };

        // Build a map from quantized (y, z) → (vid, x) for left-side vertices (x < -tol).
        let mut left_map: HashMap<(i64, i64), (u32, f32)> = HashMap::new();
        let mut center_verts: Vec<u32> = Vec::new();

        for (i, pos) in positions.iter().enumerate() {
            let x = pos[0];
            let y = pos[1];
            let z = pos[2];
            if x < -tol {
                let key = (quantize(y), quantize(z));
                // Keep the left vertex whose x is closest to 0 among candidates
                left_map.entry(key).or_insert((i as u32, x));
            } else if x.abs() <= tol {
                center_verts.push(i as u32);
            }
        }

        let mut pairs: Vec<(u32, u32)> = Vec::new();

        for (i, pos) in positions.iter().enumerate() {
            let x = pos[0];
            let y = pos[1];
            let z = pos[2];
            if x > tol {
                // Mirror x to find expected left partner
                let mirrored_x = -x;
                let key = (quantize(y), quantize(z));
                if let Some(&(left_vid, left_x)) = left_map.get(&key) {
                    // Check actual distance between (mirrored_x, y, z) and (left_x, left_y, left_z)
                    let left_pos = positions[left_vid as usize];
                    let dist = ((mirrored_x - left_x).powi(2)
                        + (y - left_pos[1]).powi(2)
                        + (z - left_pos[2]).powi(2))
                    .sqrt();
                    if dist <= tol {
                        pairs.push((left_vid, i as u32));
                    }
                }
            }
        }

        SymmetryMap {
            pairs,
            center_verts,
        }
    }

    /// Number of symmetric pairs found.
    pub fn pair_count(&self) -> usize {
        self.pairs.len()
    }

    /// Number of center vertices (on the symmetry plane).
    pub fn center_count(&self) -> usize {
        self.center_verts.len()
    }
}

/// Mirror a TargetFile's deltas: for each delta, negate dx to flip across the X axis.
/// The vertex indices remain unchanged (mirroring delta direction, not vertex identity).
pub fn mirror_target_deltas(target: &TargetFile) -> TargetFile {
    TargetFile {
        name: format!("{}-mirrored", target.name),
        deltas: target
            .deltas
            .iter()
            .map(|d| Delta {
                vid: d.vid,
                dx: -d.dx,
                dy: d.dy,
                dz: d.dz,
            })
            .collect(),
    }
}

/// Symmetrize mesh positions: for each (left, right) pair in the map,
/// average the positions and assign both to the average (left gets avg with -x, right with +x).
/// Center vertices are unchanged.
pub fn symmetrize_positions(positions: &[[f32; 3]], map: &SymmetryMap) -> Vec<[f32; 3]> {
    let mut result = positions.to_vec();
    for &(left_vid, right_vid) in &map.pairs {
        let l = positions[left_vid as usize];
        let r = positions[right_vid as usize];
        let avg_yz = ((l[1] + r[1]) / 2.0, (l[2] + r[2]) / 2.0);
        let avg_x = (l[0].abs() + r[0].abs()) / 2.0;
        result[left_vid as usize] = [-avg_x, avg_yz.0, avg_yz.1];
        result[right_vid as usize] = [avg_x, avg_yz.0, avg_yz.1];
    }
    result
}

/// Check if a target name is a left-side variant (contains "l-" prefix, "-l-" substring,
/// "left", or "lside"). Returns the right-side name if it is.
pub fn mirror_target_name(name: &str) -> Option<String> {
    let lower = name.to_lowercase();

    // Check for "-l-" substring first (more specific than "l-" prefix)
    if let Some(pos) = lower.find("-l-") {
        let rest = &name[pos + 3..];
        let prefix = &name[..pos];
        return Some(format!("{}-r-{}", prefix, rest));
    }

    // Check for "l-" prefix
    if lower.starts_with("l-") {
        return Some(format!("r-{}", &name[2..]));
    }

    // Check for "lside"
    if lower.contains("lside") {
        let replaced = replace_case_insensitive(name, "lside", "rside");
        return Some(replaced);
    }

    // Check for "left"
    if lower.contains("left") {
        let replaced = replace_case_insensitive(name, "left", "right");
        return Some(replaced);
    }

    None
}

/// Replace the first occurrence of `from` (case-insensitive) with `to` in `s`.
fn replace_case_insensitive(s: &str, from: &str, to: &str) -> String {
    let lower = s.to_lowercase();
    if let Some(pos) = lower.find(from) {
        let mut result = String::with_capacity(s.len() - from.len() + to.len());
        result.push_str(&s[..pos]);
        result.push_str(to);
        result.push_str(&s[pos + from.len()..]);
        result
    } else {
        s.to_string()
    }
}

/// Check if a target name is a left-side variant.
pub fn is_left_side(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with("l-")
        || lower.contains("-l-")
        || lower.contains("left")
        || lower.contains("lside")
}

/// Check if a target name is a right-side variant.
pub fn is_right_side(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with("r-")
        || lower.contains("-r-")
        || lower.contains("right")
        || lower.contains("rside")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirror_target_deltas_negates_dx() {
        let target = TargetFile {
            name: "test".to_string(),
            deltas: vec![Delta {
                vid: 42,
                dx: 0.5,
                dy: 0.1,
                dz: -0.2,
            }],
        };
        let mirrored = mirror_target_deltas(&target);
        assert_eq!(mirrored.deltas.len(), 1);
        assert!(
            (mirrored.deltas[0].dx + 0.5).abs() < 1e-6,
            "dx should be -0.5"
        );
        assert!((mirrored.deltas[0].dy - 0.1).abs() < 1e-6, "dy unchanged");
        assert!((mirrored.deltas[0].dz + 0.2).abs() < 1e-6, "dz unchanged");
        assert_eq!(mirrored.deltas[0].vid, 42);
    }

    #[test]
    fn mirror_target_name_l_prefix() {
        let result = mirror_target_name("l-arm-muscle.target");
        assert_eq!(result, Some("r-arm-muscle.target".to_string()));
    }

    #[test]
    fn mirror_target_name_left_word() {
        let result = mirror_target_name("leftarm-size.target");
        assert_eq!(result, Some("rightarm-size.target".to_string()));
    }

    #[test]
    fn mirror_target_name_none_for_center() {
        let result = mirror_target_name("head-age.target");
        assert_eq!(result, None);
    }

    #[test]
    fn is_left_side_true() {
        assert!(is_left_side("l-forearm-size.target"));
    }

    #[test]
    fn is_right_side_true() {
        assert!(is_right_side("r-forearm-size.target"));
    }

    #[test]
    fn symmetry_map_from_simple_pair() {
        let positions: Vec<[f32; 3]> = vec![
            [-1.0, 0.0, 0.0], // left
            [1.0, 0.0, 0.0],  // right
            [0.0, 0.0, 0.0],  // center
        ];
        let map = SymmetryMap::from_positions(&positions, 0.001);
        assert_eq!(map.pair_count(), 1, "should find 1 pair");
        assert_eq!(map.center_count(), 1, "should find 1 center vertex");
        // The pair should be (0, 1) — left vid 0, right vid 1
        assert_eq!(map.pairs[0], (0, 1));
        assert_eq!(map.center_verts[0], 2);
    }

    #[test]
    fn symmetrize_positions_averages_pair() {
        let positions: Vec<[f32; 3]> = vec![
            [-1.2, 0.5, 0.3], // left (slightly off)
            [1.0, 0.5, 0.3],  // right
            [0.0, 0.0, 0.0],  // center
        ];
        let _map = SymmetryMap::from_positions(&positions, 0.001);
        // We need to manually construct the map since positions may not match exactly
        let manual_map = SymmetryMap {
            pairs: vec![(0, 1)],
            center_verts: vec![2],
        };
        let sym = symmetrize_positions(&positions, &manual_map);
        // avg_x = (1.2 + 1.0) / 2 = 1.1
        let avg_x = (1.2_f32 + 1.0_f32) / 2.0;
        assert!(
            (sym[0][0] + avg_x).abs() < 1e-5,
            "left x should be -{}",
            avg_x
        );
        assert!(
            (sym[1][0] - avg_x).abs() < 1e-5,
            "right x should be +{}",
            avg_x
        );
        // y and z should be averaged
        assert!((sym[0][1] - 0.5).abs() < 1e-5);
        assert!((sym[1][1] - 0.5).abs() < 1e-5);
        // center unchanged
        assert!((sym[2][0] - 0.0).abs() < 1e-5);
    }
}
