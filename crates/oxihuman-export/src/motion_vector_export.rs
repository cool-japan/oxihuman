// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Motion vector export for temporal effects.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MotionVectorExport { pub vectors: Vec<[f32;2]>, pub width: usize, pub height: usize }
#[allow(dead_code)]
pub fn new_motion_vector_export(w:usize,h:usize) -> MotionVectorExport { MotionVectorExport { vectors: vec![[0.0;2];w*h], width:w, height:h } }
#[allow(dead_code)]
pub fn mv_set(e:&mut MotionVectorExport,x:usize,y:usize,v:[f32;2]) { if x<e.width&&y<e.height{e.vectors[y*e.width+x]=v;} }
#[allow(dead_code)]
pub fn mv_get(e:&MotionVectorExport,x:usize,y:usize)->[f32;2] { if x<e.width&&y<e.height{e.vectors[y*e.width+x]}else{[0.0;2]} }
#[allow(dead_code)]
pub fn mv_pixel_count(e:&MotionVectorExport)->usize { e.vectors.len() }
#[allow(dead_code)]
pub fn mv_max_magnitude(e:&MotionVectorExport)->f32 { e.vectors.iter().map(|v|(v[0]*v[0]+v[1]*v[1]).sqrt()).fold(0.0f32,f32::max) }
#[allow(dead_code)]
pub fn mv_avg_magnitude(e:&MotionVectorExport)->f32 { if e.vectors.is_empty(){0.0} else { e.vectors.iter().map(|v|(v[0]*v[0]+v[1]*v[1]).sqrt()).sum::<f32>()/e.vectors.len() as f32 } }
#[allow(dead_code)]
pub fn mv_to_json(e:&MotionVectorExport)->String { format!("{{\"width\":{},\"height\":{},\"pixels\":{}}}", e.width, e.height, e.vectors.len()) }
#[allow(dead_code)]
pub fn mv_validate(e:&MotionVectorExport)->bool { e.vectors.len()==e.width*e.height }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new(){let e=new_motion_vector_export(4,4);assert_eq!(e.vectors.len(),16);}
    #[test] fn test_set_get(){let mut e=new_motion_vector_export(2,2);mv_set(&mut e,0,0,[1.0,0.5]);let v=mv_get(&e,0,0);assert!((v[0]-1.0).abs()<1e-6);}
    #[test] fn test_pixel_count(){let e=new_motion_vector_export(3,3);assert_eq!(mv_pixel_count(&e),9);}
    #[test] fn test_max_magnitude(){let mut e=new_motion_vector_export(2,2);mv_set(&mut e,0,0,[3.0,4.0]);assert!((mv_max_magnitude(&e)-5.0).abs()<1e-5);}
    #[test] fn test_avg_magnitude(){let e=new_motion_vector_export(2,2);assert!((mv_avg_magnitude(&e)).abs()<1e-6);}
    #[test] fn test_to_json(){let e=new_motion_vector_export(2,2);assert!(mv_to_json(&e).contains("width"));}
    #[test] fn test_validate(){let e=new_motion_vector_export(2,2);assert!(mv_validate(&e));}
    #[test] fn test_oob(){let e=new_motion_vector_export(2,2);let v=mv_get(&e,99,99);assert!((v[0]).abs()<1e-6);}
    #[test] fn test_empty(){let e=new_motion_vector_export(0,0);assert!(e.vectors.is_empty());}
    #[test] fn test_magnitude_single(){let mut e=new_motion_vector_export(1,1);mv_set(&mut e,0,0,[1.0,0.0]);assert!((mv_max_magnitude(&e)-1.0).abs()<1e-6);}
}
