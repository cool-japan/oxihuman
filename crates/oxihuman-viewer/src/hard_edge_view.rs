// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hard/sharp edge highlight visualization.

/// Hard edge view configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HardEdgeConfig {
    pub angle_threshold_deg: f32,
    pub hard_color: [f32; 3],
    pub soft_color: [f32; 3],
    pub line_width: f32,
    pub enabled: bool,
}

impl Default for HardEdgeConfig {
    fn default() -> Self {
        HardEdgeConfig {
            angle_threshold_deg: 30.0,
            hard_color: [1.0, 0.8, 0.0],
            soft_color: [0.3, 0.3, 0.3],
            line_width: 1.5,
            enabled: false,
        }
    }
}

#[allow(dead_code)]
pub fn default_hard_edge_config() -> HardEdgeConfig {
    HardEdgeConfig::default()
}

#[allow(dead_code)]
pub fn he_enable(cfg: &mut HardEdgeConfig) {
    cfg.enabled = true;
}

#[allow(dead_code)]
pub fn he_disable(cfg: &mut HardEdgeConfig) {
    cfg.enabled = false;
}

#[allow(dead_code)]
pub fn he_set_threshold(cfg: &mut HardEdgeConfig, deg: f32) {
    cfg.angle_threshold_deg = deg.clamp(0.0, 180.0);
}

#[allow(dead_code)]
pub fn he_set_line_width(cfg: &mut HardEdgeConfig, w: f32) {
    cfg.line_width = w.clamp(0.5, 20.0);
}

/// Compute the dihedral angle (degrees) between two face normals.
#[allow(dead_code)]
pub fn he_dihedral_angle_deg(n1: [f32; 3], n2: [f32; 3]) -> f32 {
    let dot = (n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2]).clamp(-1.0, 1.0);
    dot.acos().to_degrees()
}

/// Determine if an edge is hard based on dihedral angle.
#[allow(dead_code)]
pub fn he_is_hard_edge(cfg: &HardEdgeConfig, n1: [f32; 3], n2: [f32; 3]) -> bool {
    he_dihedral_angle_deg(n1, n2) >= cfg.angle_threshold_deg
}

/// Return the display color for the edge.
#[allow(dead_code)]
pub fn he_edge_color(cfg: &HardEdgeConfig, is_hard: bool) -> [f32; 3] {
    if is_hard {
        cfg.hard_color
    } else {
        cfg.soft_color
    }
}

/// Count hard edges in a list of normal pairs.
#[allow(dead_code)]
pub fn he_count_hard(cfg: &HardEdgeConfig, pairs: &[([f32; 3], [f32; 3])]) -> usize {
    pairs
        .iter()
        .filter(|(n1, n2)| he_is_hard_edge(cfg, *n1, *n2))
        .count()
}

#[allow(dead_code)]
pub fn he_to_json(cfg: &HardEdgeConfig) -> String {
    format!(
        r#"{{"angle_threshold_deg":{:.2},"line_width":{:.4},"enabled":{}}}"#,
        cfg.angle_threshold_deg, cfg.line_width, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        assert!(!default_hard_edge_config().enabled);
    }

    #[test]
    fn parallel_normals_zero_angle() {
        let n = [0.0f32, 1.0, 0.0];
        assert!(he_dihedral_angle_deg(n, n) < 1e-4);
    }

    #[test]
    fn opposite_normals_180_deg() {
        let n1 = [0.0f32, 1.0, 0.0];
        let n2 = [0.0f32, -1.0, 0.0];
        assert!((he_dihedral_angle_deg(n1, n2) - 180.0).abs() < 0.01);
    }

    #[test]
    fn perpendicular_normals_90_deg() {
        let n1 = [1.0f32, 0.0, 0.0];
        let n2 = [0.0f32, 1.0, 0.0];
        assert!((he_dihedral_angle_deg(n1, n2) - 90.0).abs() < 0.01);
    }

    #[test]
    fn is_hard_edge_perpendicular() {
        let cfg = default_hard_edge_config();
        let n1 = [1.0f32, 0.0, 0.0];
        let n2 = [0.0f32, 1.0, 0.0];
        assert!(he_is_hard_edge(&cfg, n1, n2));
    }

    #[test]
    fn not_hard_edge_parallel() {
        let cfg = default_hard_edge_config();
        let n = [0.0f32, 1.0, 0.0];
        assert!(!he_is_hard_edge(&cfg, n, n));
    }

    #[test]
    fn count_hard_edges() {
        let cfg = default_hard_edge_config();
        let n1 = [1.0f32, 0.0, 0.0];
        let n2 = [0.0f32, 1.0, 0.0];
        let n3 = [0.0f32, 1.0, 0.0];
        let pairs = vec![(n1, n2), (n3, n3)];
        assert_eq!(he_count_hard(&cfg, &pairs), 1);
    }

    #[test]
    fn threshold_clamps() {
        let mut cfg = default_hard_edge_config();
        he_set_threshold(&mut cfg, 200.0);
        assert!((cfg.angle_threshold_deg - 180.0).abs() < 1e-6);
    }

    #[test]
    fn to_json_has_threshold() {
        assert!(he_to_json(&default_hard_edge_config()).contains("angle_threshold_deg"));
    }
}
