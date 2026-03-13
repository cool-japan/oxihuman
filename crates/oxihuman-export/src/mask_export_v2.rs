// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Mask export v2: vertex/face selection mask export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaskExportV2 { pub bits: Vec<bool>, pub name: String }
#[allow(dead_code)]
pub fn new_mask_export_v2(name:&str,size:usize) -> MaskExportV2 { MaskExportV2 { bits:vec![false;size], name:name.to_string() } }
#[allow(dead_code)]
pub fn mask_v2_set(m:&mut MaskExportV2,idx:usize,val:bool) { if idx<m.bits.len(){m.bits[idx]=val;} }
#[allow(dead_code)]
pub fn mask_v2_get(m:&MaskExportV2,idx:usize)->bool { m.bits.get(idx).copied().unwrap_or(false) }
#[allow(dead_code)]
pub fn mask_v2_count_set(m:&MaskExportV2)->usize { m.bits.iter().filter(|&&b|b).count() }
#[allow(dead_code)]
pub fn mask_v2_count_clear(m:&MaskExportV2)->usize { m.bits.iter().filter(|&&b|!b).count() }
#[allow(dead_code)]
pub fn mask_v2_invert(m:&mut MaskExportV2) { for b in &mut m.bits { *b = !*b; } }
#[allow(dead_code)]
pub fn mask_v2_to_json(m:&MaskExportV2)->String { format!("{{\"name\":\"{}\",\"size\":{},\"set\":{}}}", m.name, m.bits.len(), mask_v2_count_set(m)) }
#[allow(dead_code)]
pub fn mask_v2_validate(m:&MaskExportV2)->bool { !m.name.is_empty() }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new(){let m=new_mask_export_v2("test",10);assert_eq!(m.bits.len(),10);}
    #[test] fn test_set_get(){let mut m=new_mask_export_v2("t",5);mask_v2_set(&mut m,2,true);assert!(mask_v2_get(&m,2));}
    #[test] fn test_count_set(){let mut m=new_mask_export_v2("t",4);mask_v2_set(&mut m,0,true);mask_v2_set(&mut m,1,true);assert_eq!(mask_v2_count_set(&m),2);}
    #[test] fn test_count_clear(){let m=new_mask_export_v2("t",4);assert_eq!(mask_v2_count_clear(&m),4);}
    #[test] fn test_invert(){let mut m=new_mask_export_v2("t",3);mask_v2_set(&mut m,0,true);mask_v2_invert(&mut m);assert!(!mask_v2_get(&m,0));assert!(mask_v2_get(&m,1));}
    #[test] fn test_to_json(){let m=new_mask_export_v2("face_mask",5);assert!(mask_v2_to_json(&m).contains("face_mask"));}
    #[test] fn test_validate(){let m=new_mask_export_v2("t",5);assert!(mask_v2_validate(&m));}
    #[test] fn test_validate_empty_name(){let m=new_mask_export_v2("",5);assert!(!mask_v2_validate(&m));}
    #[test] fn test_oob(){let m=new_mask_export_v2("t",3);assert!(!mask_v2_get(&m,99));}
    #[test] fn test_empty(){let m=new_mask_export_v2("t",0);assert_eq!(mask_v2_count_set(&m),0);}
}
