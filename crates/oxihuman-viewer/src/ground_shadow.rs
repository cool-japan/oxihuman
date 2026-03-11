// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ground plane shadow projection for simple shadow effects.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GroundShadowConfig {
    pub light_direction: [f32; 3],
    pub ground_height: f32,
    pub shadow_color: [f32; 4],
    pub shadow_opacity: f32,
    pub blur_radius: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GroundShadowResult {
    pub projected_positions: Vec<[f32; 3]>,
    pub shadow_intensity: f32,
}

#[allow(dead_code)]
pub fn default_ground_shadow_config() -> GroundShadowConfig {
    GroundShadowConfig {
        light_direction: [0.0, -1.0, 0.3],
        ground_height: 0.0,
        shadow_color: [0.0, 0.0, 0.0, 0.5],
        shadow_opacity: 0.5,
        blur_radius: 0.5,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn project_point_to_ground(point: [f32; 3], light_dir: [f32; 3], ground_y: f32) -> [f32; 3] {
    if light_dir[1].abs() < 1e-8 {
        return [point[0], ground_y, point[2]];
    }
    let t = (ground_y - point[1]) / light_dir[1];
    [
        point[0] + light_dir[0] * t,
        ground_y,
        point[2] + light_dir[2] * t,
    ]
}

#[allow(dead_code)]
pub fn compute_shadow_matrix(light_dir: [f32; 3], ground_y: f32) -> [[f32; 4]; 4] {
    let d = light_dir;
    let ny = 1.0;
    let dot = d[1] * ny;
    [
        [dot - d[0] * 0.0, -d[1] * 0.0, -d[2] * 0.0, 0.0],
        [-d[0] * ny, dot - d[1] * ny, -d[2] * ny, 0.0],
        [-d[0] * 0.0, -d[1] * 0.0, dot - d[2] * 0.0, 0.0],
        [-d[0] * (-ground_y), -d[1] * (-ground_y), -d[2] * (-ground_y), dot],
    ]
}

#[allow(dead_code)]
pub fn project_vertices(vertices: &[[f32; 3]], cfg: &GroundShadowConfig) -> GroundShadowResult {
    let projected: Vec<[f32; 3]> = vertices
        .iter()
        .map(|v| project_point_to_ground(*v, cfg.light_direction, cfg.ground_height))
        .collect();
    GroundShadowResult {
        projected_positions: projected,
        shadow_intensity: cfg.shadow_opacity,
    }
}

#[allow(dead_code)]
pub fn ground_shadow_to_json(cfg: &GroundShadowConfig) -> String {
    format!(
        r#"{{"ground_y":{},"opacity":{},"blur":{},"enabled":{}}}"#,
        cfg.ground_height, cfg.shadow_opacity, cfg.blur_radius, cfg.enabled
    )
}

#[allow(dead_code)]
pub fn shadow_bounds(result: &GroundShadowResult) -> ([f32; 3], [f32; 3]) {
    if result.projected_positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut min = result.projected_positions[0];
    let mut max = result.projected_positions[0];
    for p in &result.projected_positions {
        for i in 0..3 {
            if p[i] < min[i] { min[i] = p[i]; }
            if p[i] > max[i] { max[i] = p[i]; }
        }
    }
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_ground_shadow_config();
        assert!(c.enabled);
        assert!((c.ground_height).abs() < 1e-6);
    }

    #[test]
    fn test_project_vertical() {
        let p = project_point_to_ground([0.0, 2.0, 0.0], [0.0, -1.0, 0.0], 0.0);
        assert!((p[1]).abs() < 1e-6);
        assert!((p[0]).abs() < 1e-6);
    }

    #[test]
    fn test_project_angled() {
        let p = project_point_to_ground([0.0, 2.0, 0.0], [1.0, -1.0, 0.0], 0.0);
        assert!((p[1]).abs() < 1e-6);
        assert!((p[0] - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_project_horizontal_light() {
        let p = project_point_to_ground([0.0, 2.0, 0.0], [1.0, 0.0, 0.0], 0.0);
        assert!((p[1]).abs() < 1e-6);
    }

    #[test]
    fn test_shadow_matrix() {
        let m = compute_shadow_matrix([0.0, -1.0, 0.0], 0.0);
        assert!((m[3][3] - (-1.0)).abs() < 1e-4);
    }

    #[test]
    fn test_project_vertices() {
        let verts = vec![[0.0, 1.0, 0.0], [1.0, 2.0, 0.0]];
        let cfg = default_ground_shadow_config();
        let r = project_vertices(&verts, &cfg);
        assert_eq!(r.projected_positions.len(), 2);
    }

    #[test]
    fn test_to_json() {
        let cfg = default_ground_shadow_config();
        let j = ground_shadow_to_json(&cfg);
        assert!(j.contains("opacity"));
    }

    #[test]
    fn test_shadow_bounds_empty() {
        let r = GroundShadowResult {
            projected_positions: vec![],
            shadow_intensity: 0.5,
        };
        let (mn, mx) = shadow_bounds(&r);
        assert!(mn[0].abs() < 1e-6);
        assert!(mx[0].abs() < 1e-6);
    }

    #[test]
    fn test_shadow_bounds() {
        let r = GroundShadowResult {
            projected_positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 2.0]],
            shadow_intensity: 0.5,
        };
        let (mn, mx) = shadow_bounds(&r);
        assert!(mn[0].abs() < 1e-6);
        assert!((mx[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_project_on_ground() {
        let verts = vec![[0.0, 0.0, 0.0]];
        let cfg = default_ground_shadow_config();
        let r = project_vertices(&verts, &cfg);
        assert!((r.projected_positions[0][1]).abs() < 1e-6);
    }
}
