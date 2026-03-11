// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Physics export v2: enhanced physics properties export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsObjectV2 { pub name: String, pub mass: f32, pub friction: f32, pub restitution: f32, pub is_kinematic: bool }
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysicsExportV2 { pub objects: Vec<PhysicsObjectV2>, pub gravity: [f32;3] }
#[allow(dead_code)]
pub fn new_physics_export_v2() -> PhysicsExportV2 { PhysicsExportV2 { objects:Vec::new(), gravity:[0.0,-9.81,0.0] } }
#[allow(dead_code)]
pub fn phys_v2_add(e:&mut PhysicsExportV2,name:&str,mass:f32) { e.objects.push(PhysicsObjectV2{name:name.to_string(),mass,friction:0.5,restitution:0.3,is_kinematic:false}); }
#[allow(dead_code)]
pub fn phys_v2_count(e:&PhysicsExportV2)->usize { e.objects.len() }
#[allow(dead_code)]
pub fn phys_v2_set_gravity(e:&mut PhysicsExportV2,g:[f32;3]) { e.gravity=g; }
#[allow(dead_code)]
pub fn phys_v2_total_mass(e:&PhysicsExportV2)->f32 { e.objects.iter().map(|o|o.mass).sum() }
#[allow(dead_code)]
pub fn phys_v2_kinematic_count(e:&PhysicsExportV2)->usize { e.objects.iter().filter(|o|o.is_kinematic).count() }
#[allow(dead_code)]
pub fn phys_v2_to_json(e:&PhysicsExportV2)->String { format!("{{\"objects\":{},\"gravity\":[{:.2},{:.2},{:.2}]}}", e.objects.len(), e.gravity[0], e.gravity[1], e.gravity[2]) }
#[allow(dead_code)]
pub fn phys_v2_validate(e:&PhysicsExportV2)->bool { e.objects.iter().all(|o| o.mass>=0.0 && (0.0..=1.0).contains(&o.friction) && (0.0..=1.0).contains(&o.restitution)) }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new(){let e=new_physics_export_v2();assert!(e.objects.is_empty());}
    #[test] fn test_add(){let mut e=new_physics_export_v2();phys_v2_add(&mut e,"box",1.0);assert_eq!(phys_v2_count(&e),1);}
    #[test] fn test_gravity(){let mut e=new_physics_export_v2();phys_v2_set_gravity(&mut e,[0.0,-10.0,0.0]);assert!((e.gravity[1]+10.0).abs()<1e-6);}
    #[test] fn test_total_mass(){let mut e=new_physics_export_v2();phys_v2_add(&mut e,"a",1.0);phys_v2_add(&mut e,"b",2.0);assert!((phys_v2_total_mass(&e)-3.0).abs()<1e-6);}
    #[test] fn test_kinematic(){let e=new_physics_export_v2();assert_eq!(phys_v2_kinematic_count(&e),0);}
    #[test] fn test_to_json(){let e=new_physics_export_v2();assert!(phys_v2_to_json(&e).contains("gravity"));}
    #[test] fn test_validate(){let mut e=new_physics_export_v2();phys_v2_add(&mut e,"a",1.0);assert!(phys_v2_validate(&e));}
    #[test] fn test_default_gravity(){let e=new_physics_export_v2();assert!((e.gravity[1]+9.81).abs()<1e-3);}
    #[test] fn test_empty_validate(){let e=new_physics_export_v2();assert!(phys_v2_validate(&e));}
    #[test] fn test_object_defaults(){let mut e=new_physics_export_v2();phys_v2_add(&mut e,"t",1.0);assert!((e.objects[0].friction-0.5).abs()<1e-6);}
}
