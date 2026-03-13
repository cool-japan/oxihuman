// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Fluid simulation cache export stub.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FluidExportConfig {
    pub grid_res: [u32; 3],
    pub voxel_size: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FluidFrame {
    pub frame: u32,
    pub density: Vec<f32>,
    pub velocity: Vec<[f32; 3]>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FluidCacheExport {
    pub config: FluidExportConfig,
    pub frames: Vec<FluidFrame>,
}

#[allow(dead_code)]
pub fn default_fluid_export_config() -> FluidExportConfig {
    FluidExportConfig {
        grid_res: [32, 32, 32],
        voxel_size: 0.1,
    }
}

#[allow(dead_code)]
pub fn new_fluid_cache_export(config: FluidExportConfig) -> FluidCacheExport {
    FluidCacheExport { config, frames: Vec::new() }
}

#[allow(dead_code)]
pub fn fluid_voxel_count(export: &FluidCacheExport) -> usize {
    let r = &export.config.grid_res;
    (r[0] * r[1] * r[2]) as usize
}

#[allow(dead_code)]
pub fn fluid_add_frame(export: &mut FluidCacheExport, frame: FluidFrame) {
    export.frames.push(frame);
}

#[allow(dead_code)]
pub fn fluid_frame_count(export: &FluidCacheExport) -> usize {
    export.frames.len()
}

#[allow(dead_code)]
pub fn fluid_get_frame(export: &FluidCacheExport, index: usize) -> Option<&FluidFrame> {
    export.frames.get(index)
}

#[allow(dead_code)]
pub fn fluid_validate(export: &FluidCacheExport) -> bool {
    let r = &export.config.grid_res;
    r[0] > 0 && r[1] > 0 && r[2] > 0 && export.config.voxel_size > 0.0
}

#[allow(dead_code)]
pub fn fluid_to_json(export: &FluidCacheExport) -> String {
    let r = &export.config.grid_res;
    format!(
        "{{\"grid_res\":[{},{},{}],\"voxel_size\":{},\"frame_count\":{}}}",
        r[0], r[1], r[2],
        export.config.voxel_size,
        export.frames.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(id: u32, n: usize) -> FluidFrame {
        FluidFrame {
            frame: id,
            density: vec![0.5; n],
            velocity: vec![[0.0, 1.0, 0.0]; n],
        }
    }

    #[test]
    fn test_default_config() {
        let cfg = default_fluid_export_config();
        assert_eq!(cfg.grid_res, [32, 32, 32]);
    }

    #[test]
    fn test_voxel_count() {
        let exp = new_fluid_cache_export(default_fluid_export_config());
        assert_eq!(fluid_voxel_count(&exp), 32 * 32 * 32);
    }

    #[test]
    fn test_add_frame() {
        let mut exp = new_fluid_cache_export(default_fluid_export_config());
        fluid_add_frame(&mut exp, make_frame(0, 10));
        assert_eq!(fluid_frame_count(&exp), 1);
    }

    #[test]
    fn test_get_frame() {
        let mut exp = new_fluid_cache_export(default_fluid_export_config());
        fluid_add_frame(&mut exp, make_frame(5, 4));
        assert_eq!(fluid_get_frame(&exp, 0).expect("should succeed").frame, 5);
        assert!(fluid_get_frame(&exp, 1).is_none());
    }

    #[test]
    fn test_validate_ok() {
        let exp = new_fluid_cache_export(default_fluid_export_config());
        assert!(fluid_validate(&exp));
    }

    #[test]
    fn test_validate_bad_voxel() {
        let cfg = FluidExportConfig { grid_res: [32, 32, 32], voxel_size: 0.0 };
        let exp = new_fluid_cache_export(cfg);
        assert!(!fluid_validate(&exp));
    }

    #[test]
    fn test_validate_bad_res() {
        let cfg = FluidExportConfig { grid_res: [0, 32, 32], voxel_size: 0.1 };
        let exp = new_fluid_cache_export(cfg);
        assert!(!fluid_validate(&exp));
    }

    #[test]
    fn test_to_json() {
        let exp = new_fluid_cache_export(default_fluid_export_config());
        let j = fluid_to_json(&exp);
        assert!(j.contains("grid_res"));
    }
}
