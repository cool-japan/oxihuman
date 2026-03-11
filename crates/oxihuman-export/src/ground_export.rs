// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Ground plane export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GroundExport { pub size: f32, pub subdivisions: usize, pub color: [f32;3], pub y_offset: f32 }
#[allow(dead_code)]
pub fn default_ground_export() -> GroundExport { GroundExport { size:10.0, subdivisions:4, color:[0.5,0.5,0.5], y_offset:0.0 } }
#[allow(dead_code)]
pub fn ground_set_size(g:&mut GroundExport,s:f32) { g.size=s.max(0.1); }
#[allow(dead_code)]
pub fn ground_vertex_count(g:&GroundExport)->usize { (g.subdivisions+1)*(g.subdivisions+1) }
#[allow(dead_code)]
pub fn ground_face_count(g:&GroundExport)->usize { g.subdivisions*g.subdivisions*2 }
#[allow(dead_code)]
pub fn ground_area(g:&GroundExport)->f32 { g.size*g.size }
#[allow(dead_code)]
pub fn ground_to_json(g:&GroundExport)->String { format!("{{\"size\":{:.1},\"subdivisions\":{},\"y_offset\":{:.2}}}", g.size, g.subdivisions, g.y_offset) }
#[allow(dead_code)]
pub fn ground_validate(g:&GroundExport)->bool { g.size>0.0 && g.subdivisions>=1 }
#[allow(dead_code)]
pub fn ground_cell_size(g:&GroundExport)->f32 { g.size/g.subdivisions as f32 }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_default(){let g=default_ground_export();assert!((g.size-10.0).abs()<1e-6);}
    #[test] fn test_set_size(){let mut g=default_ground_export();ground_set_size(&mut g,20.0);assert!((g.size-20.0).abs()<1e-6);}
    #[test] fn test_vertex_count(){let g=default_ground_export();assert_eq!(ground_vertex_count(&g),25);}
    #[test] fn test_face_count(){let g=default_ground_export();assert_eq!(ground_face_count(&g),32);}
    #[test] fn test_area(){let g=default_ground_export();assert!((ground_area(&g)-100.0).abs()<1e-3);}
    #[test] fn test_to_json(){let g=default_ground_export();assert!(ground_to_json(&g).contains("size"));}
    #[test] fn test_validate(){let g=default_ground_export();assert!(ground_validate(&g));}
    #[test] fn test_validate_bad(){let g=GroundExport{size:0.0,subdivisions:0,color:[0.0;3],y_offset:0.0};assert!(!ground_validate(&g));}
    #[test] fn test_cell_size(){let g=default_ground_export();assert!((ground_cell_size(&g)-2.5).abs()<1e-6);}
    #[test] fn test_size_clamp(){let mut g=default_ground_export();ground_set_size(&mut g,-5.0);assert!(g.size>0.0);}
}
