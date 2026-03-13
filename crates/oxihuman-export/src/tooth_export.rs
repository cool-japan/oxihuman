// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct Tooth {
    pub id: u8,
    pub pos: [f32; 3],
    pub orientation: [f32; 4],
    pub crown_height_mm: f32,
    pub root_length_mm: f32,
    pub shade: String,
}

pub fn new_tooth(id: u8, pos: [f32; 3]) -> Tooth {
    Tooth {
        id,
        pos,
        orientation: [0.0, 0.0, 0.0, 1.0],
        crown_height_mm: 8.5,
        root_length_mm: 13.0,
        shade: String::from("A2"),
    }
}

pub fn tooth_total_length(t: &Tooth) -> f32 {
    t.crown_height_mm + t.root_length_mm
}

pub fn tooth_to_json(t: &Tooth) -> String {
    format!(
        "{{\"id\":{},\"pos\":[{:.4},{:.4},{:.4}],\"crown_mm\":{:.2},\"root_mm\":{:.2},\"shade\":\"{}\"}}",
        t.id,
        t.pos[0],
        t.pos[1],
        t.pos[2],
        t.crown_height_mm,
        t.root_length_mm,
        t.shade
    )
}

pub fn teeth_to_json(teeth: &[Tooth]) -> String {
    let items: Vec<String> = teeth.iter().map(tooth_to_json).collect();
    format!("{{\"teeth\":[{}]}}", items.join(","))
}

/// Molar: id 6, 7, 8, 14, 15, 16 per arch (FDI or UNS 1-based).
pub fn tooth_is_molar(t: &Tooth) -> bool {
    matches!(t.id, 6 | 7 | 8 | 14 | 15 | 16)
}

pub fn tooth_count(teeth: &[Tooth]) -> usize {
    teeth.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tooth() {
        /* construction */
        let t = new_tooth(1, [0.0, 0.0, 0.0]);
        assert_eq!(t.id, 1);
        assert!((t.crown_height_mm - 8.5).abs() < 1e-6);
    }

    #[test]
    fn test_total_length() {
        /* crown + root */
        let t = new_tooth(1, [0.0; 3]);
        assert!((tooth_total_length(&t) - 21.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        /* JSON contains id */
        let t = new_tooth(5, [1.0, 2.0, 3.0]);
        let json = tooth_to_json(&t);
        assert!(json.contains("\"id\":5"));
    }

    #[test]
    fn test_teeth_to_json() {
        /* array in JSON */
        let teeth = vec![new_tooth(1, [0.0; 3]), new_tooth(2, [1.0; 3])];
        let json = teeth_to_json(&teeth);
        assert!(json.contains("teeth"));
    }

    #[test]
    fn test_is_molar() {
        /* id 6 is molar, id 1 is not */
        let molar = new_tooth(6, [0.0; 3]);
        let incisor = new_tooth(1, [0.0; 3]);
        assert!(tooth_is_molar(&molar));
        assert!(!tooth_is_molar(&incisor));
    }

    #[test]
    fn test_count() {
        /* count */
        let teeth: Vec<Tooth> = (1..=4).map(|i| new_tooth(i, [0.0; 3])).collect();
        assert_eq!(tooth_count(&teeth), 4);
    }
}
