// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Emissive map export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EmissiveMapExport { pub pixels: Vec<[f32;3]>, pub width: usize, pub height: usize, pub intensity: f32 }
#[allow(dead_code)]
pub fn new_emissive_map(w:usize,h:usize) -> EmissiveMapExport { EmissiveMapExport { pixels: vec![[0.0;3];w*h], width:w, height:h, intensity:1.0 } }
#[allow(dead_code)]
pub fn emissive_set_pixel(m:&mut EmissiveMapExport,x:usize,y:usize,c:[f32;3]) { if x<m.width&&y<m.height{m.pixels[y*m.width+x]=c;} }
#[allow(dead_code)]
pub fn emissive_get_pixel(m:&EmissiveMapExport,x:usize,y:usize)->[f32;3] { if x<m.width&&y<m.height{m.pixels[y*m.width+x]}else{[0.0;3]} }
#[allow(dead_code)]
pub fn emissive_set_intensity(m:&mut EmissiveMapExport,i:f32) { m.intensity=i.max(0.0); }
#[allow(dead_code)]
pub fn emissive_pixel_count(m:&EmissiveMapExport)->usize { m.pixels.len() }
#[allow(dead_code)]
pub fn emissive_avg_luminance(m:&EmissiveMapExport)->f32 { if m.pixels.is_empty(){0.0} else { m.pixels.iter().map(|p| p[0]*0.2126+p[1]*0.7152+p[2]*0.0722).sum::<f32>()/m.pixels.len() as f32 } }
#[allow(dead_code)]
pub fn emissive_to_json(m:&EmissiveMapExport)->String { format!("{{\"width\":{},\"height\":{},\"intensity\":{:.2}}}", m.width,m.height,m.intensity) }
#[allow(dead_code)]
pub fn emissive_validate(m:&EmissiveMapExport)->bool { m.pixels.len()==m.width*m.height }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new(){let m=new_emissive_map(4,4);assert_eq!(m.pixels.len(),16);}
    #[test] fn test_set_get(){let mut m=new_emissive_map(2,2);emissive_set_pixel(&mut m,0,0,[1.0,0.5,0.0]);let p=emissive_get_pixel(&m,0,0);assert!((p[0]-1.0).abs()<1e-6);}
    #[test] fn test_intensity(){let mut m=new_emissive_map(2,2);emissive_set_intensity(&mut m,2.5);assert!((m.intensity-2.5).abs()<1e-6);}
    #[test] fn test_pixel_count(){let m=new_emissive_map(3,3);assert_eq!(emissive_pixel_count(&m),9);}
    #[test] fn test_avg_luminance(){let m=new_emissive_map(2,2);assert!((emissive_avg_luminance(&m)).abs()<1e-6);}
    #[test] fn test_to_json(){let m=new_emissive_map(2,2);assert!(emissive_to_json(&m).contains("width"));}
    #[test] fn test_validate(){let m=new_emissive_map(2,2);assert!(emissive_validate(&m));}
    #[test] fn test_oob(){let m=new_emissive_map(2,2);let p=emissive_get_pixel(&m,99,99);assert!((p[0]).abs()<1e-6);}
    #[test] fn test_intensity_clamp(){let mut m=new_emissive_map(1,1);emissive_set_intensity(&mut m,-1.0);assert!((m.intensity).abs()<1e-6);}
    #[test] fn test_luminance_bright(){let mut m=new_emissive_map(1,1);emissive_set_pixel(&mut m,0,0,[1.0,1.0,1.0]);assert!(emissive_avg_luminance(&m)>0.9);}
}
