// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Contact map: boolean grid of cells in contact.
pub struct ContactMap {
    pub cells: Vec<bool>,
    pub width: usize,
    pub height: usize,
    pub cell_size_m: f32,
}

pub fn new_contact_map(w: usize, h: usize, cell_size: f32) -> ContactMap {
    ContactMap {
        cells: vec![false; w * h],
        width: w,
        height: h,
        cell_size_m: cell_size.max(0.0),
    }
}

pub fn contact_set(m: &mut ContactMap, x: usize, y: usize, in_contact: bool) {
    if x < m.width && y < m.height {
        m.cells[y * m.width + x] = in_contact;
    }
}

pub fn contact_get(m: &ContactMap, x: usize, y: usize) -> bool {
    if x < m.width && y < m.height {
        m.cells[y * m.width + x]
    } else {
        false
    }
}

pub fn contact_area(m: &ContactMap) -> f32 {
    let n = m.cells.iter().filter(|&&b| b).count();
    n as f32 * m.cell_size_m * m.cell_size_m
}

pub fn contact_count(m: &ContactMap) -> usize {
    m.cells.iter().filter(|&&b| b).count()
}

pub fn contact_to_bytes(m: &ContactMap) -> Vec<u8> {
    m.cells.iter().map(|&b| if b { 1u8 } else { 0u8 }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_contact_map_size() {
        let m = new_contact_map(5, 5, 0.01);
        assert_eq!(m.cells.len(), 25);
    }

    #[test]
    fn test_contact_set_get() {
        let mut m = new_contact_map(4, 4, 0.01);
        contact_set(&mut m, 2, 3, true);
        assert!(contact_get(&m, 2, 3));
        assert!(!contact_get(&m, 0, 0));
    }

    #[test]
    fn test_contact_get_oob() {
        let m = new_contact_map(4, 4, 0.01);
        assert!(!contact_get(&m, 10, 10));
    }

    #[test]
    fn test_contact_area() {
        let mut m = new_contact_map(4, 4, 0.1);
        contact_set(&mut m, 0, 0, true);
        contact_set(&mut m, 1, 0, true);
        assert!((contact_area(&m) - 0.02).abs() < 1e-5);
    }

    #[test]
    fn test_contact_count() {
        let mut m = new_contact_map(4, 4, 0.01);
        contact_set(&mut m, 0, 0, true);
        contact_set(&mut m, 1, 1, true);
        contact_set(&mut m, 2, 2, true);
        assert_eq!(contact_count(&m), 3);
    }

    #[test]
    fn test_contact_to_bytes() {
        let mut m = new_contact_map(3, 1, 0.01);
        contact_set(&mut m, 1, 0, true);
        let b = contact_to_bytes(&m);
        assert_eq!(b, vec![0, 1, 0]);
    }

    #[test]
    fn test_contact_area_zero() {
        let m = new_contact_map(4, 4, 0.01);
        assert!((contact_area(&m) - 0.0).abs() < 1e-9);
    }
}
