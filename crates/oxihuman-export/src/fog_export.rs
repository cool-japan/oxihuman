// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Fog settings export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FogExport { pub color: [f32;3], pub density: f32, pub start: f32, pub end: f32, pub fog_type: String }
#[allow(dead_code)]
pub fn default_fog_export() -> FogExport { FogExport { color:[0.7,0.7,0.8], density:0.01, start:10.0, end:100.0, fog_type:"linear".to_string() } }
#[allow(dead_code)]
pub fn fog_set_color(f:&mut FogExport,c:[f32;3]) { f.color=c; }
#[allow(dead_code)]
pub fn fog_set_density(f:&mut FogExport,d:f32) { f.density=d.max(0.0); }
#[allow(dead_code)]
pub fn fog_set_range(f:&mut FogExport,s:f32,e:f32) { f.start=s; f.end=e; }
#[allow(dead_code)]
pub fn fog_range(f:&FogExport)->f32 { f.end-f.start }
#[allow(dead_code)]
pub fn fog_to_json(f:&FogExport)->String { format!("{{\"type\":\"{}\",\"density\":{:.4},\"start\":{:.1},\"end\":{:.1}}}", f.fog_type, f.density, f.start, f.end) }
#[allow(dead_code)]
pub fn fog_validate(f:&FogExport)->bool { f.density>=0.0 && f.end>f.start }
#[allow(dead_code)]
pub fn fog_at_distance(f:&FogExport,d:f32)->f32 { if d<=f.start{0.0} else if d>=f.end{1.0} else { (d-f.start)/(f.end-f.start) } }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_default(){let f=default_fog_export();assert!((f.density-0.01).abs()<1e-6);}
    #[test] fn test_set_color(){let mut f=default_fog_export();fog_set_color(&mut f,[1.0,0.0,0.0]);assert!((f.color[0]-1.0).abs()<1e-6);}
    #[test] fn test_set_density(){let mut f=default_fog_export();fog_set_density(&mut f,0.05);assert!((f.density-0.05).abs()<1e-6);}
    #[test] fn test_range(){let f=default_fog_export();assert!((fog_range(&f)-90.0).abs()<1e-3);}
    #[test] fn test_to_json(){let f=default_fog_export();assert!(fog_to_json(&f).contains("density"));}
    #[test] fn test_validate(){let f=default_fog_export();assert!(fog_validate(&f));}
    #[test] fn test_at_distance_before(){let f=default_fog_export();assert!((fog_at_distance(&f,5.0)).abs()<1e-6);}
    #[test] fn test_at_distance_after(){let f=default_fog_export();assert!((fog_at_distance(&f,200.0)-1.0).abs()<1e-6);}
    #[test] fn test_at_distance_mid(){let f=default_fog_export();let v=fog_at_distance(&f,55.0);assert!(v>0.0 && v<1.0);}
    #[test] fn test_set_range(){let mut f=default_fog_export();fog_set_range(&mut f,0.0,50.0);assert!((f.end-50.0).abs()<1e-6);}
}
