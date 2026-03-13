// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Mesh topology visualization (poles, valence).
#[derive(Debug, Clone)]
pub struct TopologyView {
    pub enabled: bool,
    /// Highlight poles with 3 or 5+ edge connections.
    pub show_poles: bool,
    /// Highlight edges with irregular valence.
    pub show_irregular: bool,
}

pub fn new_topology_view() -> TopologyView {
    TopologyView {
        enabled: false,
        show_poles: true,
        show_irregular: true,
    }
}

pub fn tv_enable(v: &mut TopologyView) {
    v.enabled = true;
}

pub fn tv_disable(v: &mut TopologyView) {
    v.enabled = false;
}

pub fn tv_set_show_poles(v: &mut TopologyView, show: bool) {
    v.show_poles = show;
}

pub fn tv_set_show_irregular(v: &mut TopologyView, show: bool) {
    v.show_irregular = show;
}

/// Returns a colour for the given vertex valence.
pub fn tv_valence_color(valence: u32) -> [f32; 3] {
    match valence {
        3 => [1.0, 0.4, 0.0], // orange — n-pole
        4 => [0.2, 0.8, 0.2], // green — regular quad
        5 => [0.4, 0.4, 1.0], // blue — e-pole
        _ => [1.0, 0.0, 0.0], // red — irregular
    }
}

pub fn tv_is_pole(valence: u32) -> bool {
    valence == 3 || valence >= 5
}

pub fn tv_to_json(v: &TopologyView) -> String {
    format!(
        r#"{{"enabled":{},"show_poles":{},"show_irregular":{}}}"#,
        v.enabled, v.show_poles, v.show_irregular
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        /* enabled=false, flags true */
        let v = new_topology_view();
        assert!(!v.enabled);
        assert!(v.show_poles);
        assert!(v.show_irregular);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_topology_view();
        tv_enable(&mut v);
        assert!(v.enabled);
    }

    #[test]
    fn test_disable() {
        /* disable clears flag */
        let mut v = new_topology_view();
        tv_enable(&mut v);
        tv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_valence_color_regular() {
        /* valence 4 is green */
        let c = tv_valence_color(4);
        assert!(c[1] > c[0]);
    }

    #[test]
    fn test_valence_color_pole_3() {
        /* valence 3 is orange (r > g) */
        let c = tv_valence_color(3);
        assert!(c[0] > c[1]);
    }

    #[test]
    fn test_is_pole_3() {
        /* 3-valence is a pole */
        assert!(tv_is_pole(3));
    }

    #[test]
    fn test_is_pole_4_not() {
        /* 4-valence is not a pole */
        assert!(!tv_is_pole(4));
    }

    #[test]
    fn test_is_pole_5() {
        /* 5-valence is a pole */
        assert!(tv_is_pole(5));
    }

    #[test]
    fn test_to_json() {
        /* JSON has enabled key */
        assert!(tv_to_json(&new_topology_view()).contains("enabled"));
    }
}
