// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Cloth export v2: enhanced cloth simulation export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothPinV2 { pub vertex_index: u32, pub stiffness: f32 }
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothExportV2 { pub vertex_count: usize, pub pins: Vec<ClothPinV2>, pub mass: f32, pub damping: f32, pub stiffness: f32 }
#[allow(dead_code)]
pub fn default_cloth_export_v2(vc:usize) -> ClothExportV2 { ClothExportV2 { vertex_count:vc, pins:Vec::new(), mass:1.0, damping:0.1, stiffness:0.8 } }
#[allow(dead_code)]
pub fn cloth_v2_add_pin(c:&mut ClothExportV2,vi:u32,stiffness:f32) { c.pins.push(ClothPinV2{vertex_index:vi,stiffness:stiffness.clamp(0.0,1.0)}); }
#[allow(dead_code)]
pub fn cloth_v2_pin_count(c:&ClothExportV2)->usize { c.pins.len() }
#[allow(dead_code)]
pub fn cloth_v2_set_mass(c:&mut ClothExportV2,m:f32) { c.mass=m.max(0.001); }
#[allow(dead_code)]
pub fn cloth_v2_set_damping(c:&mut ClothExportV2,d:f32) { c.damping=d.clamp(0.0,1.0); }
#[allow(dead_code)]
pub fn cloth_v2_to_json(c:&ClothExportV2)->String { format!("{{\"vertices\":{},\"pins\":{},\"mass\":{:.2},\"damping\":{:.2}}}", c.vertex_count, c.pins.len(), c.mass, c.damping) }
#[allow(dead_code)]
pub fn cloth_v2_validate(c:&ClothExportV2)->bool { c.mass>0.0 && (0.0..=1.0).contains(&c.damping) && (0.0..=1.0).contains(&c.stiffness) }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_default(){let c=default_cloth_export_v2(100);assert_eq!(c.vertex_count,100);}
    #[test] fn test_add_pin(){let mut c=default_cloth_export_v2(10);cloth_v2_add_pin(&mut c,0,1.0);assert_eq!(cloth_v2_pin_count(&c),1);}
    #[test] fn test_set_mass(){let mut c=default_cloth_export_v2(10);cloth_v2_set_mass(&mut c,2.0);assert!((c.mass-2.0).abs()<1e-6);}
    #[test] fn test_set_damping(){let mut c=default_cloth_export_v2(10);cloth_v2_set_damping(&mut c,0.5);assert!((c.damping-0.5).abs()<1e-6);}
    #[test] fn test_to_json(){let c=default_cloth_export_v2(10);assert!(cloth_v2_to_json(&c).contains("vertices"));}
    #[test] fn test_validate(){let c=default_cloth_export_v2(10);assert!(cloth_v2_validate(&c));}
    #[test] fn test_mass_clamp(){let mut c=default_cloth_export_v2(10);cloth_v2_set_mass(&mut c,-1.0);assert!(c.mass>0.0);}
    #[test] fn test_damping_clamp(){let mut c=default_cloth_export_v2(10);cloth_v2_set_damping(&mut c,2.0);assert!((c.damping-1.0).abs()<1e-6);}
    #[test] fn test_pin_stiffness_clamp(){let mut c=default_cloth_export_v2(10);cloth_v2_add_pin(&mut c,0,5.0);assert!((c.pins[0].stiffness-1.0).abs()<1e-6);}
    #[test] fn test_empty_pins(){let c=default_cloth_export_v2(10);assert_eq!(cloth_v2_pin_count(&c),0);}
}
