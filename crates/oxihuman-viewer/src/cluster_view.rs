// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct ClusterView {
    pub enabled: bool,
    pub cluster_count: usize,
    pub show_centroids: bool,
}

pub fn new_cluster_view() -> ClusterView {
    ClusterView {
        enabled: false,
        cluster_count: 8,
        show_centroids: false,
    }
}

pub fn clv_set_cluster_count(v: &mut ClusterView, n: usize) {
    v.cluster_count = n.max(1);
}

pub fn clv_enable(v: &mut ClusterView) {
    v.enabled = true;
}

pub fn clv_toggle_centroids(v: &mut ClusterView) {
    v.show_centroids = !v.show_centroids;
}

pub fn clv_cluster_color(cluster_id: usize, total: usize) -> [f32; 3] {
    let hue = (cluster_id as f32 / total.max(1) as f32) * 360.0;
    let s = 0.8_f32;
    let v = 0.9_f32;
    /* HSV to RGB */
    let h = hue / 60.0;
    let i = h as usize % 6;
    let f = h - h.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

pub fn clv_is_enabled(v: &ClusterView) -> bool {
    v.enabled
}

pub fn clv_to_json(v: &ClusterView) -> String {
    format!(
        r#"{{"enabled":{},"cluster_count":{},"show_centroids":{}}}"#,
        v.enabled, v.cluster_count, v.show_centroids
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* disabled by default, 8 clusters */
        let v = new_cluster_view();
        assert!(!v.enabled);
        assert_eq!(v.cluster_count, 8);
    }

    #[test]
    fn test_set_cluster_count() {
        /* valid count stored */
        let mut v = new_cluster_view();
        clv_set_cluster_count(&mut v, 16);
        assert_eq!(v.cluster_count, 16);
    }

    #[test]
    fn test_set_cluster_count_min() {
        /* minimum is 1 */
        let mut v = new_cluster_view();
        clv_set_cluster_count(&mut v, 0);
        assert_eq!(v.cluster_count, 1);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_cluster_view();
        clv_enable(&mut v);
        assert!(clv_is_enabled(&v));
    }

    #[test]
    fn test_toggle_centroids() {
        /* toggle flips flag */
        let mut v = new_cluster_view();
        assert!(!v.show_centroids);
        clv_toggle_centroids(&mut v);
        assert!(v.show_centroids);
    }

    #[test]
    fn test_cluster_color_range() {
        /* colors in [0,1] */
        let c = clv_cluster_color(0, 8);
        for ch in c {
            assert!((0.0..=1.0).contains(&ch));
        }
    }

    #[test]
    fn test_cluster_color_different() {
        /* different clusters have different colors */
        let c0 = clv_cluster_color(0, 8);
        let c4 = clv_cluster_color(4, 8);
        assert!(c0 != c4);
    }

    #[test]
    fn test_to_json() {
        /* JSON has cluster_count */
        let v = new_cluster_view();
        assert!(clv_to_json(&v).contains("cluster_count"));
    }

    #[test]
    fn test_clone() {
        /* clone is independent */
        let v = new_cluster_view();
        let v2 = v.clone();
        assert_eq!(v.cluster_count, v2.cluster_count);
    }
}
