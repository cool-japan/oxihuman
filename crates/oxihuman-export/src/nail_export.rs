// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct Nail {
    pub finger_id: u8,
    pub length_mm: f32,
    pub width_mm: f32,
    pub thickness_mm: f32,
    pub curvature_deg: f32,
    pub color_rgb: [f32; 3],
}

pub fn new_nail(finger_id: u8) -> Nail {
    Nail {
        finger_id,
        length_mm: 3.0,
        width_mm: 12.0,
        thickness_mm: 0.5,
        curvature_deg: 15.0,
        color_rgb: [0.95, 0.88, 0.82],
    }
}

pub fn nail_area_mm2(n: &Nail) -> f32 {
    n.length_mm * n.width_mm
}

pub fn nail_to_json(n: &Nail) -> String {
    format!(
        "{{\"finger_id\":{},\"length_mm\":{:.2},\"width_mm\":{:.2},\"thickness_mm\":{:.3},\"curvature_deg\":{:.2}}}",
        n.finger_id, n.length_mm, n.width_mm, n.thickness_mm, n.curvature_deg
    )
}

pub fn nails_to_json(nails: &[Nail]) -> String {
    let items: Vec<String> = nails.iter().map(nail_to_json).collect();
    format!("{{\"nails\":[{}]}}", items.join(","))
}

pub fn nail_is_long(n: &Nail) -> bool {
    n.length_mm > 5.0
}

pub fn nail_count(nails: &[Nail]) -> usize {
    nails.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_nail() {
        /* default fields */
        let n = new_nail(1);
        assert_eq!(n.finger_id, 1);
        assert!((n.length_mm - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_area() {
        /* length * width */
        let n = new_nail(1);
        assert!((nail_area_mm2(&n) - 36.0).abs() < 1e-4);
    }

    #[test]
    fn test_to_json() {
        /* JSON contains finger_id */
        let n = new_nail(3);
        let json = nail_to_json(&n);
        assert!(json.contains("\"finger_id\":3"));
    }

    #[test]
    fn test_is_long_false() {
        /* default length 3 is not long */
        let n = new_nail(1);
        assert!(!nail_is_long(&n));
    }

    #[test]
    fn test_count() {
        /* count 10 nails */
        let nails: Vec<Nail> = (1..=10).map(new_nail).collect();
        assert_eq!(nail_count(&nails), 10);
    }
}
