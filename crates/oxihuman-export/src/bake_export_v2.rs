// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Bake export v2: enhanced texture bake export with multiple channels.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BakeChannelV2 { pub name: String, pub width: usize, pub height: usize, pub pixels: Vec<f32> }
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BakeExportV2 { pub channels: Vec<BakeChannelV2>, pub samples: usize }
#[allow(dead_code)]
pub fn new_bake_export_v2(samples:usize) -> BakeExportV2 { BakeExportV2 { channels:Vec::new(), samples } }
#[allow(dead_code)]
pub fn bake_v2_add_channel(e:&mut BakeExportV2,name:&str,w:usize,h:usize) { e.channels.push(BakeChannelV2{name:name.to_string(),width:w,height:h,pixels:vec![0.0;w*h]}); }
#[allow(dead_code)]
pub fn bake_v2_channel_count(e:&BakeExportV2)->usize { e.channels.len() }
#[allow(dead_code)]
pub fn bake_v2_set_pixel(e:&mut BakeExportV2,ch:usize,x:usize,y:usize,v:f32) { if let Some(c)=e.channels.get_mut(ch) { if x<c.width&&y<c.height { c.pixels[y*c.width+x]=v; } } }
#[allow(dead_code)]
pub fn bake_v2_get_pixel(e:&BakeExportV2,ch:usize,x:usize,y:usize)->f32 { e.channels.get(ch).and_then(|c| if x<c.width&&y<c.height{Some(c.pixels[y*c.width+x])}else{None}).unwrap_or(0.0) }
#[allow(dead_code)]
pub fn bake_v2_total_pixels(e:&BakeExportV2)->usize { e.channels.iter().map(|c|c.pixels.len()).sum() }
#[allow(dead_code)]
pub fn bake_v2_to_json(e:&BakeExportV2)->String { format!("{{\"channels\":{},\"samples\":{},\"total_pixels\":{}}}", e.channels.len(), e.samples, bake_v2_total_pixels(e)) }
#[allow(dead_code)]
pub fn bake_v2_validate(e:&BakeExportV2)->bool { e.samples>0 && e.channels.iter().all(|c| c.pixels.len()==c.width*c.height) }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new(){let e=new_bake_export_v2(16);assert_eq!(e.samples,16);}
    #[test] fn test_add(){let mut e=new_bake_export_v2(16);bake_v2_add_channel(&mut e,"ao",4,4);assert_eq!(bake_v2_channel_count(&e),1);}
    #[test] fn test_set_get(){let mut e=new_bake_export_v2(16);bake_v2_add_channel(&mut e,"ao",4,4);bake_v2_set_pixel(&mut e,0,1,1,0.75);assert!((bake_v2_get_pixel(&e,0,1,1)-0.75).abs()<1e-6);}
    #[test] fn test_total(){let mut e=new_bake_export_v2(16);bake_v2_add_channel(&mut e,"ao",4,4);bake_v2_add_channel(&mut e,"nm",4,4);assert_eq!(bake_v2_total_pixels(&e),32);}
    #[test] fn test_to_json(){let e=new_bake_export_v2(16);assert!(bake_v2_to_json(&e).contains("channels"));}
    #[test] fn test_validate(){let mut e=new_bake_export_v2(16);bake_v2_add_channel(&mut e,"ao",2,2);assert!(bake_v2_validate(&e));}
    #[test] fn test_validate_bad(){let e=new_bake_export_v2(0);assert!(!bake_v2_validate(&e));}
    #[test] fn test_oob(){let e=new_bake_export_v2(16);assert!((bake_v2_get_pixel(&e,0,99,99)).abs()<1e-6);}
    #[test] fn test_channel_name(){let mut e=new_bake_export_v2(16);bake_v2_add_channel(&mut e,"curvature",4,4);assert_eq!(e.channels[0].name,"curvature");}
    #[test] fn test_empty(){let e=new_bake_export_v2(16);assert_eq!(bake_v2_total_pixels(&e),0);}
}
