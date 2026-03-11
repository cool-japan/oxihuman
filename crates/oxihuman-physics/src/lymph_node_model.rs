// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct LymphNode {
    pub volume_ml: f32,
    pub filtration_efficiency: f32,
    pub pathogen_load: f32,
    pub immune_activity: f32,
}

pub fn new_lymph_node(volume_ml: f32) -> LymphNode {
    LymphNode {
        volume_ml,
        filtration_efficiency: 0.9,
        pathogen_load: 0.0,
        immune_activity: 0.0,
    }
}

pub fn lymph_node_filter(n: &mut LymphNode, input_load: f32, dt: f32) {
    let filtered = input_load * n.filtration_efficiency;
    n.pathogen_load += filtered * dt;
    n.immune_activity = (n.pathogen_load / 100.0).clamp(0.0, 1.0);
}

pub fn lymph_node_output_load(n: &LymphNode) -> f32 {
    n.pathogen_load * (1.0 - n.filtration_efficiency).max(0.0)
}

pub fn lymph_node_is_activated(n: &LymphNode) -> bool {
    n.immune_activity > 0.3
}

pub fn lymph_node_swelling(n: &LymphNode) -> f32 {
    n.volume_ml * (1.0 + n.immune_activity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_pathogen_zero() {
        /* fresh node has no pathogen load */
        let n = new_lymph_node(0.5);
        assert_eq!(n.pathogen_load, 0.0);
    }

    #[test]
    fn filter_increases_load() {
        /* filtering accumulates pathogen load */
        let mut n = new_lymph_node(0.5);
        lymph_node_filter(&mut n, 10.0, 1.0);
        assert!(n.pathogen_load > 0.0);
    }

    #[test]
    fn output_load_less_than_input() {
        /* output load is less than input for efficient node */
        let mut n = new_lymph_node(0.5);
        lymph_node_filter(&mut n, 10.0, 1.0);
        assert!(lymph_node_output_load(&n) < n.pathogen_load);
    }

    #[test]
    fn not_activated_initially() {
        /* no activation without pathogens */
        let n = new_lymph_node(0.5);
        assert!(!lymph_node_is_activated(&n));
    }

    #[test]
    fn activated_after_high_load() {
        /* high load activates the node */
        let mut n = new_lymph_node(0.5);
        lymph_node_filter(&mut n, 1000.0, 1.0);
        assert!(lymph_node_is_activated(&n));
    }

    #[test]
    fn swelling_increases_with_activity() {
        /* swollen node is larger than at rest */
        let mut n = new_lymph_node(1.0);
        lymph_node_filter(&mut n, 1000.0, 1.0);
        let swollen = lymph_node_swelling(&n);
        assert!(swollen > n.volume_ml);
    }

    #[test]
    fn swelling_at_rest_equals_volume() {
        /* no immune activity → no extra swelling */
        let n = new_lymph_node(1.0);
        assert!((lymph_node_swelling(&n) - 1.0).abs() < 1e-5);
    }
}
