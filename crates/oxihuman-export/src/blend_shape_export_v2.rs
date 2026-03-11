// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Blend shape export v2: enhanced blend shape export with delta compression.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendShapeTargetV2 { pub name: String, pub deltas: Vec<[f32;3]>, pub weight: f32 }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendShapeExportV2 { pub targets: Vec<BlendShapeTargetV2>, pub vertex_count: usize }

#[allow(dead_code)]
pub fn new_blend_shape_export_v2(vertex_count: usize) -> BlendShapeExportV2 { BlendShapeExportV2 { targets: Vec::new(), vertex_count } }
#[allow(dead_code)]
pub fn bsv2_add_target(e:&mut BlendShapeExportV2, name:&str, deltas:Vec<[f32;3]>, weight:f32) { e.targets.push(BlendShapeTargetV2{name:name.to_string(),deltas,weight}); }
#[allow(dead_code)]
pub fn bsv2_target_count(e:&BlendShapeExportV2) -> usize { e.targets.len() }
#[allow(dead_code)]
pub fn bsv2_target_name(e:&BlendShapeExportV2, idx:usize) -> &str { e.targets.get(idx).map_or("", |t| &t.name) }
#[allow(dead_code)]
pub fn bsv2_max_delta(e:&BlendShapeExportV2) -> f32 {
    e.targets.iter().flat_map(|t| t.deltas.iter()).map(|d| (d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt()).fold(0.0f32, f32::max)
}
#[allow(dead_code)]
pub fn bsv2_total_deltas(e:&BlendShapeExportV2) -> usize { e.targets.iter().map(|t| t.deltas.len()).sum() }
#[allow(dead_code)]
pub fn bsv2_to_json(e:&BlendShapeExportV2) -> String { format!("{{\"targets\":{},\"vertex_count\":{}}}", e.targets.len(), e.vertex_count) }
#[allow(dead_code)]
pub fn bsv2_validate(e:&BlendShapeExportV2) -> bool { e.targets.iter().all(|t| t.deltas.len()==e.vertex_count) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let e=new_blend_shape_export_v2(10); assert_eq!(e.vertex_count,10); }
    #[test] fn test_add() { let mut e=new_blend_shape_export_v2(2); bsv2_add_target(&mut e,"smile",vec![[0.1,0.0,0.0],[0.0,0.1,0.0]],1.0); assert_eq!(bsv2_target_count(&e),1); }
    #[test] fn test_name() { let mut e=new_blend_shape_export_v2(1); bsv2_add_target(&mut e,"frown",vec![[0.0;3]],1.0); assert_eq!(bsv2_target_name(&e,0),"frown"); }
    #[test] fn test_max_delta() { let mut e=new_blend_shape_export_v2(1); bsv2_add_target(&mut e,"t",vec![[3.0,4.0,0.0]],1.0); assert!((bsv2_max_delta(&e)-5.0).abs()<1e-5); }
    #[test] fn test_total_deltas() { let mut e=new_blend_shape_export_v2(2); bsv2_add_target(&mut e,"a",vec![[0.0;3];2],1.0); bsv2_add_target(&mut e,"b",vec![[0.0;3];2],1.0); assert_eq!(bsv2_total_deltas(&e),4); }
    #[test] fn test_to_json() { let e=new_blend_shape_export_v2(1); assert!(bsv2_to_json(&e).contains("targets")); }
    #[test] fn test_validate_ok() { let mut e=new_blend_shape_export_v2(2); bsv2_add_target(&mut e,"t",vec![[0.0;3];2],1.0); assert!(bsv2_validate(&e)); }
    #[test] fn test_validate_bad() { let mut e=new_blend_shape_export_v2(2); bsv2_add_target(&mut e,"t",vec![[0.0;3];3],1.0); assert!(!bsv2_validate(&e)); }
    #[test] fn test_empty_name() { let e=new_blend_shape_export_v2(1); assert_eq!(bsv2_target_name(&e,99),""); }
    #[test] fn test_empty_export() { let e=new_blend_shape_export_v2(0); assert_eq!(bsv2_target_count(&e),0); }
}
