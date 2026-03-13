// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct UvEdgePair {
    pub v0: usize,
    pub v1: usize,
}

#[derive(Debug, Clone)]
pub struct SeamViewConfig {
    pub enabled: bool,
    pub line_width: f32,
    pub seam_color: [f32; 3],
}

pub fn default_seam_view_config() -> SeamViewConfig {
    SeamViewConfig {
        enabled: false,
        line_width: 1.5,
        seam_color: [1.0, 0.2, 0.0],
    }
}

pub fn svm_set_seam_color(v: &mut SeamViewConfig, r: f32, g: f32, b: f32) {
    v.seam_color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

pub fn svm_set_line_width(v: &mut SeamViewConfig, w: f32) {
    v.line_width = w.max(0.1);
}

pub fn svm_enable(v: &mut SeamViewConfig) {
    v.enabled = true;
}

pub fn svm_disable(v: &mut SeamViewConfig) {
    v.enabled = false;
}

pub fn svm_edge_color(v: &SeamViewConfig, is_seam: bool) -> [f32; 3] {
    if is_seam {
        v.seam_color
    } else {
        [0.5, 0.5, 0.5]
    }
}

pub fn svm_is_seam_edge(_v: &SeamViewConfig, edge: &UvEdgePair, seams: &[(usize, usize)]) -> bool {
    seams
        .iter()
        .any(|&(a, b)| (a == edge.v0 && b == edge.v1) || (a == edge.v1 && b == edge.v0))
}

pub fn svm_count_seams(seams: &[(usize, usize)]) -> usize {
    seams.len()
}

pub fn svm_to_json(v: &SeamViewConfig) -> String {
    format!(
        r#"{{"enabled":{},"line_width":{:.4},"seam_color":[{:.4},{:.4},{:.4}]}}"#,
        v.enabled, v.line_width, v.seam_color[0], v.seam_color[1], v.seam_color[2]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        /* disabled by default */
        let v = default_seam_view_config();
        assert!(!v.enabled);
        assert!((v.line_width - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_seam_color() {
        /* color stored */
        let mut v = default_seam_view_config();
        svm_set_seam_color(&mut v, 0.5, 0.6, 0.7);
        assert!((v.seam_color[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_seam_color_clamp() {
        /* clamp to [0,1] */
        let mut v = default_seam_view_config();
        svm_set_seam_color(&mut v, 2.0, -1.0, 0.5);
        assert_eq!(v.seam_color[0], 1.0);
        assert_eq!(v.seam_color[1], 0.0);
    }

    #[test]
    fn test_set_line_width() {
        /* valid width */
        let mut v = default_seam_view_config();
        svm_set_line_width(&mut v, 3.0);
        assert!((v.line_width - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_enable_disable() {
        /* enable and disable */
        let mut v = default_seam_view_config();
        svm_enable(&mut v);
        assert!(v.enabled);
        svm_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_edge_color_seam() {
        /* seam edge gets seam color */
        let v = default_seam_view_config();
        let c = svm_edge_color(&v, true);
        assert_eq!(c, v.seam_color);
    }

    #[test]
    fn test_edge_color_non_seam() {
        /* non-seam edge gets grey */
        let v = default_seam_view_config();
        let c = svm_edge_color(&v, false);
        assert_eq!(c, [0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_is_seam_edge_true() {
        /* edge in seam list */
        let v = default_seam_view_config();
        let edge = UvEdgePair { v0: 0, v1: 1 };
        let seams = vec![(0usize, 1usize)];
        assert!(svm_is_seam_edge(&v, &edge, &seams));
    }

    #[test]
    fn test_is_seam_edge_false() {
        /* edge not in seam list */
        let v = default_seam_view_config();
        let edge = UvEdgePair { v0: 2, v1: 3 };
        let seams = vec![(0usize, 1usize)];
        assert!(!svm_is_seam_edge(&v, &edge, &seams));
    }

    #[test]
    fn test_count_seams() {
        /* count correct */
        let seams = vec![(0usize, 1usize), (2, 3), (4, 5)];
        assert_eq!(svm_count_seams(&seams), 3);
    }

    #[test]
    fn test_to_json() {
        /* JSON has line_width */
        let v = default_seam_view_config();
        assert!(svm_to_json(&v).contains("line_width"));
    }
}
