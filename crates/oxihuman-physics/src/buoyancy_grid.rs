//! Voxel-grid based buoyancy calculation for partially submerged objects.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BuoyancyGridConfig {
    pub voxel_size: f32,
    pub fluid_density: f32,
    pub gravity: f32,
    pub grid_extents: [f32; 3],
    pub grid_origin: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BuoyancyVoxel {
    pub center: [f32; 3],
    pub submerged_fraction: f32,
    pub volume: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BuoyancyGridResult {
    pub force: [f32; 3],
    pub torque: [f32; 3],
    pub submerged_volume: f32,
    pub voxel_count: usize,
}

#[allow(dead_code)]
pub struct BuoyancyGrid {
    config: BuoyancyGridConfig,
    voxels: Vec<BuoyancyVoxel>,
    water_level: f32,
    body_center: [f32; 3],
}

fn voxel_count_along(extent: f32, voxel_size: f32) -> usize {
    ((extent / voxel_size).ceil() as usize).max(1)
}

#[allow(dead_code)]
pub fn default_buoyancy_grid_config() -> BuoyancyGridConfig {
    BuoyancyGridConfig {
        voxel_size: 0.1,
        fluid_density: 1000.0,
        gravity: 9.81,
        grid_extents: [1.0, 1.0, 1.0],
        grid_origin: [-0.5, -0.5, -0.5],
    }
}

#[allow(dead_code)]
pub fn new_buoyancy_grid(config: BuoyancyGridConfig) -> BuoyancyGrid {
    let nx = voxel_count_along(config.grid_extents[0], config.voxel_size);
    let ny = voxel_count_along(config.grid_extents[1], config.voxel_size);
    let nz = voxel_count_along(config.grid_extents[2], config.voxel_size);
    let vol = config.voxel_size.powi(3);
    let mut voxels = Vec::with_capacity(nx * ny * nz);
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let cx = config.grid_origin[0] + (ix as f32 + 0.5) * config.voxel_size;
                let cy = config.grid_origin[1] + (iy as f32 + 0.5) * config.voxel_size;
                let cz = config.grid_origin[2] + (iz as f32 + 0.5) * config.voxel_size;
                voxels.push(BuoyancyVoxel { center: [cx, cy, cz], submerged_fraction: 0.0, volume: vol });
            }
        }
    }
    BuoyancyGrid { config, voxels, water_level: 0.0, body_center: [0.0; 3] }
}

