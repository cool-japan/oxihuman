// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Animation clip export v2: enhanced clip export with blending support.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimClipKeyframeV2 { pub time: f32, pub position: [f32;3], pub rotation: [f32;4] }
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimClipExportV2 { pub name: String, pub keyframes: Vec<AnimClipKeyframeV2>, pub fps: f32, pub looping: bool }
#[allow(dead_code)]
pub fn new_anim_clip_v2(name:&str,fps:f32) -> AnimClipExportV2 { AnimClipExportV2 { name:name.to_string(), keyframes:Vec::new(), fps, looping:false } }
#[allow(dead_code)]
pub fn acv2_add_keyframe(c:&mut AnimClipExportV2,t:f32,pos:[f32;3],rot:[f32;4]) { c.keyframes.push(AnimClipKeyframeV2{time:t,position:pos,rotation:rot}); }
#[allow(dead_code)]
pub fn acv2_keyframe_count(c:&AnimClipExportV2)->usize { c.keyframes.len() }
#[allow(dead_code)]
pub fn acv2_duration(c:&AnimClipExportV2)->f32 { c.keyframes.last().map_or(0.0,|k|k.time) }
#[allow(dead_code)]
pub fn acv2_set_looping(c:&mut AnimClipExportV2,l:bool) { c.looping=l; }
#[allow(dead_code)]
pub fn acv2_frame_count(c:&AnimClipExportV2)->usize { (acv2_duration(c)*c.fps) as usize }
#[allow(dead_code)]
pub fn acv2_to_json(c:&AnimClipExportV2)->String { format!("{{\"name\":\"{}\",\"keyframes\":{},\"fps\":{:.1},\"looping\":{}}}", c.name, c.keyframes.len(), c.fps, c.looping) }
#[allow(dead_code)]
pub fn acv2_validate(c:&AnimClipExportV2)->bool { c.fps>0.0 && c.keyframes.windows(2).all(|w| w[1].time>=w[0].time) }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new(){let c=new_anim_clip_v2("walk",30.0);assert_eq!(c.name,"walk");}
    #[test] fn test_add(){let mut c=new_anim_clip_v2("t",30.0);acv2_add_keyframe(&mut c,0.0,[0.0;3],[0.0,0.0,0.0,1.0]);assert_eq!(acv2_keyframe_count(&c),1);}
    #[test] fn test_duration(){let mut c=new_anim_clip_v2("t",30.0);acv2_add_keyframe(&mut c,0.0,[0.0;3],[0.0,0.0,0.0,1.0]);acv2_add_keyframe(&mut c,2.0,[1.0,0.0,0.0],[0.0,0.0,0.0,1.0]);assert!((acv2_duration(&c)-2.0).abs()<1e-6);}
    #[test] fn test_looping(){let mut c=new_anim_clip_v2("t",30.0);acv2_set_looping(&mut c,true);assert!(c.looping);}
    #[test] fn test_frame_count(){let mut c=new_anim_clip_v2("t",30.0);acv2_add_keyframe(&mut c,0.0,[0.0;3],[0.0,0.0,0.0,1.0]);acv2_add_keyframe(&mut c,1.0,[0.0;3],[0.0,0.0,0.0,1.0]);assert_eq!(acv2_frame_count(&c),30);}
    #[test] fn test_to_json(){let c=new_anim_clip_v2("idle",24.0);assert!(acv2_to_json(&c).contains("idle"));}
    #[test] fn test_validate_ok(){let mut c=new_anim_clip_v2("t",30.0);acv2_add_keyframe(&mut c,0.0,[0.0;3],[0.0,0.0,0.0,1.0]);assert!(acv2_validate(&c));}
    #[test] fn test_validate_bad_fps(){let c=AnimClipExportV2{name:"t".to_string(),keyframes:Vec::new(),fps:0.0,looping:false};assert!(!acv2_validate(&c));}
    #[test] fn test_empty_duration(){let c=new_anim_clip_v2("t",30.0);assert!((acv2_duration(&c)).abs()<1e-6);}
    #[test] fn test_validate_ordered(){let mut c=new_anim_clip_v2("t",30.0);acv2_add_keyframe(&mut c,0.0,[0.0;3],[0.0,0.0,0.0,1.0]);acv2_add_keyframe(&mut c,1.0,[0.0;3],[0.0,0.0,0.0,1.0]);assert!(acv2_validate(&c));}
}
