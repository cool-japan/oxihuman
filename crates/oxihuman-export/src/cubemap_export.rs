// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Cubemap export: export 6-face cubemap textures.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CubemapFace { pub pixels: Vec<[f32;3]>, pub width: usize, pub height: usize }
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CubemapExport { pub faces: [CubemapFace; 6], pub size: usize }

#[allow(dead_code)]
pub fn cubemap_face_names() -> [&'static str; 6] { ["pos_x","neg_x","pos_y","neg_y","pos_z","neg_z"] }
#[allow(dead_code)]
pub fn new_cubemap_export(size: usize) -> CubemapExport {
    let face = || CubemapFace { pixels: vec![[0.0;3]; size*size], width: size, height: size };
    CubemapExport { faces: [face(),face(),face(),face(),face(),face()], size }
}
#[allow(dead_code)]
pub fn cubemap_set_pixel(cm:&mut CubemapExport, face:usize, x:usize, y:usize, color:[f32;3]) {
    if face<6 && x<cm.size && y<cm.size { cm.faces[face].pixels[y*cm.size+x]=color; }
}
#[allow(dead_code)]
pub fn cubemap_get_pixel(cm:&CubemapExport, face:usize, x:usize, y:usize) -> [f32;3] {
    if face<6 && x<cm.size && y<cm.size { cm.faces[face].pixels[y*cm.size+x] } else { [0.0;3] }
}
#[allow(dead_code)]
pub fn cubemap_pixel_count(cm:&CubemapExport) -> usize { cm.size*cm.size*6 }
#[allow(dead_code)]
pub fn cubemap_face_pixel_count(cm:&CubemapExport) -> usize { cm.size*cm.size }
#[allow(dead_code)]
pub fn cubemap_to_json(cm:&CubemapExport) -> String { format!("{{\"size\":{},\"total_pixels\":{}}}", cm.size, cubemap_pixel_count(cm)) }
#[allow(dead_code)]
pub fn cubemap_validate(cm:&CubemapExport) -> bool { cm.faces.iter().all(|f| f.pixels.len()==cm.size*cm.size) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_face_names() { assert_eq!(cubemap_face_names().len(),6); }
    #[test] fn test_new() { let c=new_cubemap_export(4); assert_eq!(c.size,4); }
    #[test] fn test_set_get() { let mut c=new_cubemap_export(4); cubemap_set_pixel(&mut c,0,1,1,[1.0,0.0,0.0]); let p=cubemap_get_pixel(&c,0,1,1); assert!((p[0]-1.0).abs()<1e-6); }
    #[test] fn test_pixel_count() { let c=new_cubemap_export(4); assert_eq!(cubemap_pixel_count(&c),96); }
    #[test] fn test_face_pixel_count() { let c=new_cubemap_export(4); assert_eq!(cubemap_face_pixel_count(&c),16); }
    #[test] fn test_to_json() { let c=new_cubemap_export(2); assert!(cubemap_to_json(&c).contains("size")); }
    #[test] fn test_validate() { let c=new_cubemap_export(2); assert!(cubemap_validate(&c)); }
    #[test] fn test_oob() { let c=new_cubemap_export(2); let p=cubemap_get_pixel(&c,9,9,9); assert!((p[0]).abs()<1e-6); }
    #[test] fn test_size_1() { let c=new_cubemap_export(1); assert_eq!(cubemap_pixel_count(&c),6); }
    #[test] fn test_default_black() { let c=new_cubemap_export(2); let p=cubemap_get_pixel(&c,0,0,0); assert!((p[0]).abs()<1e-6); }
}
