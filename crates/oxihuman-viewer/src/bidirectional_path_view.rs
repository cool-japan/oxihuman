// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct BdptDebugView {
    pub show_light_paths: bool,
    pub show_camera_paths: bool,
    pub max_depth: u32,
}

pub fn new_bdpt_debug_view(max_depth: u32) -> BdptDebugView {
    BdptDebugView {
        show_light_paths: true,
        show_camera_paths: true,
        max_depth,
    }
}

pub fn bdpt_path_color(depth: u32, is_light_path: bool) -> [f32; 3] {
    let t = depth as f32 / 10.0;
    if is_light_path {
        [t.clamp(0.0, 1.0), 0.0, 1.0 - t.clamp(0.0, 1.0)]
    } else {
        [0.0, t.clamp(0.0, 1.0), 1.0 - t.clamp(0.0, 1.0)]
    }
}

pub fn bdpt_connection_weight(s: u32, t: u32) -> f32 {
    /* MIS weight stub: balanced heuristic */
    let total = (s + t) as f32;
    if total < 1e-9 {
        0.0
    } else {
        1.0 / total
    }
}

pub fn bdpt_depth_color(depth: u32, max_depth: u32) -> [f32; 3] {
    let t = if max_depth == 0 {
        0.0
    } else {
        (depth as f32 / max_depth as f32).clamp(0.0, 1.0)
    };
    [t, 0.0, 1.0 - t]
}

pub fn bdpt_strategy_count(max_depth: u32) -> u32 {
    /* (s+t) from 0 to max_depth+1: count is (max_depth+2)*(max_depth+1)/2 */
    let n = max_depth + 2;
    n * (n - 1) / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bdpt_debug_view() {
        /* max_depth stored correctly */
        let v = new_bdpt_debug_view(10);
        assert_eq!(v.max_depth, 10);
    }

    #[test]
    fn test_bdpt_path_color_light() {
        /* depth=0 light path -> [0,0,1] */
        let c = bdpt_path_color(0, true);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bdpt_connection_weight() {
        /* s=1,t=1 -> 0.5 */
        let w = bdpt_connection_weight(1, 1);
        assert!((w - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_bdpt_depth_color() {
        /* depth=max -> [1,0,0] */
        let c = bdpt_depth_color(5, 5);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bdpt_strategy_count() {
        /* max_depth=2: strategies = 4*3/2 = 6 */
        let n = bdpt_strategy_count(2);
        assert_eq!(n, 6);
    }
}
