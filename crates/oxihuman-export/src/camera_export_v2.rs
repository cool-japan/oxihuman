// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Camera export v2: enhanced camera data export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CameraExportV2 { pub fov: f32, pub near: f32, pub far: f32, pub position: [f32;3], pub target: [f32;3], pub up: [f32;3], pub projection: String }
#[allow(dead_code)]
pub fn default_camera_export_v2() -> CameraExportV2 { CameraExportV2 { fov:60.0, near:0.1, far:1000.0, position:[0.0,1.0,5.0], target:[0.0;3], up:[0.0,1.0,0.0], projection:"perspective".to_string() } }
#[allow(dead_code)]
pub fn cam_v2_set_fov(c:&mut CameraExportV2,f:f32) { c.fov=f.clamp(1.0,179.0); }
#[allow(dead_code)]
pub fn cam_v2_set_position(c:&mut CameraExportV2,p:[f32;3]) { c.position=p; }
#[allow(dead_code)]
pub fn cam_v2_set_target(c:&mut CameraExportV2,t:[f32;3]) { c.target=t; }
#[allow(dead_code)]
pub fn cam_v2_aspect_ratio(w:usize,h:usize)->f32 { if h==0{1.0} else {w as f32/h as f32} }
#[allow(dead_code)]
pub fn cam_v2_direction(c:&CameraExportV2)->[f32;3] { let d=[c.target[0]-c.position[0],c.target[1]-c.position[1],c.target[2]-c.position[2]]; let l=(d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt(); if l<1e-12{[0.0,0.0,-1.0]} else {[d[0]/l,d[1]/l,d[2]/l]} }
#[allow(dead_code)]
pub fn cam_v2_to_json(c:&CameraExportV2)->String { format!("{{\"fov\":{:.1},\"near\":{:.2},\"far\":{:.1},\"projection\":\"{}\"}}", c.fov, c.near, c.far, c.projection) }
#[allow(dead_code)]
pub fn cam_v2_validate(c:&CameraExportV2)->bool { c.fov>0.0 && c.near>0.0 && c.far>c.near }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_default(){let c=default_camera_export_v2();assert!((c.fov-60.0).abs()<1e-6);}
    #[test] fn test_set_fov(){let mut c=default_camera_export_v2();cam_v2_set_fov(&mut c,90.0);assert!((c.fov-90.0).abs()<1e-6);}
    #[test] fn test_set_position(){let mut c=default_camera_export_v2();cam_v2_set_position(&mut c,[1.0,2.0,3.0]);assert!((c.position[0]-1.0).abs()<1e-6);}
    #[test] fn test_aspect(){assert!((cam_v2_aspect_ratio(1920,1080)-1920.0/1080.0).abs()<1e-3);}
    #[test] fn test_direction(){let c=default_camera_export_v2();let d=cam_v2_direction(&c);let l=(d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt();assert!((l-1.0).abs()<1e-5);}
    #[test] fn test_to_json(){let c=default_camera_export_v2();assert!(cam_v2_to_json(&c).contains("fov"));}
    #[test] fn test_validate(){let c=default_camera_export_v2();assert!(cam_v2_validate(&c));}
    #[test] fn test_validate_bad(){let c=CameraExportV2{fov:0.0,near:0.0,far:0.0,position:[0.0;3],target:[0.0;3],up:[0.0,1.0,0.0],projection:"perspective".to_string()};assert!(!cam_v2_validate(&c));}
    #[test] fn test_fov_clamp(){let mut c=default_camera_export_v2();cam_v2_set_fov(&mut c,200.0);assert!((c.fov-179.0).abs()<1e-6);}
    #[test] fn test_aspect_zero_h(){assert!((cam_v2_aspect_ratio(100,0)-1.0).abs()<1e-6);}
}
