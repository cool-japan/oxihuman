// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export decal atlas (collection of decals packed into a texture atlas).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecalAtlasEntry {
    pub name: String,
    pub uv_offset: [f32; 2],
    pub uv_size: [f32; 2],
    pub world_size: [f32; 2],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecalAtlasExport {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub entries: Vec<DecalAtlasEntry>,
}

#[allow(dead_code)]
pub fn new_decal_atlas_export(width: u32, height: u32) -> DecalAtlasExport {
    DecalAtlasExport { atlas_width: width, atlas_height: height, entries: Vec::new() }
}

#[allow(dead_code)]
pub fn dae_add(dae: &mut DecalAtlasExport, name: &str, uv_off: [f32; 2], uv_sz: [f32; 2], world_sz: [f32; 2]) {
    dae.entries.push(DecalAtlasEntry { name: name.to_string(), uv_offset: uv_off, uv_size: uv_sz, world_size: world_sz });
}

#[allow(dead_code)]
pub fn dae_count(dae: &DecalAtlasExport) -> usize { dae.entries.len() }

#[allow(dead_code)]
pub fn dae_find<'a>(dae: &'a DecalAtlasExport, name: &str) -> Option<&'a DecalAtlasEntry> {
    dae.entries.iter().find(|e| e.name == name)
}

#[allow(dead_code)]
pub fn dae_total_uv_area(dae: &DecalAtlasExport) -> f32 {
    dae.entries.iter().map(|e| e.uv_size[0] * e.uv_size[1]).sum()
}

#[allow(dead_code)]
pub fn dae_utilization(dae: &DecalAtlasExport) -> f32 {
    let total = dae_total_uv_area(dae);
    total.min(1.0)
}

#[allow(dead_code)]
pub fn dae_validate(dae: &DecalAtlasExport) -> bool {
    dae.atlas_width > 0 && dae.atlas_height > 0 && dae.entries.iter().all(|e| e.uv_size[0] > 0.0 && e.uv_size[1] > 0.0)
}

#[allow(dead_code)]
pub fn dae_to_json(dae: &DecalAtlasExport) -> String {
    format!("{{\"atlas\":[{},{}],\"decals\":{},\"utilization\":{:.4}}}", dae.atlas_width, dae.atlas_height, dae.entries.len(), dae_utilization(dae))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> DecalAtlasExport {
        let mut d = new_decal_atlas_export(1024, 1024);
        dae_add(&mut d, "blood", [0.0, 0.0], [0.25, 0.25], [1.0, 1.0]);
        dae_add(&mut d, "scratch", [0.25, 0.0], [0.25, 0.25], [0.5, 0.5]);
        d
    }

    #[test] fn test_new() { let d = new_decal_atlas_export(512, 512); assert_eq!(dae_count(&d), 0); }
    #[test] fn test_add() { assert_eq!(dae_count(&sample()), 2); }
    #[test] fn test_find() { assert!(dae_find(&sample(), "blood").is_some()); }
    #[test] fn test_find_missing() { assert!(dae_find(&sample(), "nope").is_none()); }
    #[test] fn test_uv_area() { let a = dae_total_uv_area(&sample()); assert!(a > 0.0); }
    #[test] fn test_utilization() { let u = dae_utilization(&sample()); assert!((0.0..=1.0).contains(&u)); }
    #[test] fn test_validate() { assert!(dae_validate(&sample())); }
    #[test] fn test_to_json() { assert!(dae_to_json(&sample()).contains("utilization")); }
    #[test] fn test_atlas_size() { let d = sample(); assert_eq!(d.atlas_width, 1024); }
    #[test] fn test_world_size() { let d = sample(); assert!((d.entries[0].world_size[0] - 1.0).abs() < 1e-6); }
}
