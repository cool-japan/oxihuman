// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export billboard (impostor/sprite) representations of 3D objects.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum BillboardMode { ScreenAligned, AxisAligned, ViewPlane }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BillboardExport {
    pub name: String,
    pub mode: BillboardMode,
    pub position: [f32; 3],
    pub width: f32,
    pub height: f32,
    pub texture_id: u32,
}

#[allow(dead_code)]
pub fn new_billboard_export(name: &str, mode: BillboardMode, position: [f32; 3], width: f32, height: f32) -> BillboardExport {
    BillboardExport { name: name.to_string(), mode, position, width: width.max(0.0), height: height.max(0.0), texture_id: 0 }
}

#[allow(dead_code)]
pub fn bb_set_texture(bb: &mut BillboardExport, tex_id: u32) { bb.texture_id = tex_id; }

#[allow(dead_code)]
pub fn bb_area(bb: &BillboardExport) -> f32 { bb.width * bb.height }

#[allow(dead_code)]
pub fn bb_aspect_ratio(bb: &BillboardExport) -> f32 {
    if bb.height.abs() < 1e-12 { return 0.0; }
    bb.width / bb.height
}

#[allow(dead_code)]
pub fn bb_validate(bb: &BillboardExport) -> bool {
    bb.width > 0.0 && bb.height > 0.0 && !bb.name.is_empty()
}

#[allow(dead_code)]
pub fn bb_distance_to(bb: &BillboardExport, eye: [f32; 3]) -> f32 {
    let d = [bb.position[0]-eye[0], bb.position[1]-eye[1], bb.position[2]-eye[2]];
    (d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt()
}

#[allow(dead_code)]
pub fn bb_to_json(bb: &BillboardExport) -> String {
    format!("{{\"name\":\"{}\",\"width\":{:.2},\"height\":{:.2},\"area\":{:.4}}}", bb.name, bb.width, bb.height, bb_area(bb))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bb() -> BillboardExport { new_billboard_export("tree", BillboardMode::ScreenAligned, [0.0,1.0,0.0], 2.0, 3.0) }

    #[test] fn test_new() { let b = bb(); assert_eq!(b.name, "tree"); }
    #[test] fn test_area() { assert!((bb_area(&bb()) - 6.0).abs() < 1e-5); }
    #[test] fn test_aspect() { assert!((bb_aspect_ratio(&bb()) - 2.0/3.0).abs() < 1e-5); }
    #[test] fn test_validate() { assert!(bb_validate(&bb())); }
    #[test] fn test_distance() { assert!((bb_distance_to(&bb(), [0.0,1.0,0.0])).abs() < 1e-5); }
    #[test] fn test_set_texture() { let mut b = bb(); bb_set_texture(&mut b, 42); assert_eq!(b.texture_id, 42); }
    #[test] fn test_to_json() { assert!(bb_to_json(&bb()).contains("tree")); }
    #[test] fn test_mode() { let b = bb(); assert_eq!(b.mode, BillboardMode::ScreenAligned); }
    #[test] fn test_zero_height() { let b = new_billboard_export("x", BillboardMode::AxisAligned, [0.0,0.0,0.0], 1.0, 0.0); assert!((bb_aspect_ratio(&b)).abs() < 1e-6); }
    #[test] fn test_invalid() { let b = new_billboard_export("", BillboardMode::ViewPlane, [0.0,0.0,0.0], 1.0, 1.0); assert!(!bb_validate(&b)); }
}
