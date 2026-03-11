// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct MetropolisDebugView {
    pub show_mutations: bool,
    pub show_acceptance: bool,
    pub show_energy: bool,
}

pub fn new_metropolis_debug_view() -> MetropolisDebugView {
    MetropolisDebugView {
        show_mutations: true,
        show_acceptance: false,
        show_energy: false,
    }
}

pub fn mlt_acceptance_color(rate: f32) -> [f32; 3] {
    let r = rate.clamp(0.0, 1.0);
    [1.0 - r, r, 0.0]
}

pub fn mlt_mutation_type_color(kind: u8) -> [f32; 3] {
    match kind {
        0 => [1.0, 0.0, 0.0],
        1 => [0.0, 1.0, 0.0],
        2 => [0.0, 0.0, 1.0],
        _ => [0.5, 0.5, 0.5],
    }
}

pub fn mlt_energy_color(energy: f32, max_energy: f32) -> [f32; 3] {
    let t = if max_energy < 1e-9 {
        0.0
    } else {
        (energy / max_energy).clamp(0.0, 1.0)
    };
    [t, t * 0.5, 1.0 - t]
}

pub fn mlt_is_large_step(kind: u8) -> bool {
    kind == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_metropolis_debug_view() {
        /* show_mutations defaults to true */
        let v = new_metropolis_debug_view();
        assert!(v.show_mutations);
    }

    #[test]
    fn test_mlt_acceptance_color_low() {
        /* rate=0 -> red */
        let c = mlt_acceptance_color(0.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mlt_acceptance_color_high() {
        /* rate=1 -> green */
        let c = mlt_acceptance_color(1.0);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mlt_mutation_type_color() {
        /* kind=0 -> red */
        let c = mlt_mutation_type_color(0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_mlt_is_large_step() {
        assert!(mlt_is_large_step(0));
        assert!(!mlt_is_large_step(1));
    }
}
