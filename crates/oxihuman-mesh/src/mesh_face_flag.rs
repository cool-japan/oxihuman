// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-face bitflags for tagging faces with multiple properties.

#[allow(dead_code)]
pub const FLAG_SELECTED: u32  = 1 << 0;
#[allow(dead_code)]
pub const FLAG_HIDDEN: u32    = 1 << 1;
#[allow(dead_code)]
pub const FLAG_MARKED: u32    = 1 << 2;
#[allow(dead_code)]
pub const FLAG_BOUNDARY: u32  = 1 << 3;
#[allow(dead_code)]
pub const FLAG_DEGENERATE: u32 = 1 << 4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceFlagSet {
    pub flags: Vec<u32>,
}

#[allow(dead_code)]
pub fn new_face_flags(face_count: usize) -> FaceFlagSet {
    FaceFlagSet { flags: vec![0; face_count] }
}

#[allow(dead_code)]
pub fn ff_set(ffs: &mut FaceFlagSet, face: usize, flag: u32) {
    if face < ffs.flags.len() { ffs.flags[face] |= flag; }
}

#[allow(dead_code)]
pub fn ff_clear(ffs: &mut FaceFlagSet, face: usize, flag: u32) {
    if face < ffs.flags.len() { ffs.flags[face] &= !flag; }
}

#[allow(dead_code)]
pub fn ff_test(ffs: &FaceFlagSet, face: usize, flag: u32) -> bool {
    face < ffs.flags.len() && (ffs.flags[face] & flag) != 0
}

#[allow(dead_code)]
pub fn ff_toggle(ffs: &mut FaceFlagSet, face: usize, flag: u32) {
    if face < ffs.flags.len() { ffs.flags[face] ^= flag; }
}

#[allow(dead_code)]
pub fn ff_count_with_flag(ffs: &FaceFlagSet, flag: u32) -> usize {
    ffs.flags.iter().filter(|&&f| (f & flag) != 0).count()
}

#[allow(dead_code)]
pub fn ff_faces_with_flag(ffs: &FaceFlagSet, flag: u32) -> Vec<usize> {
    ffs.flags.iter().enumerate().filter(|(_, &f)| (f & flag) != 0).map(|(i, _)| i).collect()
}

#[allow(dead_code)]
pub fn ff_clear_all(ffs: &mut FaceFlagSet) {
    for f in &mut ffs.flags { *f = 0; }
}

#[allow(dead_code)]
pub fn ff_face_count(ffs: &FaceFlagSet) -> usize { ffs.flags.len() }

#[allow(dead_code)]
pub fn ff_to_json(ffs: &FaceFlagSet) -> String {
    format!("{{\"faces\":{},\"selected\":{},\"hidden\":{}}}", ffs.flags.len(), ff_count_with_flag(ffs, FLAG_SELECTED), ff_count_with_flag(ffs, FLAG_HIDDEN))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_new() { let f = new_face_flags(10); assert_eq!(ff_face_count(&f), 10); }
    #[test] fn test_set() { let mut f = new_face_flags(5); ff_set(&mut f, 2, FLAG_SELECTED); assert!(ff_test(&f, 2, FLAG_SELECTED)); }
    #[test] fn test_clear() { let mut f = new_face_flags(5); ff_set(&mut f, 0, FLAG_HIDDEN); ff_clear(&mut f, 0, FLAG_HIDDEN); assert!(!ff_test(&f, 0, FLAG_HIDDEN)); }
    #[test] fn test_toggle() { let mut f = new_face_flags(5); ff_toggle(&mut f, 1, FLAG_MARKED); assert!(ff_test(&f, 1, FLAG_MARKED)); ff_toggle(&mut f, 1, FLAG_MARKED); assert!(!ff_test(&f, 1, FLAG_MARKED)); }
    #[test] fn test_count() { let mut f = new_face_flags(5); ff_set(&mut f, 0, FLAG_SELECTED); ff_set(&mut f, 3, FLAG_SELECTED); assert_eq!(ff_count_with_flag(&f, FLAG_SELECTED), 2); }
    #[test] fn test_faces_with() { let mut f = new_face_flags(5); ff_set(&mut f, 1, FLAG_BOUNDARY); assert_eq!(ff_faces_with_flag(&f, FLAG_BOUNDARY), vec![1]); }
    #[test] fn test_clear_all() { let mut f = new_face_flags(3); ff_set(&mut f, 0, FLAG_SELECTED); ff_clear_all(&mut f); assert_eq!(ff_count_with_flag(&f, FLAG_SELECTED), 0); }
    #[test] fn test_out_of_bounds() { let f = new_face_flags(2); assert!(!ff_test(&f, 10, FLAG_SELECTED)); }
    #[test] fn test_multi_flags() { let mut f = new_face_flags(3); ff_set(&mut f, 0, FLAG_SELECTED | FLAG_HIDDEN); assert!(ff_test(&f, 0, FLAG_SELECTED)); assert!(ff_test(&f, 0, FLAG_HIDDEN)); }
    #[test] fn test_to_json() { let f = new_face_flags(3); assert!(ff_to_json(&f).contains("faces")); }
}
