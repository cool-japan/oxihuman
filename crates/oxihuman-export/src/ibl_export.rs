// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Image-based lighting (IBL) export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IblExport { pub irradiance_size: usize, pub prefilter_size: usize, pub brdf_lut_size: usize, pub intensity: f32 }
#[allow(dead_code)]
pub fn default_ibl_export() -> IblExport { IblExport { irradiance_size:64, prefilter_size:128, brdf_lut_size:256, intensity:1.0 } }
#[allow(dead_code)]
pub fn ibl_set_intensity(e:&mut IblExport,i:f32) { e.intensity=i.max(0.0); }
#[allow(dead_code)]
pub fn ibl_total_pixels(e:&IblExport)->usize { e.irradiance_size*e.irradiance_size*6 + e.prefilter_size*e.prefilter_size*6 + e.brdf_lut_size*e.brdf_lut_size }
#[allow(dead_code)]
pub fn ibl_memory_estimate(e:&IblExport)->usize { ibl_total_pixels(e)*12 }
#[allow(dead_code)]
pub fn ibl_to_json(e:&IblExport)->String { format!("{{\"irradiance\":{},\"prefilter\":{},\"brdf_lut\":{},\"intensity\":{:.2}}}", e.irradiance_size, e.prefilter_size, e.brdf_lut_size, e.intensity) }
#[allow(dead_code)]
pub fn ibl_validate(e:&IblExport)->bool { e.irradiance_size>0 && e.prefilter_size>0 && e.brdf_lut_size>0 && e.intensity>=0.0 }
#[allow(dead_code)]
pub fn ibl_mip_levels(size:usize)->usize { (size as f32).log2() as usize + 1 }
#[allow(dead_code)]
pub fn ibl_irradiance_face_size(e:&IblExport)->usize { e.irradiance_size*e.irradiance_size }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_default(){let e=default_ibl_export();assert_eq!(e.irradiance_size,64);}
    #[test] fn test_set_intensity(){let mut e=default_ibl_export();ibl_set_intensity(&mut e,2.0);assert!((e.intensity-2.0).abs()<1e-6);}
    #[test] fn test_total_pixels(){let e=default_ibl_export();assert!(ibl_total_pixels(&e)>0);}
    #[test] fn test_memory(){let e=default_ibl_export();assert!(ibl_memory_estimate(&e)>0);}
    #[test] fn test_to_json(){let e=default_ibl_export();assert!(ibl_to_json(&e).contains("irradiance"));}
    #[test] fn test_validate(){let e=default_ibl_export();assert!(ibl_validate(&e));}
    #[test] fn test_mip_levels(){assert_eq!(ibl_mip_levels(256),9);}
    #[test] fn test_face_size(){let e=default_ibl_export();assert_eq!(ibl_irradiance_face_size(&e),64*64);}
    #[test] fn test_intensity_clamp(){let mut e=default_ibl_export();ibl_set_intensity(&mut e,-1.0);assert!((e.intensity).abs()<1e-6);}
    #[test] fn test_validate_bad(){let e=IblExport{irradiance_size:0,prefilter_size:0,brdf_lut_size:0,intensity:-1.0};assert!(!ibl_validate(&e));}
}
