// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! LOD export v2: enhanced LOD chain export with screen coverage.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodLevelV2 { pub vertex_count: usize, pub face_count: usize, pub screen_coverage: f32 }
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LodExportV2 { pub levels: Vec<LodLevelV2> }
#[allow(dead_code)]
pub fn new_lod_export_v2() -> LodExportV2 { LodExportV2 { levels: Vec::new() } }
#[allow(dead_code)]
pub fn lod_v2_add_level(e:&mut LodExportV2,vc:usize,fc:usize,sc:f32) { e.levels.push(LodLevelV2{vertex_count:vc,face_count:fc,screen_coverage:sc}); }
#[allow(dead_code)]
pub fn lod_v2_level_count(e:&LodExportV2)->usize { e.levels.len() }
#[allow(dead_code)]
pub fn lod_v2_total_vertices(e:&LodExportV2)->usize { e.levels.iter().map(|l|l.vertex_count).sum() }
#[allow(dead_code)]
pub fn lod_v2_total_faces(e:&LodExportV2)->usize { e.levels.iter().map(|l|l.face_count).sum() }
#[allow(dead_code)]
pub fn lod_v2_reduction_ratio(e:&LodExportV2,level:usize)->f32 {
    if level==0 || e.levels.is_empty() { return 1.0; }
    let base = e.levels[0].vertex_count as f32;
    if base<1.0{return 0.0;}
    e.levels.get(level).map_or(0.0, |l| l.vertex_count as f32/base)
}
#[allow(dead_code)]
pub fn lod_v2_to_json(e:&LodExportV2)->String { format!("{{\"levels\":{},\"total_verts\":{}}}", e.levels.len(), lod_v2_total_vertices(e)) }
#[allow(dead_code)]
pub fn lod_v2_validate(e:&LodExportV2)->bool { !e.levels.is_empty() && e.levels.windows(2).all(|w| w[0].vertex_count>=w[1].vertex_count) }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new(){let e=new_lod_export_v2();assert!(e.levels.is_empty());}
    #[test] fn test_add(){let mut e=new_lod_export_v2();lod_v2_add_level(&mut e,1000,500,1.0);assert_eq!(lod_v2_level_count(&e),1);}
    #[test] fn test_total_verts(){let mut e=new_lod_export_v2();lod_v2_add_level(&mut e,100,50,1.0);lod_v2_add_level(&mut e,50,25,0.5);assert_eq!(lod_v2_total_vertices(&e),150);}
    #[test] fn test_total_faces(){let mut e=new_lod_export_v2();lod_v2_add_level(&mut e,100,50,1.0);assert_eq!(lod_v2_total_faces(&e),50);}
    #[test] fn test_reduction(){let mut e=new_lod_export_v2();lod_v2_add_level(&mut e,100,50,1.0);lod_v2_add_level(&mut e,50,25,0.5);assert!((lod_v2_reduction_ratio(&e,1)-0.5).abs()<1e-6);}
    #[test] fn test_to_json(){let e=new_lod_export_v2();assert!(lod_v2_to_json(&e).contains("levels"));}
    #[test] fn test_validate_ok(){let mut e=new_lod_export_v2();lod_v2_add_level(&mut e,100,50,1.0);lod_v2_add_level(&mut e,50,25,0.5);assert!(lod_v2_validate(&e));}
    #[test] fn test_validate_empty(){let e=new_lod_export_v2();assert!(!lod_v2_validate(&e));}
    #[test] fn test_validate_bad_order(){let mut e=new_lod_export_v2();lod_v2_add_level(&mut e,50,25,0.5);lod_v2_add_level(&mut e,100,50,1.0);assert!(!lod_v2_validate(&e));}
    #[test] fn test_reduction_base(){let e=new_lod_export_v2();assert!((lod_v2_reduction_ratio(&e,0)-1.0).abs()<1e-6);}
}