fn submerged_fraction(cy: f32, half: f32, water_y: f32) -> f32 {
    let top = cy + half;
    let bot = cy - half;
    if water_y <= bot { return 0.0; }
    if water_y >= top { return 1.0; }
    ((water_y - bot) / (top - bot)).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn buoyancy_grid_step(grid: &mut BuoyancyGrid, body_pos: [f32; 3], water_level: f32) -> BuoyancyGridResult {
    grid.water_level = water_level;
    grid.body_center = body_pos;
    let half = grid.config.voxel_size * 0.5;
    let mut force = [0.0f32; 3];
    let mut torque = [0.0f32; 3];
    let mut sub_vol = 0.0f32;

    for v in &mut grid.voxels {
        let world_cy = body_pos[1] + v.center[1];
        let frac = submerged_fraction(world_cy, half, water_level);
        v.submerged_fraction = frac;
        let sub_v = frac * v.volume;
        sub_vol += sub_v;
        let buoy_f = grid.config.fluid_density * grid.config.gravity * sub_v;
        force[1] += buoy_f;

        // Torque: r x F, with F = (0, buoy_f, 0)
        // tx = ry*Fz - rz*Fy = -rz * buoy_f
        // ty = rz*Fx - rx*Fz = 0
        // tz = rx*Fy - ry*Fx = rx * buoy_f
        let rx = v.center[0];
        let rz = v.center[2];
        torque[0] += -rz * buoy_f;
        // torque[1] stays 0
        torque[2] += rx * buoy_f;
    }

    BuoyancyGridResult {
        force,
        torque,
        submerged_volume: sub_vol,
        voxel_count: grid.voxels.len(),
    }
}

#[allow(dead_code)]
pub fn buoyancy_grid_submerged_volume(result: &BuoyancyGridResult) -> f32 {
    result.submerged_volume
}

#[allow(dead_code)]
pub fn buoyancy_grid_voxel_count(grid: &BuoyancyGrid) -> usize {
    grid.voxels.len()
}

#[allow(dead_code)]
pub fn buoyancy_grid_force(result: &BuoyancyGridResult) -> [f32; 3] {
    result.force
}

#[allow(dead_code)]
pub fn buoyancy_grid_torque(result: &BuoyancyGridResult) -> [f32; 3] {
    result.torque
}

#[allow(dead_code)]
pub fn buoyancy_grid_to_json(grid: &BuoyancyGrid) -> String {
    format!(
        "{{\"voxel_count\":{},\"water_level\":{:.4},\"voxel_size\":{:.4},\"fluid_density\":{:.4}}}",
        grid.voxels.len(), grid.water_level, grid.config.voxel_size, grid.config.fluid_density
    )
}

#[allow(dead_code)]
pub fn buoyancy_grid_reset(grid: &mut BuoyancyGrid) {
    for v in &mut grid.voxels {
        v.submerged_fraction = 0.0;
    }
    grid.water_level = 0.0;
    grid.body_center = [0.0; 3];
}

#[allow(dead_code)]
pub fn buoyancy_grid_water_level(grid: &BuoyancyGrid) -> f32 {
    grid.water_level
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> BuoyancyGrid {
        new_buoyancy_grid(default_buoyancy_grid_config())
    }

    #[test]
    fn test_default_config() {
        let cfg = default_buoyancy_grid_config();
        assert!(cfg.fluid_density > 0.0);
        assert!(cfg.voxel_size > 0.0);
    }

    #[test]
    fn test_voxel_count_nonzero() {
        let g = make_grid();
        assert!(buoyancy_grid_voxel_count(&g) > 0);
    }

    #[test]
    fn test_fully_submerged_force_positive() {
        let mut g = make_grid();
        let res = buoyancy_grid_step(&mut g, [0.0, 0.0, 0.0], 100.0);
        assert!(buoyancy_grid_force(&res)[1] > 0.0);
    }

    #[test]
    fn test_fully_above_water_zero_force() {
        let mut g = make_grid();
        let res = buoyancy_grid_step(&mut g, [0.0, 100.0, 0.0], 0.0);
        assert!((buoyancy_grid_submerged_volume(&res) - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_partial_submersion() {
        let mut g = make_grid();
        let res_half = buoyancy_grid_step(&mut g, [0.0, 0.0, 0.0], 0.0);
        let res_full = buoyancy_grid_step(&mut g, [0.0, 0.0, 0.0], 100.0);
        assert!(buoyancy_grid_submerged_volume(&res_full) >= buoyancy_grid_submerged_volume(&res_half));
    }

    #[test]
    fn test_water_level_accessor() {
        let mut g = make_grid();
        buoyancy_grid_step(&mut g, [0.0; 3], std::f32::consts::PI);
        assert!((buoyancy_grid_water_level(&g) - std::f32::consts::PI).abs() < 1e-5);
    }

    #[test]
    fn test_reset() {
        let mut g = make_grid();
        buoyancy_grid_step(&mut g, [0.0; 3], 100.0);
        buoyancy_grid_reset(&mut g);
        assert!((buoyancy_grid_water_level(&g) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json_fields() {
        let g = make_grid();
        let json = buoyancy_grid_to_json(&g);
        assert!(json.contains("voxel_count"));
        assert!(json.contains("water_level"));
        assert!(json.contains("fluid_density"));
    }

    #[test]
    fn test_voxel_count_in_result() {
        let mut g = make_grid();
        let res = buoyancy_grid_step(&mut g, [0.0; 3], 0.0);
        assert_eq!(res.voxel_count, buoyancy_grid_voxel_count(&g));
    }
}
