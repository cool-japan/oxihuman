// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export grip/grasp pose data for hands and interactive objects.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum GripType { Power, Precision, Pinch, Hook, Custom }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GripExport {
    pub name: String,
    pub grip_type: GripType,
    pub hand: String,
    pub finger_curls: [f32; 5],
    pub contact_points: Vec<[f32; 3]>,
    pub strength: f32,
}

#[allow(dead_code)]
pub fn new_grip_export(name: &str, grip_type: GripType, hand: &str) -> GripExport {
    GripExport { name: name.to_string(), grip_type, hand: hand.to_string(), finger_curls: [0.0; 5], contact_points: Vec::new(), strength: 1.0 }
}

#[allow(dead_code)]
pub fn grip_set_curls(g: &mut GripExport, curls: [f32; 5]) {
    for (i, &c) in curls.iter().enumerate() { g.finger_curls[i] = c.clamp(0.0, 1.0); }
}

#[allow(dead_code)]
pub fn grip_add_contact(g: &mut GripExport, point: [f32; 3]) {
    g.contact_points.push(point);
}

#[allow(dead_code)]
pub fn grip_set_strength(g: &mut GripExport, s: f32) { g.strength = s.clamp(0.0, 1.0); }

#[allow(dead_code)]
pub fn grip_avg_curl(g: &GripExport) -> f32 {
    g.finger_curls.iter().sum::<f32>() / 5.0
}

#[allow(dead_code)]
pub fn grip_contact_count(g: &GripExport) -> usize { g.contact_points.len() }

#[allow(dead_code)]
pub fn grip_validate(g: &GripExport) -> bool {
    !g.name.is_empty() && (0.0..=1.0).contains(&g.strength) && g.finger_curls.iter().all(|&c| (0.0..=1.0).contains(&c))
}

#[allow(dead_code)]
pub fn grip_to_json(g: &GripExport) -> String {
    format!("{{\"name\":\"{}\",\"hand\":\"{}\",\"strength\":{:.2},\"contacts\":{}}}", g.name, g.hand, g.strength, g.contact_points.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> GripExport {
        let mut g = new_grip_export("hammer", GripType::Power, "right");
        grip_set_curls(&mut g, [0.8, 0.9, 0.9, 0.7, 0.3]);
        grip_add_contact(&mut g, [0.0, 0.0, 0.0]);
        g
    }

    #[test] fn test_new() { let g = new_grip_export("test", GripType::Precision, "left"); assert_eq!(g.name, "test"); }
    #[test] fn test_curls() { let g = sample(); assert!(g.finger_curls[0] > 0.0); }
    #[test] fn test_avg_curl() { let g = sample(); assert!(grip_avg_curl(&g) > 0.0); }
    #[test] fn test_contact_count() { assert_eq!(grip_contact_count(&sample()), 1); }
    #[test] fn test_strength() { let mut g = sample(); grip_set_strength(&mut g, 0.5); assert!((g.strength - 0.5).abs() < 1e-5); }
    #[test] fn test_validate() { assert!(grip_validate(&sample())); }
    #[test] fn test_to_json() { assert!(grip_to_json(&sample()).contains("hammer")); }
    #[test] fn test_grip_type() { let g = sample(); assert_eq!(g.grip_type, GripType::Power); }
    #[test] fn test_clamp_curl() { let mut g = sample(); grip_set_curls(&mut g, [2.0,2.0,2.0,2.0,2.0]); assert!(g.finger_curls.iter().all(|&c| (c - 1.0).abs() < 1e-6)); }
    #[test] fn test_clamp_strength() { let mut g = sample(); grip_set_strength(&mut g, 5.0); assert!((g.strength - 1.0).abs() < 1e-6); }
}
