// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cloth simulation pin constraint export.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothPin {
    pub vertex_index: u32,
    pub strength: f32,
    pub world_position: Option<[f32; 3]>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothPinExport {
    pub pins: Vec<ClothPin>,
}

#[allow(dead_code)]
pub fn new_cloth_pin_export() -> ClothPinExport {
    ClothPinExport { pins: Vec::new() }
}

#[allow(dead_code)]
pub fn add_pin(exp: &mut ClothPinExport, vertex: u32, strength: f32) {
    exp.pins.push(ClothPin {
        vertex_index: vertex,
        strength: strength.clamp(0.0, 1.0),
        world_position: None,
    });
}

#[allow(dead_code)]
pub fn add_pin_at(exp: &mut ClothPinExport, vertex: u32, strength: f32, pos: [f32; 3]) {
    exp.pins.push(ClothPin {
        vertex_index: vertex,
        strength: strength.clamp(0.0, 1.0),
        world_position: Some(pos),
    });
}

#[allow(dead_code)]
pub fn pin_count(exp: &ClothPinExport) -> usize {
    exp.pins.len()
}

#[allow(dead_code)]
pub fn is_pinned(exp: &ClothPinExport, vertex: u32) -> bool {
    exp.pins.iter().any(|p| p.vertex_index == vertex)
}

#[allow(dead_code)]
pub fn remove_pin(exp: &mut ClothPinExport, vertex: u32) {
    exp.pins.retain(|p| p.vertex_index != vertex);
}

#[allow(dead_code)]
pub fn pins_with_position(exp: &ClothPinExport) -> usize {
    exp.pins
        .iter()
        .filter(|p| p.world_position.is_some())
        .count()
}

#[allow(dead_code)]
pub fn avg_pin_strength(exp: &ClothPinExport) -> f32 {
    if exp.pins.is_empty() {
        return 0.0;
    }
    exp.pins.iter().map(|p| p.strength).sum::<f32>() / exp.pins.len() as f32
}

#[allow(dead_code)]
pub fn cloth_pin_to_json(exp: &ClothPinExport) -> String {
    format!(
        "{{\"pin_count\":{},\"avg_strength\":{}}}",
        pin_count(exp),
        avg_pin_strength(exp)
    )
}

#[allow(dead_code)]
pub fn validate_pins(exp: &ClothPinExport) -> bool {
    exp.pins.iter().all(|p| (0.0..=1.0).contains(&p.strength))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let exp = new_cloth_pin_export();
        assert_eq!(pin_count(&exp), 0);
    }

    #[test]
    fn test_add_pin() {
        let mut exp = new_cloth_pin_export();
        add_pin(&mut exp, 5, 1.0);
        assert_eq!(pin_count(&exp), 1);
    }

    #[test]
    fn test_is_pinned() {
        let mut exp = new_cloth_pin_export();
        add_pin(&mut exp, 3, 0.8);
        assert!(is_pinned(&exp, 3));
        assert!(!is_pinned(&exp, 99));
    }

    #[test]
    fn test_remove_pin() {
        let mut exp = new_cloth_pin_export();
        add_pin(&mut exp, 7, 1.0);
        remove_pin(&mut exp, 7);
        assert_eq!(pin_count(&exp), 0);
    }

    #[test]
    fn test_add_pin_at() {
        let mut exp = new_cloth_pin_export();
        add_pin_at(&mut exp, 2, 0.5, [1.0, 2.0, 3.0]);
        assert_eq!(pins_with_position(&exp), 1);
    }

    #[test]
    fn test_avg_strength() {
        let mut exp = new_cloth_pin_export();
        add_pin(&mut exp, 0, 1.0);
        add_pin(&mut exp, 1, 0.0);
        assert!((avg_pin_strength(&exp) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_json_output() {
        let exp = new_cloth_pin_export();
        let j = cloth_pin_to_json(&exp);
        assert!(j.contains("pin_count"));
    }

    #[test]
    fn test_validate_pins() {
        let mut exp = new_cloth_pin_export();
        add_pin(&mut exp, 0, 0.5);
        assert!(validate_pins(&exp));
    }

    #[test]
    fn test_clamp_on_add() {
        let mut exp = new_cloth_pin_export();
        add_pin(&mut exp, 0, 2.0);
        assert!((exp.pins[0].strength - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_avg_empty() {
        let exp = new_cloth_pin_export();
        assert!((avg_pin_strength(&exp)).abs() < 1e-6);
    }
}
