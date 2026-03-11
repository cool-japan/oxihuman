// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 1D elementary cellular automaton (Wolfram rules 0-255).

/// A 1D elementary cellular automaton.
#[derive(Debug, Clone)]
pub struct CellularAutomaton1D {
    pub cells: Vec<u8>,
    pub rule: u8,
    pub generation: u64,
}

impl CellularAutomaton1D {
    pub fn new(width: usize, rule: u8) -> Self {
        let mut cells = vec![0u8; width];
        if width > 0 {
            cells[width / 2] = 1; /* seed: single 1 in center */
        }
        Self {
            cells,
            rule,
            generation: 0,
        }
    }

    pub fn with_cells(cells: Vec<u8>, rule: u8) -> Self {
        Self {
            cells,
            rule,
            generation: 0,
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self) {
        let n = self.cells.len();
        let mut next = vec![0u8; n];
        for i in 0..n {
            let left = if i == 0 { 0 } else { self.cells[i - 1] };
            let center = self.cells[i];
            let right = if i == n - 1 { 0 } else { self.cells[i + 1] };
            let pattern = (left << 2) | (center << 1) | right;
            next[i] = (self.rule >> pattern) & 1;
        }
        self.cells = next;
        self.generation += 1;
    }

    pub fn iterate_n(&mut self, n: u64) {
        for _ in 0..n {
            self.step();
        }
    }

    pub fn live_count(&self) -> usize {
        self.cells.iter().filter(|&&c| c == 1).count()
    }

    pub fn density(&self) -> f64 {
        if self.cells.is_empty() {
            0.0
        } else {
            self.live_count() as f64 / self.cells.len() as f64
        }
    }

    pub fn width(&self) -> usize {
        self.cells.len()
    }
}

pub fn new_ca1d(width: usize, rule: u8) -> CellularAutomaton1D {
    CellularAutomaton1D::new(width, rule)
}

pub fn ca1d_step(ca: &mut CellularAutomaton1D) {
    ca.step();
}

pub fn ca1d_iterate(ca: &mut CellularAutomaton1D, n: u64) {
    ca.iterate_n(n);
}

pub fn ca1d_live_count(ca: &CellularAutomaton1D) -> usize {
    ca.live_count()
}

pub fn ca1d_density(ca: &CellularAutomaton1D) -> f64 {
    ca.density()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_single_center_cell() {
        let ca = new_ca1d(11, 30);
        assert_eq!(ca.cells[5], 1);
    }

    #[test]
    fn test_rule_0_all_die() {
        /* Rule 0: all outputs 0 */
        let mut ca = new_ca1d(10, 0);
        ca1d_step(&mut ca);
        assert_eq!(ca1d_live_count(&ca), 0);
    }

    #[test]
    fn test_rule_255_all_live() {
        /* Rule 255: all outputs 1 */
        let mut ca = new_ca1d(10, 255);
        ca1d_step(&mut ca);
        assert_eq!(ca1d_live_count(&ca), 10);
    }

    #[test]
    fn test_generation_increments() {
        let mut ca = new_ca1d(10, 30);
        ca1d_iterate(&mut ca, 5);
        assert_eq!(ca.generation, 5);
    }

    #[test]
    fn test_width() {
        let ca = new_ca1d(20, 110);
        assert_eq!(ca.width(), 20);
    }

    #[test]
    fn test_density_range() {
        let mut ca = new_ca1d(21, 30);
        ca1d_iterate(&mut ca, 10);
        let d = ca1d_density(&ca);
        assert!((0.0..=1.0).contains(&d));
    }

    #[test]
    fn test_rule_30_expands() {
        /* Rule 30 is chaotic: live count should grow from 1 initially */
        let mut ca = new_ca1d(51, 30);
        ca1d_iterate(&mut ca, 5);
        assert!(ca1d_live_count(&ca) > 1);
    }

    #[test]
    fn test_rule_110_turing_complete() {
        /* Rule 110 is Turing-complete; check it runs without panic */
        let mut ca = new_ca1d(51, 110);
        ca1d_iterate(&mut ca, 10);
        assert!(ca.generation == 10);
    }

    #[test]
    fn test_custom_initial() {
        let cells = vec![1, 0, 0, 1, 0];
        let ca = CellularAutomaton1D::with_cells(cells, 30);
        assert_eq!(ca1d_live_count(&ca), 2);
    }

    #[test]
    fn test_step_is_deterministic() {
        let mut ca1 = new_ca1d(21, 30);
        let mut ca2 = new_ca1d(21, 30);
        ca1d_step(&mut ca1);
        ca1d_step(&mut ca2);
        assert_eq!(ca1.cells, ca2.cells);
    }
}
